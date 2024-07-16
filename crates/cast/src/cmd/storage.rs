use crate::{opts::parse_slot, Cast};
use alloy_network::AnyNetwork;
use alloy_primitives::{Address, B256, U256};
use alloy_provider::Provider;
use alloy_rpc_types::BlockId;
use alloy_transport::Transport;
use clap::Parser;
use comfy_table::{presets::ASCII_MARKDOWN, Table};
use eyre::Result;
use foundry_block_explorers::Client;
use foundry_cli::{
    opts::{CoreBuildArgs, EtherscanOpts, RpcOpts},
    utils,
};
use foundry_common::{
    abi::find_source,
    compile::{etherscan_project, ProjectCompiler},
    ens::NameOrAddress,
};
use foundry_compilers::{
    artifacts::{ConfigurableContractArtifact, StorageLayout},
    compilers::{
        solc::{Solc, SolcCompiler},
        Compiler, CompilerSettings,
    },
    Artifact, Project,
};
use foundry_config::{
    figment::{self, value::Dict, Metadata, Profile},
    impl_figment_convert_cast, Config,
};
use semver::Version;
use std::str::FromStr;

/// The minimum Solc version for outputting storage layouts.
///
/// https://github.com/ethereum/solidity/blob/develop/Changelog.md#065-2020-04-06
const MIN_SOLC: Version = Version::new(0, 6, 5);

/// CLI arguments for `cast storage`.
#[derive(Clone, Debug, Parser)]
pub struct StorageArgs {
    /// The contract address.
    #[arg(value_parser = NameOrAddress::from_str)]
    address: NameOrAddress,

    /// The storage slot number.
    #[arg(value_parser = parse_slot)]
    slot: Option<B256>,

    /// The block height to query at.
    ///
    /// Can also be the tags earliest, finalized, safe, latest, or pending.
    #[arg(long, short)]
    block: Option<BlockId>,

    #[command(flatten)]
    rpc: RpcOpts,

    #[command(flatten)]
    etherscan: EtherscanOpts,

    #[command(flatten)]
    build: CoreBuildArgs,
}

impl_figment_convert_cast!(StorageArgs);

impl figment::Provider for StorageArgs {
    fn metadata(&self) -> Metadata {
        Metadata::named("StorageArgs")
    }

    fn data(&self) -> Result<figment::value::Map<Profile, Dict>, figment::Error> {
        let mut map = self.build.data()?;
        let dict = map.get_mut(&Config::selected_profile()).unwrap();
        dict.extend(self.rpc.dict());
        dict.extend(self.etherscan.dict());
        Ok(map)
    }
}

impl StorageArgs {
    pub async fn run(self) -> Result<()> {
        let config = Config::from(&self);

        let Self { address, slot, block, build, .. } = self;
        let provider = utils::get_provider(&config)?;
        let address = address.resolve(&provider).await?;

        // Slot was provided, perform a simple RPC call
        if let Some(slot) = slot {
            let cast = Cast::new(provider);
            println!("{}", cast.storage(address, slot, block).await?);
            return Ok(());
        }

        // No slot was provided
        // Get deployed bytecode at given address
        let address_code =
            provider.get_code_at(address).block_id(block.unwrap_or_default()).await?;
        if address_code.is_empty() {
            eyre::bail!("Provided address has no deployed code and thus no storage");
        }

        // Check if we're in a forge project and if we can find the address' code
        let mut project = build.project()?;
        if project.paths.has_input_files() {
            // Find in artifacts and pretty print
            add_storage_layout_output(&mut project);
            let out = ProjectCompiler::new().compile(&project)?;
            let artifact = out.artifacts().find(|(_, artifact)| {
                artifact.get_deployed_bytecode_bytes().is_some_and(|b| *b == address_code)
            });
            if let Some((_, artifact)) = artifact {
                return fetch_and_print_storage(provider, address, block, artifact, true).await;
            }
        }

        // Not a forge project or artifact not found
        // Get code from Etherscan
        eprintln!("No matching artifacts found, fetching source code from Etherscan...");

        if !self.etherscan.has_key() {
            eyre::bail!("You must provide an Etherscan API key if you're fetching a remote contract's storage.");
        }

        let chain = utils::get_chain(config.chain, &provider).await?;
        let api_key = config.get_etherscan_api_key(Some(chain)).unwrap_or_default();
        let client = Client::new(chain, api_key)?;
        let source = find_source(client, address).await?;
        let metadata = source.items.first().unwrap();
        if metadata.is_vyper() {
            eyre::bail!("Contract at provided address is not a valid Solidity contract")
        }

        let version = metadata.compiler_version()?;
        let auto_detect = version < MIN_SOLC;

        // Create a new temp project
        // TODO: Cache instead of using a temp directory: metadata from Etherscan won't change
        let root = tempfile::tempdir()?;
        let root_path = root.path();
        let mut project = etherscan_project(metadata, root_path)?;
        add_storage_layout_output(&mut project);

        project.compiler = if auto_detect {
            SolcCompiler::AutoDetect
        } else {
            SolcCompiler::Specific(Solc::find_or_install(&version)?)
        };

        // Compile
        let mut out = ProjectCompiler::new().quiet(true).compile(&project)?;
        let artifact = {
            let (_, mut artifact) = out
                .artifacts()
                .find(|(name, _)| name == &metadata.contract_name)
                .ok_or_else(|| eyre::eyre!("Could not find artifact"))?;

            if is_storage_layout_empty(&artifact.storage_layout) && auto_detect {
                // try recompiling with the minimum version
                eprintln!("The requested contract was compiled with {version} while the minimum version for storage layouts is {MIN_SOLC} and as a result the output may be empty.");
                let solc = Solc::find_or_install(&MIN_SOLC)?;
                project.compiler = SolcCompiler::Specific(solc);
                if let Ok(output) = ProjectCompiler::new().quiet(true).compile(&project) {
                    out = output;
                    let (_, new_artifact) = out
                        .artifacts()
                        .find(|(name, _)| name == &metadata.contract_name)
                        .ok_or_else(|| eyre::eyre!("Could not find artifact"))?;
                    artifact = new_artifact;
                }
            }

            artifact
        };

        // Clear temp directory
        root.close()?;

        fetch_and_print_storage(provider, address, block, artifact, true).await
    }
}

