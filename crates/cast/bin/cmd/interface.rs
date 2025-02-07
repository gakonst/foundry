use alloy_json_abi::{ContractObject, JsonAbi};
use alloy_primitives::Address;
use clap::Parser;
use eyre::{Context, Result};
use foundry_block_explorers::Client;
use foundry_cli::{opts::EtherscanOpts, utils::LoadConfig};
use foundry_common::{
    compile::{PathOrContractInfo, ProjectCompiler},
    find_target_path, fs, shell, ContractsByArtifact,
};
use foundry_config::load_config;
use itertools::Itertools;
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

/// CLI arguments for `cast interface`.
#[derive(Clone, Debug, Parser)]
pub struct InterfaceArgs {
    /// The target contract, which can be one of:
    /// - A file path to an ABI JSON file.
    /// - A contract identifier in the form `<path>:<contractname>` or just `<contractname>`.
    /// - An Ethereum address, for which the ABI will be fetched from Etherscan.
    contract: String,

    /// The name to use for the generated interface.
    ///
    /// Only relevant when retrieving the ABI from a file.
    #[arg(long, short)]
    name: Option<String>,

    /// Solidity pragma version.
    #[arg(long, short, default_value = "^0.8.4", value_name = "VERSION")]
    pragma: String,

    /// The path to the output file.
    ///
    /// If not specified, the interface will be output to stdout.
    #[arg(
        short,
        long,
        value_hint = clap::ValueHint::FilePath,
        value_name = "PATH",
    )]
    output: Option<PathBuf>,

    #[command(flatten)]
    etherscan: EtherscanOpts,
}

impl InterfaceArgs {
    pub async fn run(self) -> Result<()> {
        let Self { contract, name, pragma, output: output_location, etherscan } = self;

        // Determine if the target contract is an ABI file, a local contract or an Ethereum address.
        let abis = if Path::new(&contract).is_file() &&
            fs::read_to_string(&contract)
                .ok()
                .and_then(|content| serde_json::from_str::<Value>(&content).ok())
                .is_some()
        {
            load_abi_from_file(&contract, name)?
        } else {
            match Address::from_str(&contract) {
                Ok(address) => fetch_abi_from_etherscan(address, &etherscan).await?,
                Err(_) => load_abi_from_artifact(&contract)?,
            }
        };

        // Retrieve interfaces from the array of ABIs.
        let interfaces = get_interfaces(abis)?;

        // Print result or write to file.
        let res = if shell::is_json() {
            // Format as JSON.
            interfaces.iter().map(|iface| &iface.json_abi).format("\n").to_string()
        } else {
            // Format as Solidity.
            format!(
                "// SPDX-License-Identifier: UNLICENSED\n\
                 pragma solidity {pragma};\n\n\
                 {}",
                interfaces.iter().map(|iface| &iface.source).format("\n")
            )
        };

        if let Some(loc) = output_location {
            if let Some(parent) = loc.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&loc, res)?;
            sh_println!("Saved interface at {}", loc.display())?;
        } else {
            sh_print!("{res}")?;
        }

        Ok(())
    }
}

struct InterfaceSource {
    json_abi: String,
    source: String,
}

/// Load the ABI from a file.
pub fn load_abi_from_file(path: &str, name: Option<String>) -> Result<Vec<(JsonAbi, String)>> {
    let file = std::fs::read_to_string(path).wrap_err("unable to read abi file")?;
    let obj: ContractObject = serde_json::from_str(&file)?;
    let abi = obj.abi.ok_or_else(|| eyre::eyre!("could not find ABI in file {path}"))?;
    let name = name.unwrap_or_else(|| "Interface".to_owned());
    Ok(vec![(abi, name)])
}

/// Load the ABI from the artifact of a locally compiled contract.
fn load_abi_from_artifact(path_or_contract: &str) -> Result<Vec<(JsonAbi, String)>> {
    let config = load_config()?;
    let project = config.project()?;
    let compiler = ProjectCompiler::new().quiet(true);

    let contract = PathOrContractInfo::from_str(path_or_contract)?;

    let target_path = find_target_path(&project, &contract)?;
    let output = compiler.files([target_path.clone()]).compile(&project)?;

    let contracts_by_artifact = ContractsByArtifact::from(output);

    let maybe_abi = contracts_by_artifact
        .find_abi_by_name_or_src_path(contract.name().unwrap_or(&target_path.to_string_lossy()));

    let (abi, name) =
        maybe_abi.as_ref().ok_or_else(|| eyre::eyre!("Failed to fetch lossless ABI"))?;

    Ok(vec![(abi.clone(), contract.name().unwrap_or(name).to_string())])
}

/// Fetches the ABI of a contract from Etherscan.
pub async fn fetch_abi_from_etherscan(
    address: Address,
    etherscan: &EtherscanOpts,
) -> Result<Vec<(JsonAbi, String)>> {
    let config = etherscan.load_config()?;
    let chain = config.chain.unwrap_or_default();
    let api_key = config.get_etherscan_api_key(Some(chain)).unwrap_or_default();
    let client = Client::new(chain, api_key)?;
    let source = client.contract_source_code(address).await?;
    source.items.into_iter().map(|item| Ok((item.abi()?, item.contract_name))).collect()
}

/// Converts a vector of tuples containing the ABI and contract name into a vector of
/// `InterfaceSource` objects.
fn get_interfaces(abis: Vec<(JsonAbi, String)>) -> Result<Vec<InterfaceSource>> {
    abis.into_iter()
        .map(|(contract_abi, name)| {
            let source = match foundry_cli::utils::abi_to_solidity(&contract_abi, &name) {
                Ok(generated_source) => generated_source,
                Err(e) => {
                    warn!("Failed to format interface for {name}: {e}");
                    contract_abi.to_sol(&name, None)
                }
            };
            Ok(InterfaceSource { json_abi: serde_json::to_string_pretty(&contract_abi)?, source })
        })
        .collect()
}
