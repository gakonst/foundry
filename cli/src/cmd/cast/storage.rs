use crate::{
    cmd::forge::{build, inspect::print_storage_layout},
    opts::cast::{parse_block_id, parse_name_or_address, parse_slot},
    utils::consume_config_rpc_url,
};
use cast::Cast;
use clap::Parser;
use ethers::{
    prelude::*,
    solc::artifacts::{output_selection::ContractOutputSelection, Optimizer, Settings},
};
use eyre::{ContextCompat, Result};
use foundry_common::{compile::compile, try_get_http_provider};
use foundry_config::Config;

#[derive(Debug, Clone, Parser)]
pub struct StorageArgs {
    // Storage
    #[clap(help = "The contract address.", parse(try_from_str = parse_name_or_address), value_name = "ADDRESS")]
    address: NameOrAddress,
    #[clap(help = "The storage slot number (hex or decimal)", parse(try_from_str = parse_slot), value_name = "SLOT")]
    slot: Option<H256>,
    #[clap(long, env = "ETH_RPC_URL", value_name = "URL")]
    rpc_url: Option<String>,
    #[clap(
        long,
        short = 'B',
        help = "The block height you want to query at.",
        long_help = "The block height you want to query at. Can also be the tags earliest, latest, or pending.",
        parse(try_from_str = parse_block_id),
        value_name = "BLOCK"
    )]
    block: Option<BlockId>,

    // Etherscan
    #[clap(long, short, env = "ETHERSCAN_API_KEY", help = "etherscan API key", value_name = "KEY")]
    etherscan_api_key: Option<String>,
    #[clap(
        long,
        visible_alias = "chain-id",
        env = "CHAIN",
        help = "The chain ID the contract is deployed to.",
        default_value = "mainnet",
        value_name = "CHAIN"
    )]
    chain: Chain,

    // Forge
    #[clap(flatten)]
    build: build::CoreBuildArgs,
}

impl StorageArgs {
    pub async fn run(self) -> Result<()> {
        let StorageArgs { address, block, build, rpc_url, slot, chain, etherscan_api_key } = self;

        let rpc_url = consume_config_rpc_url(rpc_url);
        let provider = try_get_http_provider(rpc_url)?;

        let address = match address {
            NameOrAddress::Name(name) => provider.resolve_name(&name).await?,
            NameOrAddress::Address(address) => address,
        };

        // Slot was provided, perform a simple RPC call
        if let Some(slot) = slot {
            let cast = Cast::new(provider);
            println!("{}", cast.storage(address, slot, block).await?);
            return Ok(())
        }

        // No slot was provided

        // Get deployed bytecode at given address
        let address_code = provider.get_code(address, block).await?;
        if address_code.is_empty() {
            eyre::bail!("Provided address has no deployed code and thus no storage");
        }

        // Check if we're in a forge project
        let project = build.project()?;
        if project.paths.has_input_files() {
            // Find in artifacts and pretty print
            let project = with_storage_layout_output(project);
            let out = compile(&project, false, false)?;
            let artifact = out.artifacts().find(|(_, artifact)| match artifact.deployed_bytecode {
                Some(ref deployed_code) => match deployed_code.bytecode {
                    Some(ref bytecode) => match bytecode.object.as_bytes() {
                        Some(bytes) => bytes == &address_code,
                        None => false,
                    },
                    None => false,
                },
                None => false,
            });
            if let Some((_, artifact)) = artifact {
                return print_storage_layout(&artifact.storage_layout, true)
            }
        }

        // Not a forge project or artifact not found
        // Get code from Etherscan
        let api_key = etherscan_api_key.or_else(|| {
            let config = Config::load();
            config.get_etherscan_api_key(Some(chain))
        }).ok_or_else(|| eyre::eyre!("No Etherscan API Key is set. Consider using the ETHERSCAN_API_KEY env var, or setting the -e CLI argument or etherscan-api-key in foundry.toml"))?;
        let client = ethers::etherscan::Client::new(chain, api_key)?;
        println!("No artifacts found, fetching source code from etherscan...");
        let source = client.contract_source_code(address).await?;
        let source_tree = source.source_tree()?;

        // Create a new temp project
        let root = tempfile::tempdir()?;
        let root_path = root.path();
        // let root = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/temp_build"));
        // let root_path = root.as_path();
        source_tree.write_to(root_path)?;

        // Configure Solc
        let paths = ProjectPathsConfig::builder().sources(root_path).build_with_root(root_path);

        let metadata = &source.items[0];
        let mut settings = Settings::default();

        let mut optimizer = Optimizer::default();
        if parse_etherscan_bool(&metadata.optimization_used)? {
            optimizer.enable();
            match metadata.runs.parse::<usize>() {
                Ok(runs) => optimizer.runs(runs),
                _ => {}
            };
        }
        settings.optimizer = optimizer;
        if !metadata.source_code.contains("pragma solidity") {
            eyre::bail!("Only Solidity verified contracts are allowed")
        }
        settings.evm_version = Some(metadata.evm_version.parse().unwrap_or_default());

        let solc = match parse_etherscan_compiler_version(&metadata.compiler_version) {
            Ok(v) => Solc::find_or_install_svm_version(v)?,
            Err(_) => Solc::default(),
        }
        .with_base_path(root_path);
        let solc_config = SolcConfig::builder().settings(settings).build();

        let project = Project::builder()
            .solc(solc)
            .solc_config(solc_config)
            .no_auto_detect()
            .ephemeral()
            .no_artifacts()
            .ignore_error_code(1878) // License warning
            .ignore_error_code(5574) // Contract code size warning
            .paths(paths)
            .build()?;
        let mut project = with_storage_layout_output(project);

        // Compile
        let out = match compile(&project, false, false) {
            Ok(out) => Ok(out),
            // metadata does not contain many compiler settings...
            Err(e) => {
                if e.to_string().contains("--via-ir") {
                    project.solc_config.settings.via_ir = Some(true);
                    compile(&project, false, false)
                } else {
                    Err(e)
                }
            }
        }?;
        let artifact = out.artifacts().find(|(name, _)| name == &metadata.contract_name);
        let artifact = artifact.wrap_err("Compilation failed")?.1;
        print_storage_layout(&artifact.storage_layout, true)
    }
}

fn with_storage_layout_output(mut project: Project) -> Project {
    let mut outputs = ContractOutputSelection::basic();
    outputs.push(ContractOutputSelection::Metadata);
    outputs.push(ContractOutputSelection::StorageLayout);
    let settings = project.solc_config.settings.with_extra_output(outputs);

    project.solc_config.settings = settings;
    project
}

/// Usually 0 or 1
fn parse_etherscan_bool(s: &str) -> Result<bool> {
    let s = s.trim();
    match s.parse::<u8>() {
        Ok(n) => match n {
            0 | 1 => Ok(n != 0),
            _ => Err(eyre::eyre!("error parsing bool value from etherscan: number is not 0 or 1")),
        },
        Err(e) => match s.parse::<bool>() {
            Ok(b) => Ok(b),
            Err(_) => Err(eyre::eyre!("error parsing bool value from etherscan: {}", e)),
        },
    }
}

fn parse_etherscan_compiler_version(s: &str) -> Result<&str> {
    // "v0.6.8+commit.0bbfe453"
    let mut version = s.trim().split('+');
    // "v0.6.8"
    let version = version.next().wrap_err("got empty compiler version from etherscan")?;
    // "0.6.8"
    Ok(version.strip_prefix('v').unwrap_or(version))
}
