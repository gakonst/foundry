use clap::{Parser, ValueHint};
use eyre::Result;
use foundry_cli::opts::EtherscanOpts;
use std::path::PathBuf;

const DEFAULT_CRATE_NAME: &str = "foundry-contracts";
const DEFAULT_CRATE_VERSION: &str = "0.0.1";

/// CLI arguments for `cast bind`.
#[derive(Clone, Debug, Parser)]
pub struct BindArgs {
    /// The contract address, or the path to an ABI Directory
    ///
    /// If an address is specified, then the ABI is fetched from Etherscan.
    path_or_address: String,

    /// Path to where bindings will be stored
    #[arg(
        short,
        long,
        value_hint = ValueHint::DirPath,
        value_name = "PATH"
    )]
    pub output_dir: Option<PathBuf>,

    /// The name of the Rust crate to generate.
    ///
    /// This should be a valid crates.io crate name. However, this is currently not validated by
    /// this command.
    #[arg(
        long,
        default_value = DEFAULT_CRATE_NAME,
        value_name = "NAME"
    )]
    crate_name: String,

    /// The version of the Rust crate to generate.
    ///
    /// This should be a standard semver version string. However, it is not currently validated by
    /// this command.
    #[arg(
        long,
        default_value = DEFAULT_CRATE_VERSION,
        value_name = "VERSION"
    )]
    crate_version: String,

    /// Generate bindings as separate files.
    #[arg(long)]
    separate_files: bool,

    #[command(flatten)]
    etherscan: EtherscanOpts,
}

impl BindArgs {
    pub async fn run(self) -> Result<()> {
        Err(eyre::eyre!(
            "`cast bind` has been removed.\n\
             Please use `cast etherscan-source` to create a Forge project from an Etherscan source\n\
             and `forge bind` to generate the bindings to it instead."
        ))
    }
}
