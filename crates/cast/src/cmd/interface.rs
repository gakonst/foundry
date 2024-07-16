use alloy_chains::Chain;
use alloy_json_abi::ContractObject;
use alloy_primitives::Address;
use clap::Parser;
use eyre::{Context, Result};
use foundry_block_explorers::Client;
use foundry_cli::opts::EtherscanOpts;
use foundry_common::fs;
use foundry_config::Config;
use itertools::Itertools;
use std::path::{Path, PathBuf};
use tracing::warn;

/// CLI arguments for `cast interface`.
#[derive(Clone, Debug, Parser)]
pub struct InterfaceArgs {
    /// The contract address, or the path to an ABI file.
    ///
    /// If an address is specified, then the ABI is fetched from Etherscan.
    path_or_address: String,

    /// The name to use for the generated interface.
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

    /// If specified, the interface will be output as JSON rather than Solidity.
    #[arg(long, short)]
    json: bool,

    #[command(flatten)]
    etherscan: EtherscanOpts,
}

impl InterfaceArgs {
    pub async fn run(self) -> Result<()> {
        let Self { path_or_address, name, pragma, output: output_location, etherscan, json } = self;
        let source = if Path::new(&path_or_address).exists() {
            AbiPath::Local { path: path_or_address, name }
        } else {
            let config = Config::from(&etherscan);
            let chain = config.chain.unwrap_or_default();
            let api_key = config.get_etherscan_api_key(Some(chain)).unwrap_or_default();
            AbiPath::Etherscan {
                chain,
                api_key,
                address: path_or_address.parse().wrap_err("invalid path or address")?,
            }
        };

        let items = match source {
            AbiPath::Local { path, name } => {
                let file = std::fs::read_to_string(&path).wrap_err("unable to read abi file")?;
                let obj: ContractObject = serde_json::from_str(&file)?;
                let abi =
                    obj.abi.ok_or_else(|| eyre::eyre!("could not find ABI in file {path}"))?;
                let name = name.unwrap_or_else(|| "Interface".to_owned());
                vec![(abi, name)]
            }
            AbiPath::Etherscan { address, chain, api_key } => {
                let client = Client::new(chain, api_key)?;
                let source = client.contract_source_code(address).await?;
                source
                    .items
                    .into_iter()
                    .map(|item| Ok((item.abi()?, item.contract_name)))
                    .collect::<Result<Vec<_>>>()?
            }
        };

        let interfaces = items
            .into_iter()
            .map(|(contract_abi, name)| {
                let source = match foundry_cli::utils::abi_to_solidity(&contract_abi, &name) {
                    Ok(generated_source) => generated_source,
                    Err(e) => {
                        warn!("Failed to format interface for {name}: {e}");
                        contract_abi.to_sol(&name, None)
                    }
                };
                Ok(InterfaceSource {
                    json_abi: serde_json::to_string_pretty(&contract_abi)?,
                    source,
                })
            })
            .collect::<Result<Vec<InterfaceSource>>>()?;

        // put it all together
        let res = if json {
            interfaces.iter().map(|iface| &iface.json_abi).format("\n").to_string()
        } else {
            format!(
                "// SPDX-License-Identifier: UNLICENSED\n\
                 pragma solidity {pragma};\n\n\
                 {}",
                interfaces.iter().map(|iface| &iface.source).format("\n")
            )
        };

        // print or write to file
        if let Some(loc) = output_location {
            if let Some(parent) = loc.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&loc, res)?;
            println!("Saved interface at {}", loc.display());
        } else {
            print!("{res}");
        }
        Ok(())
    }
}

struct InterfaceSource {
    json_abi: String,
    source: String,
}

// Local is a path to the directory containing the ABI files
// In case of etherscan, ABI is fetched from the address on the chain
enum AbiPath {
    Local { path: String, name: Option<String> },
    Etherscan { address: Address, chain: Chain, api_key: String },
}