/// Represents the value of a storage slot `eth_getStorageAt` call.
#[derive(Clone, Debug, PartialEq, Eq)]
struct StorageValue {
    /// The slot number.
    slot: B256,
    /// The value as returned by `eth_getStorageAt`.
    raw_slot_value: B256,
}

impl StorageValue {
    /// Returns the value of the storage slot, applying the offset if necessary.
    fn value(&self, offset: i64, number_of_bytes: Option<usize>) -> B256 {
        let offset = offset as usize;
        let mut end = 32;
        if let Some(number_of_bytes) = number_of_bytes {
            end = offset + number_of_bytes;
            if end > 32 {
                end = 32;
            }
        }

        // reverse range, because the value is stored in big endian
        let raw_sliced_value = &self.raw_slot_value.as_slice()[32 - end..32 - offset];

        // copy the raw sliced value as tail
        let mut value = [0u8; 32];
        value[32 - raw_sliced_value.len()..32].copy_from_slice(raw_sliced_value);
        B256::from(value)
    }
}

async fn fetch_and_print_storage<P: Provider<T, AnyNetwork>, T: Transport + Clone>(
    provider: P,
    address: Address,
    block: Option<BlockId>,
    artifact: &ConfigurableContractArtifact,
    pretty: bool,
) -> Result<()> {
    if is_storage_layout_empty(&artifact.storage_layout) {
        eprintln!("Storage layout is empty.");
        Ok(())
    } else {
        let layout = artifact.storage_layout.as_ref().unwrap().clone();
        let values = fetch_storage_slots(provider, address, block, &layout).await?;
        print_storage(layout, values, pretty)
    }
}

async fn fetch_storage_slots<P: Provider<T, AnyNetwork>, T: Transport + Clone>(
    provider: P,
    address: Address,
    block: Option<BlockId>,
    layout: &StorageLayout,
) -> Result<Vec<StorageValue>> {
    let requests = layout.storage.iter().map(|storage_slot| async {
        let slot = B256::from(U256::from_str(&storage_slot.slot)?);
        let raw_slot_value = provider
            .get_storage_at(address, slot.into())
            .block_id(block.unwrap_or_default())
            .await?;

        let value = StorageValue { slot, raw_slot_value: raw_slot_value.into() };

        Ok(value)
    });

    futures::future::try_join_all(requests).await
}

fn print_storage(layout: StorageLayout, values: Vec<StorageValue>, pretty: bool) -> Result<()> {
    if !pretty {
        println!("{}", serde_json::to_string_pretty(&serde_json::to_value(layout)?)?);
        return Ok(())
    }

    let mut table = Table::new();
    table.load_preset(ASCII_MARKDOWN);
    table.set_header(["Name", "Type", "Slot", "Offset", "Bytes", "Value", "Hex Value", "Contract"]);

    for (slot, storage_value) in layout.storage.into_iter().zip(values) {
        let storage_type = layout.types.get(&slot.storage_type);
        let value = storage_value
            .value(slot.offset, storage_type.and_then(|t| t.number_of_bytes.parse::<usize>().ok()));
        let converted_value = U256::from_be_bytes(value.0);

        table.add_row([
            slot.label.as_str(),
            storage_type.map_or("?", |t| &t.label),
            &slot.slot,
            &slot.offset.to_string(),
            storage_type.map_or("?", |t| &t.number_of_bytes),
            &converted_value.to_string(),
            &value.to_string(),
            &slot.contract,
        ]);
    }

    println!("{table}");

    Ok(())
}

fn add_storage_layout_output<C: Compiler>(project: &mut Project<C>) {
    project.artifacts.additional_values.storage_layout = true;
    project.settings.update_output_selection(|selection| {
        selection.0.values_mut().for_each(|contract_selection| {
            contract_selection
                .values_mut()
                .for_each(|selection| selection.push("storageLayout".to_string()))
        });
    })
}

fn is_storage_layout_empty(storage_layout: &Option<StorageLayout>) -> bool {
    if let Some(ref s) = storage_layout {
        s.storage.is_empty()
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_storage_etherscan_api_key() {
        let args =
            StorageArgs::parse_from(["foundry-cli", "addr", "--etherscan-api-key", "dummykey"]);
        assert_eq!(args.etherscan.key(), Some("dummykey".to_string()));

        std::env::set_var("ETHERSCAN_API_KEY", "FXY");
        let config = Config::from(&args);
        std::env::remove_var("ETHERSCAN_API_KEY");
        assert_eq!(config.etherscan_api_key, Some("dummykey".to_string()));

        let key = config.get_etherscan_api_key(None).unwrap();
        assert_eq!(key, "dummykey".to_string());
    }
}
