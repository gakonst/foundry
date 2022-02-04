use crate::cmd::Cmd;

use clap::{Parser, ValueHint};
use ethers::contract::MultiAbigen;
use foundry_config::{
    figment::{
        self,
        error::Kind::InvalidType,
        value::{Dict, Map, Value},
        Metadata, Profile, Provider,
    },
    impl_figment_convert, Config,
};
use serde::Serialize;
use std::{fs, path::PathBuf};

impl_figment_convert!(BindArgs);

static DEFAULT_CRATE_NAME: &str = "foundry-contracts";
static DEFAULT_CRATE_VERSION: &str = "0.0.1";

#[derive(Debug, Clone, Parser, Serialize)]
pub struct BindArgs {
    #[clap(
        help = "The project's root path. By default, this is the root directory of the current Git repository or the current working directory if it is not part of a Git repository",
        long,
        value_hint = ValueHint::DirPath
    )]
    #[serde(skip)]
    pub root: Option<PathBuf>,

    #[clap(
        help = "Path to where the contract artifacts are stored",
        long = "out",
        short,
        value_hint = ValueHint::DirPath
    )]
    #[serde(rename = "out", skip_serializing_if = "Option::is_none")]
    pub out_path: Option<PathBuf>,

    #[clap(
        help = "A directory at which to write the generated rust crate",
        long = "bindings-root",
        value_hint = ValueHint::DirPath
    )]
    #[serde(skip)]
    pub bindings_root: Option<PathBuf>,

    #[clap(
        long = "crate-name",
        help = "The name of the rust crate to generate. This should be a valid crates.io crate name. However, it is not currently validated by this command.",
        default_value = DEFAULT_CRATE_NAME,
    )]
    #[serde(skip)]
    crate_name: String,

    #[clap(
        long = "crate-version",
        help = "The version of the rust crate to generate. This should be a standard semver version string. However, it is not currently validated by this command.",
        default_value = DEFAULT_CRATE_VERSION,
    )]
    #[serde(skip)]
    crate_version: String,

    #[clap(
        long = "overwrite",
        help = "Overwrite existing generated bindings. If set to false, the command will check that the bindings are correct, and then exit. If set to true, it will instead delete and overwrite the bindings."
    )]
    #[serde(skip)]
    overwrite: bool,

    #[clap(long = "single-file", help = "Generate bindings as a single file.")]
    #[serde(skip)]
    single_file: bool,
}

impl BindArgs {
    /// Get the path to the foundry artifacts directory
    fn artifacts(&self) -> PathBuf {
        let c: Config = self.into();
        c.out
    }

    /// Get the path to the root of the autogenerated crate
    fn bindings_root(&self) -> PathBuf {
        self.bindings_root.clone().unwrap_or_else(|| self.artifacts().join("bindings"))
    }

    /// `true` if the arguments set `crate_version` or `crate_name`
    fn gen_crate(&self) -> bool {
        self.crate_name != DEFAULT_CRATE_NAME || self.crate_version != DEFAULT_CRATE_VERSION
    }

    /// `true` if the bindings root already exists
    fn bindings_exist(&self) -> bool {
        self.bindings_root().is_dir()
    }

    /// Instantiate the multi-abigen
    fn get_multi(&self) -> eyre::Result<MultiAbigen> {
        MultiAbigen::from_json_files(self.artifacts())
    }

    /// Check that the existing bindings match the expected abigen output
    fn check_existing_bindings(&self) -> eyre::Result<()> {
        let bindings = self.get_multi()?.build()?;
        println!("Checking bindings for {} contracts", bindings.len());
        if self.gen_crate() {
            bindings.ensure_consistent_crate(
                &self.crate_name,
                &self.crate_version,
                self.bindings_root(),
                self.single_file,
            )?;
        } else {
            bindings.ensure_consistent_module(self.bindings_root(), self.single_file)?;
        }
        Ok(())
    }

    /// Generate the bindings
    fn generate_bindings(&self) -> eyre::Result<()> {
        let bindings = self.get_multi()?.build()?;
        println!("Generating bindings for {} contracts", bindings.len());
        if self.gen_crate() {
            bindings.write_to_crate(
                &self.crate_name,
                &self.crate_version,
                self.bindings_root(),
                self.single_file,
            )?;
        } else {
            bindings.write_to_module(self.bindings_root(), self.single_file)?;
        }
        Ok(())
    }
}

impl Cmd for BindArgs {
    type Output = ();

    fn run(self) -> eyre::Result<Self::Output> {
        if !self.overwrite && self.bindings_exist() {
            println!("Bindings found. Checking for consistency.");
            return self.check_existing_bindings()
        }

        if self.overwrite {
            fs::remove_dir_all(self.bindings_root())?;
        }

        self.generate_bindings()?;

        println!("Bindings have been output to {}", self.bindings_root().to_str().unwrap());
        Ok(())
    }
}

// Make this args a `figment::Provider` so that it can be merged into the `Config`
impl Provider for BindArgs {
    fn metadata(&self) -> Metadata {
        Metadata::named("Bind Args Provider")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        let value = Value::serialize(self)?;
        let error = InvalidType(value.to_actual(), "map".into());
        let dict = value.into_dict().ok_or(error)?;
        Ok(Map::from([(Config::selected_profile(), dict)]))
    }
}
