//! Foundry configuration.

#![warn(missing_docs, unused_crate_dependencies)]

#[macro_use]
extern crate tracing;

use crate::cache::StorageCachingConfig;
use alloy_primitives::{address, Address, B256, U256};
use eyre::{ContextCompat, WrapErr};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    value::{Dict, Map, Value},
    Error, Figment, Metadata, Profile, Provider,
};
use foundry_compilers::{
    artifacts::{
        output_selection::ContractOutputSelection, serde_helpers, BytecodeHash, DebuggingSettings,
        Libraries, ModelCheckerSettings, ModelCheckerTarget, Optimizer, OptimizerDetails,
        RevertStrings, Settings, SettingsMetadata, Severity,
    },
    cache::SOLIDITY_FILES_CACHE_FILENAME,
    error::SolcError,
    remappings::{RelativeRemapping, Remapping},
    ConfigurableArtifacts, EvmVersion, Project, ProjectPathsConfig, Solc, SolcConfig,
};
use inflector::Inflector;
use once_cell::sync::Lazy;
use regex::Regex;
use revm_primitives::SpecId;
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

// Macros useful for creating a figment.
mod macros;

// Utilities for making it easier to handle tests.
pub mod utils;
pub use crate::utils::*;

mod endpoints;
pub use endpoints::{ResolvedRpcEndpoints, RpcEndpoint, RpcEndpoints};

mod etherscan;
mod resolve;
pub use resolve::UnresolvedEnvVarError;

pub mod cache;
use cache::{Cache, ChainCache};

pub mod fmt;
pub use fmt::FormatterConfig;

pub mod fs_permissions;
pub use crate::fs_permissions::FsPermissions;

pub mod error;
pub use error::SolidityErrorCode;

pub mod doc;
pub use doc::DocConfig;

mod warning;
pub use warning::*;

// helpers for fixing configuration warnings
pub mod fix;

// reexport so cli types can implement `figment::Provider` to easily merge compiler arguments
pub use alloy_chains::{Chain, NamedChain};
pub use figment;

/// config providers
pub mod providers;

use crate::{
    error::ExtractConfigError,
    etherscan::{EtherscanConfigError, EtherscanConfigs, ResolvedEtherscanConfig},
};
use providers::*;

mod fuzz;
pub use fuzz::{FuzzConfig, FuzzDictionaryConfig};

mod invariant;
use crate::fs_permissions::PathPermission;
pub use invariant::InvariantConfig;
use providers::remappings::RemappingsProvider;

mod inline;
pub use inline::{validate_profiles, InlineConfig, InlineConfigError, InlineConfigParser, NatSpec};

/// Foundry configuration
///
/// # Defaults
///
/// All configuration values have a default, documented in the [fields](#fields)
/// section below. [`Config::default()`] returns the default values for
/// the default profile while [`Config::with_root()`] returns the values based on the given
/// directory. [`Config::load()`] starts with the default profile and merges various providers into
/// the config, same for [`Config::load_with_root()`], but there the default values are determined
/// by [`Config::with_root()`]
///
/// # Provider Details
///
/// `Config` is a Figment [`Provider`] with the following characteristics:
///
///   * **Profile**
///
///     The profile is set to the value of the `profile` field.
///
///   * **Metadata**
///
///     This provider is named `Foundry Config`. It does not specify a
///     [`Source`](figment::Source) and uses default interpolation.
///
///   * **Data**
///
///     The data emitted by this provider are the keys and values corresponding
///     to the fields and values of the structure. The dictionary is emitted to
///     the "default" meta-profile.
///
/// Note that these behaviors differ from those of [`Config::figment()`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The selected profile. **(default: _default_ `default`)**
    ///
    /// **Note:** This field is never serialized nor deserialized. When a
    /// `Config` is merged into a `Figment` as a `Provider`, this profile is
    /// selected on the `Figment`. When a `Config` is extracted, this field is
    /// set to the extracting Figment's selected `Profile`.
    #[serde(skip)]
    pub profile: Profile,
    /// path of the source contracts dir, like `src` or `contracts`
    pub src: PathBuf,
    /// path of the test dir
    pub test: PathBuf,
    /// path of the script dir
    pub script: PathBuf,
    /// path to where artifacts shut be written to
    pub out: PathBuf,
    /// all library folders to include, `lib`, `node_modules`
    pub libs: Vec<PathBuf>,
    /// `Remappings` to use for this repo
    pub remappings: Vec<RelativeRemapping>,
    /// Whether to autodetect remappings by scanning the `libs` folders recursively
    pub auto_detect_remappings: bool,
    /// library addresses to link
    pub libraries: Vec<String>,
    /// whether to enable cache
    pub cache: bool,
    /// where the cache is stored if enabled
    pub cache_path: PathBuf,
    /// where the broadcast logs are stored
    pub broadcast: PathBuf,
    /// additional solc allow paths for `--allow-paths`
    pub allow_paths: Vec<PathBuf>,
    /// additional solc include paths for `--include-path`
    pub include_paths: Vec<PathBuf>,
    /// whether to force a `project.clean()`
    pub force: bool,
    /// evm version to use
    #[serde(with = "from_str_lowercase")]
    pub evm_version: EvmVersion,
    /// list of contracts to report gas of
    pub gas_reports: Vec<String>,
    /// list of contracts to ignore for gas reports
    pub gas_reports_ignore: Vec<String>,
    /// The Solc instance to use if any.
    ///
    /// This takes precedence over `auto_detect_solc`, if a version is set then this overrides
    /// auto-detection.
    ///
    /// **Note** for backwards compatibility reasons this also accepts solc_version from the toml
    /// file, see [`BackwardsCompatProvider`]
    pub solc: Option<SolcReq>,
    /// whether to autodetect the solc compiler version to use
    pub auto_detect_solc: bool,
    /// Offline mode, if set, network access (downloading solc) is disallowed.
    ///
    /// Relationship with `auto_detect_solc`:
    ///    - if `auto_detect_solc = true` and `offline = true`, the required solc version(s) will
    ///      be auto detected but if the solc version is not installed, it will _not_ try to
    ///      install it
    pub offline: bool,
    /// Whether to activate optimizer
    pub optimizer: bool,
    /// Sets the optimizer runs
    pub optimizer_runs: usize,
    /// Switch optimizer components on or off in detail.
    /// The "enabled" switch above provides two defaults which can be
    /// tweaked here. If "details" is given, "enabled" can be omitted.
    pub optimizer_details: Option<OptimizerDetails>,
    /// Model checker settings.
    pub model_checker: Option<ModelCheckerSettings>,
    /// verbosity to use
    pub verbosity: u8,
    /// url of the rpc server that should be used for any rpc calls
    pub eth_rpc_url: Option<String>,
    /// JWT secret that should be used for any rpc calls
    pub eth_rpc_jwt: Option<String>,
    /// etherscan API key, or alias for an `EtherscanConfig` in `etherscan` table
    pub etherscan_api_key: Option<String>,
    /// Multiple etherscan api configs and their aliases
    #[serde(default, skip_serializing_if = "EtherscanConfigs::is_empty")]
    pub etherscan: EtherscanConfigs,
    /// list of solidity error codes to always silence in the compiler output
    pub ignored_error_codes: Vec<SolidityErrorCode>,
    /// When true, compiler warnings are treated as errors
    pub deny_warnings: bool,
    /// Only run test functions matching the specified regex pattern.
    #[serde(rename = "match_test")]
    pub test_pattern: Option<RegexWrapper>,
    /// Only run test functions that do not match the specified regex pattern.
    #[serde(rename = "no_match_test")]
    pub test_pattern_inverse: Option<RegexWrapper>,
    /// Only run tests in contracts matching the specified regex pattern.
    #[serde(rename = "match_contract")]
    pub contract_pattern: Option<RegexWrapper>,
    /// Only run tests in contracts that do not match the specified regex pattern.
    #[serde(rename = "no_match_contract")]
    pub contract_pattern_inverse: Option<RegexWrapper>,
    /// Only run tests in source files matching the specified glob pattern.
    #[serde(rename = "match_path", with = "from_opt_glob")]
    pub path_pattern: Option<globset::Glob>,
    /// Only run tests in source files that do not match the specified glob pattern.
    #[serde(rename = "no_match_path", with = "from_opt_glob")]
    pub path_pattern_inverse: Option<globset::Glob>,
    /// Configuration for fuzz testing
    pub fuzz: FuzzConfig,
    /// Configuration for invariant testing
    pub invariant: InvariantConfig,
    /// Whether to allow ffi cheatcodes in test
    pub ffi: bool,
    /// The address which will be executing all tests
    pub sender: Address,
    /// The tx.origin value during EVM execution
    pub tx_origin: Address,
    /// the initial balance of each deployed test contract
    pub initial_balance: U256,
    /// the block.number value during EVM execution
    pub block_number: u64,
    /// pins the block number for the state fork
    pub fork_block_number: Option<u64>,
    /// The chain name or EIP-155 chain ID.
    #[serde(rename = "chain_id", alias = "chain")]
    pub chain: Option<Chain>,
    /// Block gas limit.
    pub gas_limit: GasLimit,
    /// EIP-170: Contract code size limit in bytes. Useful to increase this because of tests.
    pub code_size_limit: Option<usize>,
    /// `tx.gasprice` value during EVM execution.
    ///
    /// This is an Option, so we can determine in fork mode whether to use the config's gas price
    /// (if set by user) or the remote client's gas price.
    pub gas_price: Option<u64>,
    /// The base fee in a block.
    pub block_base_fee_per_gas: u64,
    /// The `block.coinbase` value during EVM execution.
    pub block_coinbase: Address,
    /// The `block.timestamp` value during EVM execution.
    pub block_timestamp: u64,
    /// The `block.difficulty` value during EVM execution.
    pub block_difficulty: u64,
    /// Before merge the `block.max_hash`, after merge it is `block.prevrandao`.
    pub block_prevrandao: B256,
    /// the `block.gaslimit` value during EVM execution
    pub block_gas_limit: Option<GasLimit>,
    /// The memory limit per EVM execution in bytes.
    /// If this limit is exceeded, a `MemoryLimitOOG` result is thrown.
    ///
    /// The default is 128MiB.
    pub memory_limit: u64,
    /// Additional output selection for all contracts, such as "ir", "devdoc", "storageLayout",
    /// etc.
    ///
    /// See the [Solc Compiler Api](https://docs.soliditylang.org/en/latest/using-the-compiler.html#compiler-api) for more information.
    ///
    /// The following values are always set because they're required by `forge`:
    /// ```json
    /// {
    ///   "*": [
    ///       "abi",
    ///       "evm.bytecode",
    ///       "evm.deployedBytecode",
    ///       "evm.methodIdentifiers"
    ///     ]
    /// }
    /// ```
    #[serde(default)]
    pub extra_output: Vec<ContractOutputSelection>,
    /// If set, a separate JSON file will be emitted for every contract depending on the
    /// selection, eg. `extra_output_files = ["metadata"]` will create a `metadata.json` for
    /// each contract in the project.
    ///
    /// See [Contract Metadata](https://docs.soliditylang.org/en/latest/metadata.html) for more information.
    ///
    /// The difference between `extra_output = ["metadata"]` and
    /// `extra_output_files = ["metadata"]` is that the former will include the
    /// contract's metadata in the contract's json artifact, whereas the latter will emit the
    /// output selection as separate files.
    #[serde(default)]
    pub extra_output_files: Vec<ContractOutputSelection>,
    /// Whether to print the names of the compiled contracts.
    pub names: bool,
    /// Whether to print the sizes of the compiled contracts.
    pub sizes: bool,
    /// If set to true, changes compilation pipeline to go through the Yul intermediate
    /// representation.
    pub via_ir: bool,
    /// RPC storage caching settings determines what chains and endpoints to cache
    pub rpc_storage_caching: StorageCachingConfig,
    /// Disables storage caching entirely. This overrides any settings made in
    /// `rpc_storage_caching`
    pub no_storage_caching: bool,
    /// Disables rate limiting entirely. This overrides any settings made in
    /// `compute_units_per_second`
    pub no_rpc_rate_limit: bool,
    /// Multiple rpc endpoints and their aliases
    #[serde(default, skip_serializing_if = "RpcEndpoints::is_empty")]
    pub rpc_endpoints: RpcEndpoints,
    /// Whether to store the referenced sources in the metadata as literal data.
    pub use_literal_content: bool,
    /// Whether to include the metadata hash.
    ///
    /// The metadata hash is machine dependent. By default, this is set to [BytecodeHash::None] to allow for deterministic code, See: <https://docs.soliditylang.org/en/latest/metadata.html>
    #[serde(with = "from_str_lowercase")]
    pub bytecode_hash: BytecodeHash,
    /// Whether to append the metadata hash to the bytecode.
    ///
    /// If this is `false` and the `bytecode_hash` option above is not `None` solc will issue a
    /// warning.
    pub cbor_metadata: bool,
    /// How to treat revert (and require) reason strings.
    #[serde(with = "serde_helpers::display_from_str_opt")]
    pub revert_strings: Option<RevertStrings>,
    /// Whether to compile in sparse mode
    ///
    /// If this option is enabled, only the required contracts/files will be selected to be
    /// included in solc's output selection, see also
    /// [OutputSelection](foundry_compilers::artifacts::output_selection::OutputSelection)
    pub sparse_mode: bool,
    /// Whether to emit additional build info files
    ///
    /// If set to `true`, `ethers-solc` will generate additional build info json files for every
    /// new build, containing the `CompilerInput` and `CompilerOutput`
    pub build_info: bool,
    /// The path to the `build-info` directory that contains the build info json files.
    pub build_info_path: Option<PathBuf>,
    /// Configuration for `forge fmt`
    pub fmt: FormatterConfig,
    /// Configuration for `forge doc`
    pub doc: DocConfig,
    /// Configures the permissions of cheat codes that touch the file system.
    ///
    /// This includes what operations can be executed (read, write)
    pub fs_permissions: FsPermissions,

    /// Temporary config to enable [SpecId::CANCUN]
    ///
    /// <https://github.com/foundry-rs/foundry/issues/5782>
    /// Should be removed once EvmVersion Cancun is supported by solc
    pub cancun: bool,

    /// Address labels
    pub labels: HashMap<Address, String>,

    /// The root path where the config detection started from, `Config::with_root`
    #[doc(hidden)]
    //  We're skipping serialization here, so it won't be included in the [`Config::to_string()`]
    // representation, but will be deserialized from the `Figment` so that forge commands can
    // override it.
    #[serde(rename = "root", default, skip_serializing)]
    pub __root: RootPath,
    /// PRIVATE: This structure may grow, As such, constructing this structure should
    /// _always_ be done using a public constructor or update syntax:
    ///
    /// ```ignore
    /// use foundry_config::Config;
    ///
    /// let config = Config { src: "other".into(), ..Default::default() };
    /// ```
    #[doc(hidden)]
    #[serde(skip)]
    pub __non_exhaustive: (),
    /// Warnings gathered when loading the Config. See [`WarningsProvider`] for more information
    #[serde(default, skip_serializing)]
    pub __warnings: Vec<Warning>,
}

/// Mapping of fallback standalone sections. See [`FallbackProfileProvider`]
pub static STANDALONE_FALLBACK_SECTIONS: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| HashMap::from([("invariant", "fuzz")]));

/// Deprecated keys.
pub static DEPRECATIONS: Lazy<HashMap<String, String>> = Lazy::new(|| HashMap::from([]));

impl Config {
    /// The default profile: "default"
    pub const DEFAULT_PROFILE: Profile = Profile::const_new("default");

    /// The hardhat profile: "hardhat"
    pub const HARDHAT_PROFILE: Profile = Profile::const_new("hardhat");

    /// TOML section for profiles
    pub const PROFILE_SECTION: &'static str = "profile";

    /// Standalone sections in the config which get integrated into the selected profile
    pub const STANDALONE_SECTIONS: &'static [&'static str] =
        &["rpc_endpoints", "etherscan", "fmt", "doc", "fuzz", "invariant", "labels"];

    /// File name of config toml file
    pub const FILE_NAME: &'static str = "foundry.toml";

    /// The name of the directory foundry reserves for itself under the user's home directory: `~`
    pub const FOUNDRY_DIR_NAME: &'static str = ".foundry";

    /// Default address for tx.origin
    ///
    /// `0x1804c8AB1F12E6bbf3894d4083f33e07309d1f38`
    pub const DEFAULT_SENDER: Address = address!("1804c8AB1F12E6bbf3894d4083f33e07309d1f38");

    /// Returns the current `Config`
    ///
    /// See `Config::figment`
    #[track_caller]
    pub fn load() -> Self {
        Config::from_provider(Config::figment())
    }

    /// Returns the current `Config`
    ///
    /// See `Config::figment_with_root`
    #[track_caller]
    pub fn load_with_root(root: impl Into<PathBuf>) -> Self {
        Config::from_provider(Config::figment_with_root(root))
    }

    /// Extract a `Config` from `provider`, panicking if extraction fails.
    ///
    /// # Panics
    ///
    /// If extraction fails, prints an error message indicating the failure and
    /// panics. For a version that doesn't panic, use [`Config::try_from()`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use figment::providers::{Env, Format, Toml};
    /// use foundry_config::Config;
    ///
    /// // Use foundry's default `Figment`, but allow values from `other.toml`
    /// // to supersede its values.
    /// let figment = Config::figment().merge(Toml::file("other.toml").nested());
    ///
    /// let config = Config::from_provider(figment);
    /// ```
    #[track_caller]
    pub fn from_provider<T: Provider>(provider: T) -> Self {
        trace!("load config with provider: {:?}", provider.metadata());
        Self::try_from(provider).unwrap_or_else(|err| panic!("{}", err))
    }

    /// Attempts to extract a `Config` from `provider`, returning the result.
    ///
    /// # Example
    ///
    /// ```rust
    /// use figment::providers::{Env, Format, Toml};
    /// use foundry_config::Config;
    ///
    /// // Use foundry's default `Figment`, but allow values from `other.toml`
    /// // to supersede its values.
    /// let figment = Config::figment().merge(Toml::file("other.toml").nested());
    ///
    /// let config = Config::try_from(figment);
    /// ```
    pub fn try_from<T: Provider>(provider: T) -> Result<Self, ExtractConfigError> {
        let figment = Figment::from(provider);
        let mut config = figment.extract::<Self>().map_err(ExtractConfigError::new)?;
        config.profile = figment.profile().clone();
        Ok(config)
    }

    /// The config supports relative paths and tracks the root path separately see
    /// `Config::with_root`
    ///
    /// This joins all relative paths with the current root and attempts to make them canonic
    #[must_use]
    pub fn canonic(self) -> Self {
        let root = self.__root.0.clone();
        self.canonic_at(root)
    }

    /// Joins all relative paths with the given root so that paths that are defined as:
    ///
    /// ```toml
    /// [profile.default]
    /// src = "src"
    /// out = "./out"
    /// libs = ["lib", "/var/lib"]
    /// ```
    ///
    /// Will be made canonic with the given root:
    ///
    /// ```toml
    /// [profile.default]
    /// src = "<root>/src"
    /// out = "<root>/out"
    /// libs = ["<root>/lib", "/var/lib"]
    /// ```
    #[must_use]
    pub fn canonic_at(mut self, root: impl Into<PathBuf>) -> Self {
        let root = canonic(root);

        fn p(root: &Path, rem: &Path) -> PathBuf {
            canonic(root.join(rem))
        }

        self.src = p(&root, &self.src);
        self.test = p(&root, &self.test);
        self.script = p(&root, &self.script);
        self.out = p(&root, &self.out);
        self.broadcast = p(&root, &self.broadcast);
        self.cache_path = p(&root, &self.cache_path);

        if let Some(build_info_path) = self.build_info_path {
            self.build_info_path = Some(p(&root, &build_info_path));
        }

        self.libs = self.libs.into_iter().map(|lib| p(&root, &lib)).collect();

        self.remappings =
            self.remappings.into_iter().map(|r| RelativeRemapping::new(r.into(), &root)).collect();

        self.allow_paths = self.allow_paths.into_iter().map(|allow| p(&root, &allow)).collect();

        self.include_paths = self.include_paths.into_iter().map(|allow| p(&root, &allow)).collect();

        self.fs_permissions.join_all(&root);

        if let Some(ref mut model_checker) = self.model_checker {
            model_checker.contracts = std::mem::take(&mut model_checker.contracts)
                .into_iter()
                .map(|(path, contracts)| {
                    (format!("{}", p(&root, path.as_ref()).display()), contracts)
                })
                .collect();
        }

        self
    }

    /// Returns a sanitized version of the Config where are paths are set correctly and potential
    /// duplicates are resolved
    ///
    /// See [`Self::canonic`]
    #[must_use]
    pub fn sanitized(self) -> Self {
        let mut config = self.canonic();

        config.sanitize_remappings();

        config.libs.sort_unstable();
        config.libs.dedup();

        config
    }

    /// Cleans up any duplicate `Remapping` and sorts them
    ///
    /// On windows this will convert any `\` in the remapping path into a `/`
    pub fn sanitize_remappings(&mut self) {
        #[cfg(target_os = "windows")]
        {
            // force `/` in remappings on windows
            use path_slash::PathBufExt;
            self.remappings.iter_mut().for_each(|r| {
                r.path.path = r.path.path.to_slash_lossy().into_owned().into();
            });
        }
    }

    /// Returns the directory in which dependencies should be installed
    ///
    /// Returns the first dir from `libs` that is not `node_modules` or `lib` if `libs` is empty
    pub fn install_lib_dir(&self) -> &Path {
        self.libs
            .iter()
            .find(|p| !p.ends_with("node_modules"))
            .map(|p| p.as_path())
            .unwrap_or_else(|| Path::new("lib"))
    }

    /// Serves as the entrypoint for obtaining the project.
    ///
    /// Returns the `Project` configured with all `solc` and path related values.
    ///
    /// *Note*: this also _cleans_ [`Project::cleanup`] the workspace if `force` is set to true.
    ///
    /// # Example
    ///
    /// ```
    /// use foundry_config::Config;
    /// let config = Config::load_with_root(".").sanitized();
    /// let project = config.project();
    /// ```
    pub fn project(&self) -> Result<Project, SolcError> {
        self.create_project(self.cache, false)
    }

    /// Same as [`Self::project()`] but sets configures the project to not emit artifacts and ignore
    /// cache, caching causes no output until https://github.com/gakonst/ethers-rs/issues/727
    pub fn ephemeral_no_artifacts_project(&self) -> Result<Project, SolcError> {
        self.create_project(false, true)
    }

    fn create_project(&self, cached: bool, no_artifacts: bool) -> Result<Project, SolcError> {
        let mut project = Project::builder()
            .artifacts(self.configured_artifacts_handler())
            .paths(self.project_paths())
            .allowed_path(&self.__root.0)
            .allowed_paths(&self.libs)
            .allowed_paths(&self.allow_paths)
            .include_paths(&self.include_paths)
            .solc_config(SolcConfig::builder().settings(self.solc_settings()?).build())
            .ignore_error_codes(self.ignored_error_codes.iter().copied().map(Into::into))
            .set_compiler_severity_filter(if self.deny_warnings {
                Severity::Warning
            } else {
                Severity::Error
            })
            .set_auto_detect(self.is_auto_detect())
            .set_offline(self.offline)
            .set_cached(cached)
            .set_build_info(cached & self.build_info)
            .set_no_artifacts(no_artifacts)
            .build()?;

        if self.force {
            project.cleanup()?;
        }

        if let Some(solc) = self.ensure_solc()? {
            project.solc = solc;
        }

        Ok(project)
    }

    /// Ensures that the configured version is installed if explicitly set
    ///
    /// If `solc` is [`SolcReq::Version`] then this will download and install the solc version if
    /// it's missing, unless the `offline` flag is enabled, in which case an error is thrown.
    ///
    /// If `solc` is [`SolcReq::Local`] then this will ensure that the path exists.
    fn ensure_solc(&self) -> Result<Option<Solc>, SolcError> {
        if let Some(ref solc) = self.solc {
            let solc = match solc {
                SolcReq::Version(version) => {
                    let v = version.to_string();
                    let mut solc = Solc::find_svm_installed_version(&v)?;
                    if solc.is_none() {
                        if self.offline {
                            return Err(SolcError::msg(format!(
                                "can't install missing solc {version} in offline mode"
                            )))
                        }
                        Solc::blocking_install(version)?;
                        solc = Solc::find_svm_installed_version(&v)?;
                    }
                    solc
                }
                SolcReq::Local(solc) => {
                    if !solc.is_file() {
                        return Err(SolcError::msg(format!(
                            "`solc` {} does not exist",
                            solc.display()
                        )))
                    }
                    Some(Solc::new(solc))
                }
            };
            return Ok(solc)
        }

        Ok(None)
    }

    /// Returns the [SpecId] derived from the configured [EvmVersion]
    #[inline]
    pub fn evm_spec_id(&self) -> SpecId {
        if self.cancun {
            return SpecId::CANCUN
        }
        evm_spec_id(&self.evm_version)
    }

    /// Returns whether the compiler version should be auto-detected
    ///
    /// Returns `false` if `solc_version` is explicitly set, otherwise returns the value of
    /// `auto_detect_solc`
    pub fn is_auto_detect(&self) -> bool {
        if self.solc.is_some() {
            return false
        }
        self.auto_detect_solc
    }

    /// Whether caching should be enabled for the given chain id
    pub fn enable_caching(&self, endpoint: &str, chain_id: impl Into<u64>) -> bool {
        !self.no_storage_caching &&
            self.rpc_storage_caching.enable_for_chain_id(chain_id.into()) &&
            self.rpc_storage_caching.enable_for_endpoint(endpoint)
    }

    /// Returns the `ProjectPathsConfig`  sub set of the config.
    ///
    /// **NOTE**: this uses the paths as they are and does __not__ modify them, see
    /// `[Self::sanitized]`
    ///
    /// # Example
    ///
    /// ```
    /// use foundry_config::Config;
    /// let config = Config::load_with_root(".").sanitized();
    /// let paths = config.project_paths();
    /// ```
    pub fn project_paths(&self) -> ProjectPathsConfig {
        let mut builder = ProjectPathsConfig::builder()
            .cache(self.cache_path.join(SOLIDITY_FILES_CACHE_FILENAME))
            .sources(&self.src)
            .tests(&self.test)
            .scripts(&self.script)
            .artifacts(&self.out)
            .libs(self.libs.clone())
            .remappings(self.get_all_remappings());

        if let Some(build_info_path) = &self.build_info_path {
            builder = builder.build_infos(build_info_path);
        }

        builder.build_with_root(&self.__root.0)
    }

    /// Returns all configured [`Remappings`]
    ///
    /// **Note:** this will add an additional `<src>/=<src path>` remapping here, see
    /// [Self::get_source_dir_remapping()]
    ///
    /// So that
    ///
    /// ```solidity
    /// import "./math/math.sol";
    /// import "contracts/tokens/token.sol";
    /// ```
    ///
    /// in `contracts/contract.sol` are resolved to
    ///
    /// ```text
    /// contracts/tokens/token.sol
    /// contracts/math/math.sol
    /// ```
    pub fn get_all_remappings(&self) -> Vec<Remapping> {
        self.remappings.iter().map(|m| m.clone().into()).collect()
    }

    /// Returns the configured rpc jwt secret
    ///
    /// Returns:
    ///    - The jwt secret, if configured
    ///
    /// # Example
    ///
    /// ```
    /// use foundry_config::Config;
    /// # fn t() {
    /// let config = Config::with_root("./");
    /// let rpc_jwt = config.get_rpc_jwt_secret().unwrap().unwrap();
    /// # }
    /// ```
    pub fn get_rpc_jwt_secret(&self) -> Result<Option<Cow<str>>, UnresolvedEnvVarError> {
        Ok(self.eth_rpc_jwt.as_ref().map(|jwt| Cow::Borrowed(jwt.as_str())))
    }

    /// Returns the configured rpc url
    ///
    /// Returns:
    ///    - the matching, resolved url of  `rpc_endpoints` if `eth_rpc_url` is an alias
    ///    - the `eth_rpc_url` as-is if it isn't an alias
    ///
    /// # Example
    ///
    /// ```
    /// use foundry_config::Config;
    /// # fn t() {
    /// let config = Config::with_root("./");
    /// let rpc_url = config.get_rpc_url().unwrap().unwrap();
    /// # }
    /// ```
    pub fn get_rpc_url(&self) -> Option<Result<Cow<str>, UnresolvedEnvVarError>> {
        let maybe_alias = self.eth_rpc_url.as_ref().or(self.etherscan_api_key.as_ref())?;
        if let Some(alias) = self.get_rpc_url_with_alias(maybe_alias) {
            Some(alias)
        } else {
            Some(Ok(Cow::Borrowed(self.eth_rpc_url.as_deref()?)))
        }
    }

    /// Resolves the given alias to a matching rpc url
    ///
    /// Returns:
    ///    - the matching, resolved url of  `rpc_endpoints` if `maybe_alias` is an alias
    ///    - None otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use foundry_config::Config;
    /// # fn t() {
    /// let config = Config::with_root("./");
    /// let rpc_url = config.get_rpc_url_with_alias("mainnet").unwrap().unwrap();
    /// # }
    /// ```
    pub fn get_rpc_url_with_alias(
        &self,
        maybe_alias: &str,
    ) -> Option<Result<Cow<str>, UnresolvedEnvVarError>> {
        let mut endpoints = self.rpc_endpoints.clone().resolved();
        Some(endpoints.remove(maybe_alias)?.map(Cow::Owned))
    }

    /// Returns the configured rpc, or the fallback url
    ///
    /// # Example
    ///
    /// ```
    /// use foundry_config::Config;
    /// # fn t() {
    /// let config = Config::with_root("./");
    /// let rpc_url = config.get_rpc_url_or("http://localhost:8545").unwrap();
    /// # }
    /// ```
    pub fn get_rpc_url_or<'a>(
        &'a self,
        fallback: impl Into<Cow<'a, str>>,
    ) -> Result<Cow<str>, UnresolvedEnvVarError> {
        if let Some(url) = self.get_rpc_url() {
            url
        } else {
            Ok(fallback.into())
        }
    }

    /// Returns the configured rpc or `"http://localhost:8545"` if no `eth_rpc_url` is set
    ///
    /// # Example
    ///
    /// ```
    /// use foundry_config::Config;
    /// # fn t() {
    /// let config = Config::with_root("./");
    /// let rpc_url = config.get_rpc_url_or_localhost_http().unwrap();
    /// # }
    /// ```
    pub fn get_rpc_url_or_localhost_http(&self) -> Result<Cow<str>, UnresolvedEnvVarError> {
        self.get_rpc_url_or("http://localhost:8545")
    }

    /// Returns the `EtherscanConfig` to use, if any
    ///
    /// Returns
    ///  - the matching `ResolvedEtherscanConfig` of the `etherscan` table if `etherscan_api_key` is
    ///    an alias
    ///  - the Mainnet  `ResolvedEtherscanConfig` if `etherscan_api_key` is set, `None` otherwise
    ///
    /// # Example
    ///
    /// ```
    /// use foundry_config::Config;
    /// # fn t() {
    /// let config = Config::with_root("./");
    /// let etherscan_config = config.get_etherscan_config().unwrap().unwrap();
    /// let client = etherscan_config.into_client().unwrap();
    /// # }
    /// ```
    pub fn get_etherscan_config(
        &self,
    ) -> Option<Result<ResolvedEtherscanConfig, EtherscanConfigError>> {
        let maybe_alias = self.etherscan_api_key.as_ref().or(self.eth_rpc_url.as_ref())?;
        if self.etherscan.contains_key(maybe_alias) {
            // etherscan points to an alias in the `etherscan` table, so we try to resolve that
            let mut resolved = self.etherscan.clone().resolved();
            return resolved.remove(maybe_alias)
        }

        // we treat the `etherscan_api_key` as actual API key
        // if no chain provided, we assume mainnet
        let chain = self.chain.unwrap_or(Chain::mainnet());
        let api_key = self.etherscan_api_key.as_ref()?;
        ResolvedEtherscanConfig::create(api_key, chain).map(Ok)
    }

    /// Same as [`Self::get_etherscan_config()`] but optionally updates the config with the given
    /// `chain`, and `etherscan_api_key`
    ///
    /// If not matching alias was found, then this will try to find the first entry in the table
    /// with a matching chain id. If an etherscan_api_key is already set it will take precedence
    /// over the chain's entry in the table.
    pub fn get_etherscan_config_with_chain(
        &self,
        chain: Option<Chain>,
    ) -> Result<Option<ResolvedEtherscanConfig>, EtherscanConfigError> {
        if let Some(maybe_alias) = self.etherscan_api_key.as_ref().or(self.eth_rpc_url.as_ref()) {
            if self.etherscan.contains_key(maybe_alias) {
                return self.etherscan.clone().resolved().remove(maybe_alias).transpose()
            }
        }

        // try to find by comparing chain IDs after resolving
        if let Some(res) =
            chain.and_then(|chain| self.etherscan.clone().resolved().find_chain(chain))
        {
            match (res, self.etherscan_api_key.as_ref()) {
                (Ok(mut config), Some(key)) => {
                    // we update the key, because if an etherscan_api_key is set, it should take
                    // precedence over the entry, since this is usually set via env var or CLI args.
                    config.key = key.clone();
                    return Ok(Some(config))
                }
                (Ok(config), None) => return Ok(Some(config)),
                (Err(err), None) => return Err(err),
                (Err(_), Some(_)) => {
                    // use the etherscan key as fallback
                }
            }
        }

        // etherscan fallback via API key
        if let Some(key) = self.etherscan_api_key.as_ref() {
            let chain = chain.or(self.chain).unwrap_or_default();
            return Ok(ResolvedEtherscanConfig::create(key, chain))
        }

        Ok(None)
    }

    /// Helper function to just get the API key
    pub fn get_etherscan_api_key(&self, chain: Option<Chain>) -> Option<String> {
        self.get_etherscan_config_with_chain(chain).ok().flatten().map(|c| c.key)
    }

    /// Returns the remapping for the project's _src_ directory
    ///
    /// **Note:** this will add an additional `<src>/=<src path>` remapping here so imports that
    /// look like `import {Foo} from "src/Foo.sol";` are properly resolved.
    ///
    /// This is due the fact that `solc`'s VFS resolves [direct imports](https://docs.soliditylang.org/en/develop/path-resolution.html#direct-imports) that start with the source directory's name.
    pub fn get_source_dir_remapping(&self) -> Option<Remapping> {
        get_dir_remapping(&self.src)
    }

    /// Returns the remapping for the project's _test_ directory, but only if it exists
    pub fn get_test_dir_remapping(&self) -> Option<Remapping> {
        if self.__root.0.join(&self.test).exists() {
            get_dir_remapping(&self.test)
        } else {
            None
        }
    }

    /// Returns the remapping for the project's _script_ directory, but only if it exists
    pub fn get_script_dir_remapping(&self) -> Option<Remapping> {
        if self.__root.0.join(&self.script).exists() {
            get_dir_remapping(&self.script)
        } else {
            None
        }
    }

    /// Returns the `Optimizer` based on the configured settings
    pub fn optimizer(&self) -> Optimizer {
        // only configure optimizer settings if optimizer is enabled
        let details = if self.optimizer { self.optimizer_details.clone() } else { None };

        Optimizer { enabled: Some(self.optimizer), runs: Some(self.optimizer_runs), details }
    }

    /// returns the [`foundry_compilers::ConfigurableArtifacts`] for this config, that includes the
    /// `extra_output` fields
    pub fn configured_artifacts_handler(&self) -> ConfigurableArtifacts {
        let mut extra_output = self.extra_output.clone();
        // Sourcify verification requires solc metadata output. Since, it doesn't
        // affect the UX & performance of the compiler, output the metadata files
        // by default.
        // For more info see: <https://github.com/foundry-rs/foundry/issues/2795>
        // Metadata is not emitted as separate file because this breaks typechain support: <https://github.com/foundry-rs/foundry/issues/2969>
        if !extra_output.contains(&ContractOutputSelection::Metadata) {
            extra_output.push(ContractOutputSelection::Metadata);
        }

        ConfigurableArtifacts::new(extra_output, self.extra_output_files.clone())
    }

    /// Parses all libraries in the form of
    /// `<file>:<lib>:<addr>`
    pub fn parsed_libraries(&self) -> Result<Libraries, SolcError> {
        Libraries::parse(&self.libraries)
    }

    /// Returns the configured `solc` `Settings` that includes:
    ///   - all libraries
    ///   - the optimizer (including details, if configured)
    ///   - evm version
    pub fn solc_settings(&self) -> Result<Settings, SolcError> {
        let libraries = self.parsed_libraries()?.with_applied_remappings(&self.project_paths());
        let optimizer = self.optimizer();

        // By default if no targets are specifically selected the model checker uses all targets.
        // This might be too much here, so only enable assertion checks.
        // If users wish to enable all options they need to do so explicitly.
        let mut model_checker = self.model_checker.clone();
        if let Some(ref mut model_checker_settings) = model_checker {
            if model_checker_settings.targets.is_none() {
                model_checker_settings.targets = Some(vec![ModelCheckerTarget::Assert]);
            }
        }

        let mut settings = Settings {
            optimizer,
            evm_version: Some(self.evm_version),
            libraries,
            metadata: Some(SettingsMetadata {
                use_literal_content: Some(self.use_literal_content),
                bytecode_hash: Some(self.bytecode_hash),
                cbor_metadata: Some(self.cbor_metadata),
            }),
            debug: self.revert_strings.map(|revert_strings| DebuggingSettings {
                revert_strings: Some(revert_strings),
                debug_info: Vec::new(),
            }),
            model_checker,
            ..Default::default()
        }
        .with_extra_output(self.configured_artifacts_handler().output_selection())
        .with_ast();

        if self.via_ir {
            settings = settings.with_via_ir();
        }

        Ok(settings)
    }

    /// Returns the default figment
    ///
    /// The default figment reads from the following sources, in ascending
    /// priority order:
    ///
    ///   1. [`Config::default()`] (see [defaults](#defaults))
    ///   2. `foundry.toml` _or_ filename in `FOUNDRY_CONFIG` environment variable
    ///   3. `FOUNDRY_` prefixed environment variables
    ///
    /// The profile selected is the value set in the `FOUNDRY_PROFILE`
    /// environment variable. If it is not set, it defaults to `default`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use foundry_config::Config;
    /// use serde::Deserialize;
    ///
    /// let my_config = Config::figment().extract::<Config>();
    /// ```
    pub fn figment() -> Figment {
        Config::default().into()
    }

    /// Returns the default figment enhanced with additional context extracted from the provided
    /// root, like remappings and directories.
    ///
    /// # Example
    ///
    /// ```rust
    /// use foundry_config::Config;
    /// use serde::Deserialize;
    ///
    /// let my_config = Config::figment_with_root(".").extract::<Config>();
    /// ```
    pub fn figment_with_root(root: impl Into<PathBuf>) -> Figment {
        Self::with_root(root).into()
    }

    /// Creates a new Config that adds additional context extracted from the provided root.
    ///
    /// # Example
    ///
    /// ```rust
    /// use foundry_config::Config;
    /// let my_config = Config::with_root(".");
    /// ```
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        // autodetect paths
        let root = root.into();
        let paths = ProjectPathsConfig::builder().build_with_root(&root);
        let artifacts: PathBuf = paths.artifacts.file_name().unwrap().into();
        Config {
            __root: paths.root.into(),
            src: paths.sources.file_name().unwrap().into(),
            out: artifacts.clone(),
            libs: paths.libraries.into_iter().map(|lib| lib.file_name().unwrap().into()).collect(),
            remappings: paths
                .remappings
                .into_iter()
                .map(|r| RelativeRemapping::new(r, &root))
                .collect(),
            fs_permissions: FsPermissions::new([PathPermission::read(artifacts)]),
            ..Config::default()
        }
    }

    /// Returns the default config but with hardhat paths
    pub fn hardhat() -> Self {
        Config {
            src: "contracts".into(),
            out: "artifacts".into(),
            libs: vec!["node_modules".into()],
            ..Config::default()
        }
    }

    /// Returns the default config that uses dapptools style paths
    pub fn dapptools() -> Self {
        Config {
            chain: Some(Chain::from_id(99)),
            block_timestamp: 0,
            block_number: 0,
            ..Config::default()
        }
    }

    /// Extracts a basic subset of the config, used for initialisations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use foundry_config::Config;
    /// let my_config = Config::with_root(".").into_basic();
    /// ```
    pub fn into_basic(self) -> BasicConfig {
        BasicConfig {
            profile: self.profile,
            src: self.src,
            out: self.out,
            libs: self.libs,
            remappings: self.remappings,
        }
    }

    /// Updates the `foundry.toml` file for the given `root` based on the provided closure.
    ///
    /// **Note:** the closure will only be invoked if the `foundry.toml` file exists, See
    /// [Self::get_config_path()] and if the closure returns `true`.
    pub fn update_at<F>(root: impl Into<PathBuf>, f: F) -> eyre::Result<()>
    where
        F: FnOnce(&Config, &mut toml_edit::Document) -> bool,
    {
        let config = Self::load_with_root(root).sanitized();
        config.update(|doc| f(&config, doc))
    }

    /// Updates the `foundry.toml` file this `Config` ias based on with the provided closure.
    ///
    /// **Note:** the closure will only be invoked if the `foundry.toml` file exists, See
    /// [Self::get_config_path()] and if the closure returns `true`
    pub fn update<F>(&self, f: F) -> eyre::Result<()>
    where
        F: FnOnce(&mut toml_edit::Document) -> bool,
    {
        let file_path = self.get_config_path();
        if !file_path.exists() {
            return Ok(())
        }
        let contents = fs::read_to_string(&file_path)?;
        let mut doc = contents.parse::<toml_edit::Document>()?;
        if f(&mut doc) {
            fs::write(file_path, doc.to_string())?;
        }
        Ok(())
    }

    /// Sets the `libs` entry inside a `foundry.toml` file but only if it exists
    ///
    /// # Errors
    ///
    /// An error if the `foundry.toml` could not be parsed.
    pub fn update_libs(&self) -> eyre::Result<()> {
        self.update(|doc| {
            let profile = self.profile.as_str().as_str();
            let root = &self.__root.0;
            let libs: toml_edit::Value = self
                .libs
                .iter()
                .map(|path| {
                    let path =
                        if let Ok(relative) = path.strip_prefix(root) { relative } else { path };
                    toml_edit::Value::from(&*path.to_string_lossy())
                })
                .collect();
            let libs = toml_edit::value(libs);
            doc[Config::PROFILE_SECTION][profile]["libs"] = libs;
            true
        })
    }

    /// Serialize the config type as a String of TOML.
    ///
    /// This serializes to a table with the name of the profile
    ///
    /// ```toml
    /// [profile.default]
    /// src = "src"
    /// out = "out"
    /// libs = ["lib"]
    /// # ...
    /// ```
    pub fn to_string_pretty(&self) -> Result<String, toml::ser::Error> {
        // serializing to value first to prevent `ValueAfterTable` errors
        let mut value = toml::Value::try_from(self)?;
        // Config map always gets serialized as a table
        let value_table = value.as_table_mut().unwrap();
        // remove standalone sections from inner table
        let standalone_sections = Config::STANDALONE_SECTIONS
            .iter()
            .filter_map(|section| {
                let section = section.to_string();
                value_table.remove(&section).map(|value| (section, value))
            })
            .collect::<Vec<_>>();
        // wrap inner table in [profile.<profile>]
        let mut wrapping_table = [(
            Config::PROFILE_SECTION.into(),
            toml::Value::Table([(self.profile.to_string(), value)].into_iter().collect()),
        )]
        .into_iter()
        .collect::<toml::map::Map<_, _>>();
        // insert standalone sections
        for (section, value) in standalone_sections {
            wrapping_table.insert(section, value);
        }
        // stringify
        toml::to_string_pretty(&toml::Value::Table(wrapping_table))
    }

    /// Returns the path to the `foundry.toml`  of this `Config`
    pub fn get_config_path(&self) -> PathBuf {
        self.__root.0.join(Config::FILE_NAME)
    }

    /// Returns the selected profile
    ///
    /// If the `FOUNDRY_PROFILE` env variable is not set, this returns the `DEFAULT_PROFILE`
    pub fn selected_profile() -> Profile {
        Profile::from_env_or("FOUNDRY_PROFILE", Config::DEFAULT_PROFILE)
    }

    /// Returns the path to foundry's global toml file that's stored at `~/.foundry/foundry.toml`
    pub fn foundry_dir_toml() -> Option<PathBuf> {
        Self::foundry_dir().map(|p| p.join(Config::FILE_NAME))
    }

    /// Returns the path to foundry's config dir `~/.foundry/`
    pub fn foundry_dir() -> Option<PathBuf> {
        dirs_next::home_dir().map(|p| p.join(Config::FOUNDRY_DIR_NAME))
    }

    /// Returns the path to foundry's cache dir `~/.foundry/cache`
    pub fn foundry_cache_dir() -> Option<PathBuf> {
        Self::foundry_dir().map(|p| p.join("cache"))
    }

    /// Returns the path to foundry rpc cache dir `~/.foundry/cache/rpc`
    pub fn foundry_rpc_cache_dir() -> Option<PathBuf> {
        Some(Self::foundry_cache_dir()?.join("rpc"))
    }
    /// Returns the path to foundry chain's cache dir `~/.foundry/cache/rpc/<chain>`
    pub fn foundry_chain_cache_dir(chain_id: impl Into<Chain>) -> Option<PathBuf> {
        Some(Self::foundry_rpc_cache_dir()?.join(chain_id.into().to_string()))
    }

    /// Returns the path to foundry's etherscan cache dir `~/.foundry/cache/etherscan`
    pub fn foundry_etherscan_cache_dir() -> Option<PathBuf> {
        Some(Self::foundry_cache_dir()?.join("etherscan"))
    }

    /// Returns the path to foundry's keystores dir `~/.foundry/keystores`
    pub fn foundry_keystores_dir() -> Option<PathBuf> {
        Some(Self::foundry_dir()?.join("keystores"))
    }

    /// Returns the path to foundry's etherscan cache dir for `chain_id`
    /// `~/.foundry/cache/etherscan/<chain>`
    pub fn foundry_etherscan_chain_cache_dir(chain_id: impl Into<Chain>) -> Option<PathBuf> {
        Some(Self::foundry_etherscan_cache_dir()?.join(chain_id.into().to_string()))
    }

    /// Returns the path to the cache dir of the `block` on the `chain`
    /// `~/.foundry/cache/rpc/<chain>/<block>
    pub fn foundry_block_cache_dir(chain_id: impl Into<Chain>, block: u64) -> Option<PathBuf> {
        Some(Self::foundry_chain_cache_dir(chain_id)?.join(format!("{block}")))
    }

    /// Returns the path to the cache file of the `block` on the `chain`
    /// `~/.foundry/cache/rpc/<chain>/<block>/storage.json`
    pub fn foundry_block_cache_file(chain_id: impl Into<Chain>, block: u64) -> Option<PathBuf> {
        Some(Self::foundry_block_cache_dir(chain_id, block)?.join("storage.json"))
    }

    #[doc = r#"Returns the path to `foundry`'s data directory inside the user's data directory
    |Platform | Value                                 | Example                          |
    | ------- | ------------------------------------- | -------------------------------- |
    | Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config/foundry | /home/alice/.config/foundry|
    | macOS   | `$HOME`/Library/Application Support/foundry   | /Users/Alice/Library/Application Support/foundry |
    | Windows | `{FOLDERID_RoamingAppData}/foundry`           | C:\Users\Alice\AppData\Roaming/foundry   |
    "#]
    pub fn data_dir() -> eyre::Result<PathBuf> {
        let path = dirs_next::data_dir().wrap_err("Failed to find data directory")?.join("foundry");
        std::fs::create_dir_all(&path).wrap_err("Failed to create module directory")?;
        Ok(path)
    }

    /// Returns the path to the `foundry.toml` file, the file is searched for in
    /// the current working directory and all parent directories until the root,
    /// and the first hit is used.
    ///
    /// If this search comes up empty, then it checks if a global `foundry.toml` exists at
    /// `~/.foundry/foundry.tol`, see [`Self::foundry_dir_toml()`]
    pub fn find_config_file() -> Option<PathBuf> {
        fn find(path: &Path) -> Option<PathBuf> {
            if path.is_absolute() {
                return match path.is_file() {
                    true => Some(path.to_path_buf()),
                    false => None,
                }
            }
            let cwd = std::env::current_dir().ok()?;
            let mut cwd = cwd.as_path();
            loop {
                let file_path = cwd.join(path);
                if file_path.is_file() {
                    return Some(file_path)
                }
                cwd = cwd.parent()?;
            }
        }
        find(Env::var_or("FOUNDRY_CONFIG", Config::FILE_NAME).as_ref())
            .or_else(|| Self::foundry_dir_toml().filter(|p| p.exists()))
    }

    /// Clears the foundry cache
    pub fn clean_foundry_cache() -> eyre::Result<()> {
        if let Some(cache_dir) = Config::foundry_cache_dir() {
            let path = cache_dir.as_path();
            let _ = fs::remove_dir_all(path);
        } else {
            eyre::bail!("failed to get foundry_cache_dir");
        }

        Ok(())
    }

    /// Clears the foundry cache for `chain`
    pub fn clean_foundry_chain_cache(chain: Chain) -> eyre::Result<()> {
        if let Some(cache_dir) = Config::foundry_chain_cache_dir(chain) {
            let path = cache_dir.as_path();
            let _ = fs::remove_dir_all(path);
        } else {
            eyre::bail!("failed to get foundry_chain_cache_dir");
        }

        Ok(())
    }

    /// Clears the foundry cache for `chain` and `block`
    pub fn clean_foundry_block_cache(chain: Chain, block: u64) -> eyre::Result<()> {
        if let Some(cache_dir) = Config::foundry_block_cache_dir(chain, block) {
            let path = cache_dir.as_path();
            let _ = fs::remove_dir_all(path);
        } else {
            eyre::bail!("failed to get foundry_block_cache_dir");
        }

        Ok(())
    }

    /// Clears the foundry etherscan cache
    pub fn clean_foundry_etherscan_cache() -> eyre::Result<()> {
        if let Some(cache_dir) = Config::foundry_etherscan_cache_dir() {
            let path = cache_dir.as_path();
            let _ = fs::remove_dir_all(path);
        } else {
            eyre::bail!("failed to get foundry_etherscan_cache_dir");
        }

        Ok(())
    }

    /// Clears the foundry etherscan cache for `chain`
    pub fn clean_foundry_etherscan_chain_cache(chain: Chain) -> eyre::Result<()> {
        if let Some(cache_dir) = Config::foundry_etherscan_chain_cache_dir(chain) {
            let path = cache_dir.as_path();
            let _ = fs::remove_dir_all(path);
        } else {
            eyre::bail!("failed to get foundry_etherscan_cache_dir for chain: {}", chain);
        }

        Ok(())
    }

    /// List the data in the foundry cache
    pub fn list_foundry_cache() -> eyre::Result<Cache> {
        if let Some(cache_dir) = Config::foundry_rpc_cache_dir() {
            let mut cache = Cache { chains: vec![] };
            if !cache_dir.exists() {
                return Ok(cache)
            }
            if let Ok(entries) = cache_dir.as_path().read_dir() {
                for entry in entries.flatten().filter(|x| x.path().is_dir()) {
                    match Chain::from_str(&entry.file_name().to_string_lossy()) {
                        Ok(chain) => cache.chains.push(Self::list_foundry_chain_cache(chain)?),
                        Err(_) => continue,
                    }
                }
                Ok(cache)
            } else {
                eyre::bail!("failed to access foundry_cache_dir");
            }
        } else {
            eyre::bail!("failed to get foundry_cache_dir");
        }
    }

    /// List the cached data for `chain`
    pub fn list_foundry_chain_cache(chain: Chain) -> eyre::Result<ChainCache> {
        let block_explorer_data_size = match Config::foundry_etherscan_chain_cache_dir(chain) {
            Some(cache_dir) => Self::get_cached_block_explorer_data(&cache_dir)?,
            None => {
                warn!("failed to access foundry_etherscan_chain_cache_dir");
                0
            }
        };

        if let Some(cache_dir) = Config::foundry_chain_cache_dir(chain) {
            let blocks = Self::get_cached_blocks(&cache_dir)?;
            Ok(ChainCache {
                name: chain.to_string(),
                blocks,
                block_explorer: block_explorer_data_size,
            })
        } else {
            eyre::bail!("failed to get foundry_chain_cache_dir");
        }
    }

    //The path provided to this function should point to a cached chain folder
    fn get_cached_blocks(chain_path: &Path) -> eyre::Result<Vec<(String, u64)>> {
        let mut blocks = vec![];
        if !chain_path.exists() {
            return Ok(blocks)
        }
        for block in chain_path.read_dir()?.flatten().filter(|x| x.file_type().unwrap().is_dir()) {
            let filepath = block.path().join("storage.json");
            blocks.push((
                block.file_name().to_string_lossy().into_owned(),
                fs::metadata(filepath)?.len(),
            ));
        }
        Ok(blocks)
    }

    //The path provided to this function should point to the etherscan cache for a chain
    fn get_cached_block_explorer_data(chain_path: &Path) -> eyre::Result<u64> {
        if !chain_path.exists() {
            return Ok(0)
        }

        fn dir_size_recursive(mut dir: fs::ReadDir) -> eyre::Result<u64> {
            dir.try_fold(0, |acc, file| {
                let file = file?;
                let size = match file.metadata()? {
                    data if data.is_dir() => dir_size_recursive(fs::read_dir(file.path())?)?,
                    data => data.len(),
                };
                Ok(acc + size)
            })
        }

        dir_size_recursive(fs::read_dir(chain_path)?)
    }

    fn merge_toml_provider(
        mut figment: Figment,
        toml_provider: impl Provider,
        profile: Profile,
    ) -> Figment {
        figment = figment.select(profile.clone());

        // add warnings
        figment = {
            let warnings = WarningsProvider::for_figment(&toml_provider, &figment);
            figment.merge(warnings)
        };

        // use [profile.<profile>] as [<profile>]
        let mut profiles = vec![Config::DEFAULT_PROFILE];
        if profile != Config::DEFAULT_PROFILE {
            profiles.push(profile.clone());
        }
        let provider = toml_provider.strict_select(profiles);

        // apply any key fixes
        let provider = BackwardsCompatTomlProvider(ForcedSnakeCaseData(provider));

        // merge the default profile as a base
        if profile != Config::DEFAULT_PROFILE {
            figment = figment.merge(provider.rename(Config::DEFAULT_PROFILE, profile.clone()));
        }
        // merge special keys into config
        for standalone_key in Config::STANDALONE_SECTIONS {
            if let Some(fallback) = STANDALONE_FALLBACK_SECTIONS.get(standalone_key) {
                figment = figment.merge(
                    provider
                        .fallback(standalone_key, fallback)
                        .wrap(profile.clone(), standalone_key),
                );
            } else {
                figment = figment.merge(provider.wrap(profile.clone(), standalone_key));
            }
        }
        // merge the profile
        figment = figment.merge(provider);
        figment
    }
}

impl From<Config> for Figment {
    fn from(c: Config) -> Figment {
        let profile = Config::selected_profile();
        let mut figment = Figment::default().merge(DappHardhatDirProvider(&c.__root.0));

        // merge global foundry.toml file
        if let Some(global_toml) = Config::foundry_dir_toml().filter(|p| p.exists()) {
            figment = Config::merge_toml_provider(
                figment,
                TomlFileProvider::new(None, global_toml).cached(),
                profile.clone(),
            );
        }
        // merge local foundry.toml file
        figment = Config::merge_toml_provider(
            figment,
            TomlFileProvider::new(Some("FOUNDRY_CONFIG"), c.__root.0.join(Config::FILE_NAME))
                .cached(),
            profile.clone(),
        );

        // merge environment variables
        figment = figment
            .merge(
                Env::prefixed("DAPP_")
                    .ignore(&["REMAPPINGS", "LIBRARIES", "FFI", "FS_PERMISSIONS"])
                    .global(),
            )
            .merge(
                Env::prefixed("DAPP_TEST_")
                    .ignore(&["CACHE", "FUZZ_RUNS", "DEPTH", "FFI", "FS_PERMISSIONS"])
                    .global(),
            )
            .merge(DappEnvCompatProvider)
            .merge(Env::raw().only(&["ETHERSCAN_API_KEY", "ETH_RPC_URL"]))
            .merge(
                Env::prefixed("FOUNDRY_")
                    .ignore(&["PROFILE", "REMAPPINGS", "LIBRARIES", "FFI", "FS_PERMISSIONS"])
                    .map(|key| {
                        let key = key.as_str();
                        if Config::STANDALONE_SECTIONS.iter().any(|section| {
                            key.starts_with(&format!("{}_", section.to_ascii_uppercase()))
                        }) {
                            key.replacen('_', ".", 1).into()
                        } else {
                            key.into()
                        }
                    })
                    .global(),
            )
            .select(profile.clone());

        // we try to merge remappings after we've merged all other providers, this prevents
        // redundant fs lookups to determine the default remappings that are eventually updated by
        // other providers, like the toml file
        let remappings = RemappingsProvider {
            auto_detect_remappings: figment
                .extract_inner::<bool>("auto_detect_remappings")
                .unwrap_or(true),
            lib_paths: figment
                .extract_inner::<Vec<PathBuf>>("libs")
                .map(Cow::Owned)
                .unwrap_or_else(|_| Cow::Borrowed(&c.libs)),
            root: &c.__root.0,
            remappings: figment.extract_inner::<Vec<Remapping>>("remappings"),
        };
        let merge = figment.merge(remappings);

        Figment::from(c).merge(merge).select(profile)
    }
}

/// Wrapper type for `regex::Regex` that implements `PartialEq`
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegexWrapper {
    #[serde(with = "serde_regex")]
    inner: regex::Regex,
}

impl std::ops::Deref for RegexWrapper {
    type Target = regex::Regex;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::cmp::PartialEq for RegexWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<RegexWrapper> for regex::Regex {
    fn from(wrapper: RegexWrapper) -> Self {
        wrapper.inner
    }
}

impl From<regex::Regex> for RegexWrapper {
    fn from(re: Regex) -> Self {
        RegexWrapper { inner: re }
    }
}

/// Ser/de `globset::Glob` explicitly to handle `Option<Glob>` properly
pub(crate) mod from_opt_glob {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<globset::Glob>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(glob) => serializer.serialize_str(glob.glob()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<globset::Glob>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = s {
            return Ok(Some(globset::Glob::new(&s).map_err(serde::de::Error::custom)?))
        }
        Ok(None)
    }
}

/// A helper wrapper around the root path used during Config detection
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RootPath(pub PathBuf);

impl Default for RootPath {
    fn default() -> Self {
        ".".into()
    }
}

impl<P: Into<PathBuf>> From<P> for RootPath {
    fn from(p: P) -> Self {
        RootPath(p.into())
    }
}

impl AsRef<Path> for RootPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

/// Parses a config profile
///
/// All `Profile` date is ignored by serde, however the `Config::to_string_pretty` includes it and
/// returns a toml table like
///
/// ```toml
/// #[profile.default]
/// src = "..."
/// ```
/// This ignores the `#[profile.default]` part in the toml
pub fn parse_with_profile<T: serde::de::DeserializeOwned>(
    s: &str,
) -> Result<Option<(Profile, T)>, Error> {
    let figment = Config::merge_toml_provider(
        Figment::new(),
        Toml::string(s).nested(),
        Config::DEFAULT_PROFILE,
    );
    if figment.profiles().any(|p| p == Config::DEFAULT_PROFILE) {
        Ok(Some((Config::DEFAULT_PROFILE, figment.select(Config::DEFAULT_PROFILE).extract()?)))
    } else {
        Ok(None)
    }
}

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("Foundry Config")
    }

    #[track_caller]
    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        let mut data = Serialized::defaults(self).data()?;
        if let Some(entry) = data.get_mut(&self.profile) {
            entry.insert("root".to_string(), Value::serialize(self.__root.clone())?);
        }
        Ok(data)
    }

    fn profile(&self) -> Option<Profile> {
        Some(self.profile.clone())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profile: Self::DEFAULT_PROFILE,
            fs_permissions: FsPermissions::new([PathPermission::read("out")]),
            cancun: false,
            __root: Default::default(),
            src: "src".into(),
            test: "test".into(),
            script: "script".into(),
            out: "out".into(),
            libs: vec!["lib".into()],
            cache: true,
            cache_path: "cache".into(),
            broadcast: "broadcast".into(),
            allow_paths: vec![],
            include_paths: vec![],
            force: false,
            evm_version: EvmVersion::Paris,
            gas_reports: vec!["*".to_string()],
            gas_reports_ignore: vec![],
            solc: None,
            auto_detect_solc: true,
            offline: false,
            optimizer: true,
            optimizer_runs: 200,
            optimizer_details: None,
            model_checker: None,
            extra_output: Default::default(),
            extra_output_files: Default::default(),
            names: false,
            sizes: false,
            test_pattern: None,
            test_pattern_inverse: None,
            contract_pattern: None,
            contract_pattern_inverse: None,
            path_pattern: None,
            path_pattern_inverse: None,
            fuzz: Default::default(),
            invariant: Default::default(),
            ffi: false,
            sender: Config::DEFAULT_SENDER,
            tx_origin: Config::DEFAULT_SENDER,
            initial_balance: U256::from(0xffffffffffffffffffffffffu128),
            block_number: 1,
            fork_block_number: None,
            chain: None,
            gas_limit: i64::MAX.into(),
            code_size_limit: None,
            gas_price: None,
            block_base_fee_per_gas: 0,
            block_coinbase: Address::ZERO,
            block_timestamp: 1,
            block_difficulty: 0,
            block_prevrandao: Default::default(),
            block_gas_limit: None,
            memory_limit: 1 << 27, // 2**27 = 128MiB = 134_217_728 bytes
            eth_rpc_url: None,
            eth_rpc_jwt: None,
            etherscan_api_key: None,
            verbosity: 0,
            remappings: vec![],
            auto_detect_remappings: true,
            libraries: vec![],
            ignored_error_codes: vec![
                SolidityErrorCode::SpdxLicenseNotProvided,
                SolidityErrorCode::ContractExceeds24576Bytes,
                SolidityErrorCode::ContractInitCodeSizeExceeds49152Bytes,
            ],
            deny_warnings: false,
            via_ir: false,
            rpc_storage_caching: Default::default(),
            rpc_endpoints: Default::default(),
            etherscan: Default::default(),
            no_storage_caching: false,
            no_rpc_rate_limit: false,
            use_literal_content: false,
            bytecode_hash: BytecodeHash::Ipfs,
            cbor_metadata: true,
            revert_strings: None,
            sparse_mode: false,
            build_info: false,
            build_info_path: None,
            fmt: Default::default(),
            doc: Default::default(),
            labels: Default::default(),
            __non_exhaustive: (),
            __warnings: vec![],
        }
    }
}

/// Wrapper for the config's `gas_limit` value necessary because toml-rs can't handle larger number because integers are stored signed: <https://github.com/alexcrichton/toml-rs/issues/256>
///
/// Due to this limitation this type will be serialized/deserialized as String if it's larger than
/// `i64`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GasLimit(pub u64);

impl From<u64> for GasLimit {
    fn from(gas: u64) -> Self {
        Self(gas)
    }
}
impl From<i64> for GasLimit {
    fn from(gas: i64) -> Self {
        Self(gas as u64)
    }
}
impl From<i32> for GasLimit {
    fn from(gas: i32) -> Self {
        Self(gas as u64)
    }
}
impl From<u32> for GasLimit {
    fn from(gas: u32) -> Self {
        Self(gas as u64)
    }
}

impl From<GasLimit> for u64 {
    fn from(gas: GasLimit) -> Self {
        gas.0
    }
}

impl Serialize for GasLimit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.0 > i64::MAX as u64 {
            serializer.serialize_str(&self.0.to_string())
        } else {
            serializer.serialize_u64(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for GasLimit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Gas {
            Number(u64),
            Text(String),
        }

        let gas = match Gas::deserialize(deserializer)? {
            Gas::Number(num) => GasLimit(num),
            Gas::Text(s) => match s.as_str() {
                "max" | "MAX" | "Max" | "u64::MAX" | "u64::Max" => GasLimit(u64::MAX),
                s => GasLimit(s.parse().map_err(D::Error::custom)?),
            },
        };

        Ok(gas)
    }
}

/// Variants for selecting the [`Solc`] instance
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SolcReq {
    /// Requires a specific solc version, that's either already installed (via `svm`) or will be
    /// auto installed (via `svm`)
    Version(Version),
    /// Path to an existing local solc installation
    Local(PathBuf),
}

impl<T: AsRef<str>> From<T> for SolcReq {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        if let Ok(v) = Version::from_str(s) {
            SolcReq::Version(v)
        } else {
            SolcReq::Local(s.into())
        }
    }
}

/// A convenience provider to retrieve a toml file.
/// This will return an error if the env var is set but the file does not exist
struct TomlFileProvider {
    pub env_var: Option<&'static str>,
    pub default: PathBuf,
    pub cache: Option<Result<Map<Profile, Dict>, Error>>,
}

impl TomlFileProvider {
    fn new(env_var: Option<&'static str>, default: impl Into<PathBuf>) -> Self {
        Self { env_var, default: default.into(), cache: None }
    }

    fn env_val(&self) -> Option<String> {
        self.env_var.and_then(Env::var)
    }

    fn file(&self) -> PathBuf {
        self.env_val().map(PathBuf::from).unwrap_or_else(|| self.default.clone())
    }

    fn is_missing(&self) -> bool {
        if let Some(file) = self.env_val() {
            let path = Path::new(&file);
            if !path.exists() {
                return true
            }
        }
        false
    }

    pub fn cached(mut self) -> Self {
        self.cache = Some(self.read());
        self
    }

    fn read(&self) -> Result<Map<Profile, Dict>, Error> {
        use serde::de::Error as _;
        if let Some(file) = self.env_val() {
            let path = Path::new(&file);
            if !path.exists() {
                return Err(Error::custom(format!(
                    "Config file `{}` set in env var `{}` does not exist",
                    file,
                    self.env_var.unwrap()
                )))
            }
            Toml::file(file)
        } else {
            Toml::file(&self.default)
        }
        .nested()
        .data()
    }
}

impl Provider for TomlFileProvider {
    fn metadata(&self) -> Metadata {
        if self.is_missing() {
            Metadata::named("TOML file provider")
        } else {
            Toml::file(self.file()).nested().metadata()
        }
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        if let Some(cache) = self.cache.as_ref() {
            cache.clone()
        } else {
            self.read()
        }
    }
}

/// A Provider that ensures all keys are snake case if they're not standalone sections, See
/// `Config::STANDALONE_SECTIONS`
struct ForcedSnakeCaseData<P>(P);

impl<P: Provider> Provider for ForcedSnakeCaseData<P> {
    fn metadata(&self) -> Metadata {
        self.0.metadata()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let mut map = Map::new();
        for (profile, dict) in self.0.data()? {
            if Config::STANDALONE_SECTIONS.contains(&profile.as_ref()) {
                // don't force snake case for keys in standalone sections
                map.insert(profile, dict);
                continue
            }
            map.insert(profile, dict.into_iter().map(|(k, v)| (k.to_snake_case(), v)).collect());
        }
        Ok(map)
    }
}

/// A Provider that handles breaking changes in toml files
struct BackwardsCompatTomlProvider<P>(P);

impl<P: Provider> Provider for BackwardsCompatTomlProvider<P> {
    fn metadata(&self) -> Metadata {
        self.0.metadata()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let mut map = Map::new();
        let solc_env = std::env::var("FOUNDRY_SOLC_VERSION")
            .or_else(|_| std::env::var("DAPP_SOLC_VERSION"))
            .map(Value::from)
            .ok();
        for (profile, mut dict) in self.0.data()? {
            if let Some(v) = solc_env.clone() {
                // ENV var takes precedence over config file
                dict.insert("solc".to_string(), v);
            } else if let Some(v) = dict.remove("solc_version") {
                // only insert older variant if not already included
                if !dict.contains_key("solc") {
                    dict.insert("solc".to_string(), v);
                }
            }
            map.insert(profile, dict);
        }
        Ok(map)
    }
}

/// A provider that sets the `src` and `output` path depending on their existence.
struct DappHardhatDirProvider<'a>(&'a Path);

impl<'a> Provider for DappHardhatDirProvider<'a> {
    fn metadata(&self) -> Metadata {
        Metadata::named("Dapp Hardhat dir compat")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let mut dict = Dict::new();
        dict.insert(
            "src".to_string(),
            ProjectPathsConfig::find_source_dir(self.0)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
                .into(),
        );
        dict.insert(
            "out".to_string(),
            ProjectPathsConfig::find_artifacts_dir(self.0)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
                .into(),
        );

        // detect libs folders:
        //   if `lib` _and_ `node_modules` exists: include both
        //   if only `node_modules` exists: include `node_modules`
        //   include `lib` otherwise
        let mut libs = vec![];
        let node_modules = self.0.join("node_modules");
        let lib = self.0.join("lib");
        if node_modules.exists() {
            if lib.exists() {
                libs.push(lib.file_name().unwrap().to_string_lossy().to_string());
            }
            libs.push(node_modules.file_name().unwrap().to_string_lossy().to_string());
        } else {
            libs.push(lib.file_name().unwrap().to_string_lossy().to_string());
        }

        dict.insert("libs".to_string(), libs.into());

        Ok(Map::from([(Config::selected_profile(), dict)]))
    }
}

/// A provider that checks for DAPP_ env vars that are named differently than FOUNDRY_
struct DappEnvCompatProvider;

impl Provider for DappEnvCompatProvider {
    fn metadata(&self) -> Metadata {
        Metadata::named("Dapp env compat")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        use serde::de::Error as _;
        use std::env;

        let mut dict = Dict::new();
        if let Ok(val) = env::var("DAPP_TEST_NUMBER") {
            dict.insert(
                "block_number".to_string(),
                val.parse::<u64>().map_err(figment::Error::custom)?.into(),
            );
        }
        if let Ok(val) = env::var("DAPP_TEST_ADDRESS") {
            dict.insert("sender".to_string(), val.into());
        }
        if let Ok(val) = env::var("DAPP_FORK_BLOCK") {
            dict.insert(
                "fork_block_number".to_string(),
                val.parse::<u64>().map_err(figment::Error::custom)?.into(),
            );
        } else if let Ok(val) = env::var("DAPP_TEST_NUMBER") {
            dict.insert(
                "fork_block_number".to_string(),
                val.parse::<u64>().map_err(figment::Error::custom)?.into(),
            );
        }
        if let Ok(val) = env::var("DAPP_TEST_TIMESTAMP") {
            dict.insert(
                "block_timestamp".to_string(),
                val.parse::<u64>().map_err(figment::Error::custom)?.into(),
            );
        }
        if let Ok(val) = env::var("DAPP_BUILD_OPTIMIZE_RUNS") {
            dict.insert(
                "optimizer_runs".to_string(),
                val.parse::<u64>().map_err(figment::Error::custom)?.into(),
            );
        }
        if let Ok(val) = env::var("DAPP_BUILD_OPTIMIZE") {
            // Activate Solidity optimizer (0 or 1)
            let val = val.parse::<u8>().map_err(figment::Error::custom)?;
            if val > 1 {
                return Err(
                    format!("Invalid $DAPP_BUILD_OPTIMIZE value `{val}`, expected 0 or 1").into()
                )
            }
            dict.insert("optimizer".to_string(), (val == 1).into());
        }

        // libraries in env vars either as `[..]` or single string separated by comma
        if let Ok(val) = env::var("DAPP_LIBRARIES").or_else(|_| env::var("FOUNDRY_LIBRARIES")) {
            dict.insert("libraries".to_string(), utils::to_array_value(&val)?);
        }

        let mut fuzz_dict = Dict::new();
        if let Ok(val) = env::var("DAPP_TEST_FUZZ_RUNS") {
            fuzz_dict.insert(
                "runs".to_string(),
                val.parse::<u32>().map_err(figment::Error::custom)?.into(),
            );
        }
        dict.insert("fuzz".to_string(), fuzz_dict.into());

        let mut invariant_dict = Dict::new();
        if let Ok(val) = env::var("DAPP_TEST_DEPTH") {
            invariant_dict.insert(
                "depth".to_string(),
                val.parse::<u32>().map_err(figment::Error::custom)?.into(),
            );
        }
        dict.insert("invariant".to_string(), invariant_dict.into());

        Ok(Map::from([(Config::selected_profile(), dict)]))
    }
}

/// Renames a profile from `from` to `to
///
/// For example given:
///
/// ```toml
/// [from]
/// key = "value"
/// ```
///
/// RenameProfileProvider will output
///
/// ```toml
/// [to]
/// key = "value"
/// ```
struct RenameProfileProvider<P> {
    provider: P,
    from: Profile,
    to: Profile,
}

impl<P> RenameProfileProvider<P> {
    pub fn new(provider: P, from: impl Into<Profile>, to: impl Into<Profile>) -> Self {
        Self { provider, from: from.into(), to: to.into() }
    }
}

impl<P: Provider> Provider for RenameProfileProvider<P> {
    fn metadata(&self) -> Metadata {
        self.provider.metadata()
    }
    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let mut data = self.provider.data()?;
        if let Some(data) = data.remove(&self.from) {
            return Ok(Map::from([(self.to.clone(), data)]))
        }
        Ok(Default::default())
    }
    fn profile(&self) -> Option<Profile> {
        Some(self.to.clone())
    }
}

/// Unwraps a profile reducing the key depth
///
/// For example given:
///
/// ```toml
/// [wrapping_key.profile]
/// key = "value"
/// ```
///
/// UnwrapProfileProvider will output:
///
/// ```toml
/// [profile]
/// key = "value"
/// ```
struct UnwrapProfileProvider<P> {
    provider: P,
    wrapping_key: Profile,
    profile: Profile,
}

impl<P> UnwrapProfileProvider<P> {
    pub fn new(provider: P, wrapping_key: impl Into<Profile>, profile: impl Into<Profile>) -> Self {
        Self { provider, wrapping_key: wrapping_key.into(), profile: profile.into() }
    }
}

impl<P: Provider> Provider for UnwrapProfileProvider<P> {
    fn metadata(&self) -> Metadata {
        self.provider.metadata()
    }
    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        self.provider.data().and_then(|mut data| {
            if let Some(profiles) = data.remove(&self.wrapping_key) {
                for (profile_str, profile_val) in profiles {
                    let profile = Profile::new(&profile_str);
                    if profile != self.profile {
                        continue
                    }
                    match profile_val {
                        Value::Dict(_, dict) => return Ok(profile.collect(dict)),
                        bad_val => {
                            let mut err = Error::from(figment::error::Kind::InvalidType(
                                bad_val.to_actual(),
                                "dict".into(),
                            ));
                            err.metadata = Some(self.provider.metadata());
                            err.profile = Some(self.profile.clone());
                            return Err(err)
                        }
                    }
                }
            }
            Ok(Default::default())
        })
    }
    fn profile(&self) -> Option<Profile> {
        Some(self.profile.clone())
    }
}

/// Wraps a profile in another profile
///
/// For example given:
///
/// ```toml
/// [profile]
/// key = "value"
/// ```
///
/// WrapProfileProvider will output:
///
/// ```toml
/// [wrapping_key.profile]
/// key = "value"
/// ```
struct WrapProfileProvider<P> {
    provider: P,
    wrapping_key: Profile,
    profile: Profile,
}

impl<P> WrapProfileProvider<P> {
    pub fn new(provider: P, wrapping_key: impl Into<Profile>, profile: impl Into<Profile>) -> Self {
        Self { provider, wrapping_key: wrapping_key.into(), profile: profile.into() }
    }
}

impl<P: Provider> Provider for WrapProfileProvider<P> {
    fn metadata(&self) -> Metadata {
        self.provider.metadata()
    }
    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        if let Some(inner) = self.provider.data()?.remove(&self.profile) {
            let value = Value::from(inner);
            let dict = [(self.profile.to_string().to_snake_case(), value)].into_iter().collect();
            Ok(self.wrapping_key.collect(dict))
        } else {
            Ok(Default::default())
        }
    }
    fn profile(&self) -> Option<Profile> {
        Some(self.profile.clone())
    }
}

/// Extracts the profile from the `profile` key and using the original key as backup, merging
/// values where necessary
///
/// For example given:
///
/// ```toml
/// [profile.cool]
/// key = "value"
///
/// [cool]
/// key2 = "value2"
/// ```
///
/// OptionalStrictProfileProvider will output:
///
/// ```toml
/// [cool]
/// key = "value"
/// key2 = "value2"
/// ```
///
/// And emit a deprecation warning
struct OptionalStrictProfileProvider<P> {
    provider: P,
    profiles: Vec<Profile>,
}

impl<P> OptionalStrictProfileProvider<P> {
    pub const PROFILE_PROFILE: Profile = Profile::const_new("profile");

    pub fn new(provider: P, profiles: impl IntoIterator<Item = impl Into<Profile>>) -> Self {
        Self { provider, profiles: profiles.into_iter().map(|profile| profile.into()).collect() }
    }
}

impl<P: Provider> Provider for OptionalStrictProfileProvider<P> {
    fn metadata(&self) -> Metadata {
        self.provider.metadata()
    }
    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let mut figment = Figment::from(&self.provider);
        for profile in &self.profiles {
            figment = figment.merge(UnwrapProfileProvider::new(
                &self.provider,
                Self::PROFILE_PROFILE,
                profile.clone(),
            ));
        }
        figment.data().map_err(|err| {
            // figment does tag metadata and tries to map metadata to an error, since we use a new
            // figment in this provider this new figment does not know about the metadata of the
            // provider and can't map the metadata to the error. Therefor we return the root error
            // if this error originated in the provider's data.
            if let Err(root_err) = self.provider.data() {
                return root_err
            }
            err
        })
    }
    fn profile(&self) -> Option<Profile> {
        self.profiles.last().cloned()
    }
}

trait ProviderExt: Provider {
    fn rename(
        &self,
        from: impl Into<Profile>,
        to: impl Into<Profile>,
    ) -> RenameProfileProvider<&Self> {
        RenameProfileProvider::new(self, from, to)
    }

    fn wrap(
        &self,
        wrapping_key: impl Into<Profile>,
        profile: impl Into<Profile>,
    ) -> WrapProfileProvider<&Self> {
        WrapProfileProvider::new(self, wrapping_key, profile)
    }

    fn strict_select(
        &self,
        profiles: impl IntoIterator<Item = impl Into<Profile>>,
    ) -> OptionalStrictProfileProvider<&Self> {
        OptionalStrictProfileProvider::new(self, profiles)
    }

    fn fallback(
        &self,
        profile: impl Into<Profile>,
        fallback: impl Into<Profile>,
    ) -> FallbackProfileProvider<&Self> {
        FallbackProfileProvider::new(self, profile, fallback)
    }
}
impl<P: Provider> ProviderExt for P {}

/// A subset of the foundry `Config`
/// used to initialize a `foundry.toml` file
///
/// # Example
///
/// ```rust
/// use foundry_config::{BasicConfig, Config};
/// use serde::Deserialize;
///
/// let my_config = Config::figment().extract::<BasicConfig>();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicConfig {
    /// the profile tag: `[profile.default]`
    #[serde(skip)]
    pub profile: Profile,
    /// path of the source contracts dir, like `src` or `contracts`
    pub src: PathBuf,
    /// path to where artifacts shut be written to
    pub out: PathBuf,
    /// all library folders to include, `lib`, `node_modules`
    pub libs: Vec<PathBuf>,
    /// `Remappings` to use for this repo
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remappings: Vec<RelativeRemapping>,
}

impl BasicConfig {
    /// Serialize the config as a String of TOML.
    ///
    /// This serializes to a table with the name of the profile
    pub fn to_string_pretty(&self) -> Result<String, toml::ser::Error> {
        let s = toml::to_string_pretty(self)?;
        Ok(format!(
            "\
[profile.{}]
{s}
# See more config options https://github.com/foundry-rs/foundry/blob/master/crates/config/README.md#all-options\n",
            self.profile
        ))
    }
}

pub(crate) mod from_str_lowercase {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: std::fmt::Display,
        S: Serializer,
    {
        serializer.collect_str(&value.to_string().to_lowercase())
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        String::deserialize(deserializer)?.to_lowercase().parse().map_err(serde::de::Error::custom)
    }
}

fn canonic(path: impl Into<PathBuf>) -> PathBuf {
    let path = path.into();
    foundry_compilers::utils::canonicalize(&path).unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cache::{CachedChains, CachedEndpoints},
        endpoints::{RpcEndpoint, RpcEndpointConfig, RpcEndpointType},
        etherscan::ResolvedEtherscanConfigs,
        fs_permissions::PathPermission,
    };
    use alloy_primitives::Address;
    use figment::{error::Kind::InvalidType, value::Value, Figment};
    use foundry_compilers::artifacts::{ModelCheckerEngine, YulDetails};
    use pretty_assertions::assert_eq;
    use std::{collections::BTreeMap, fs::File, io::Write, str::FromStr};
    use tempfile::tempdir;
    use NamedChain::Moonbeam;

    // Helper function to clear `__warnings` in config, since it will be populated during loading
    // from file, causing testing problem when comparing to those created from `default()`, etc.
    fn clear_warning(config: &mut Config) {
        config.__warnings = vec![];
    }

    #[test]
    fn default_sender() {
        assert_eq!(
            Config::DEFAULT_SENDER,
            Address::from_str("0x1804c8AB1F12E6bbf3894d4083f33e07309d1f38").unwrap()
        );
    }

    #[test]
    fn test_caching() {
        let mut config = Config::default();
        let chain_id = NamedChain::Mainnet;
        let url = "https://eth-mainnet.alchemyapi";
        assert!(config.enable_caching(url, chain_id));

        config.no_storage_caching = true;
        assert!(!config.enable_caching(url, chain_id));

        config.no_storage_caching = false;
        assert!(!config.enable_caching(url, NamedChain::Dev));
    }

    #[test]
    fn test_install_dir() {
        figment::Jail::expect_with(|jail| {
            let config = Config::load();
            assert_eq!(config.install_lib_dir(), PathBuf::from("lib"));
            jail.create_file(
                "foundry.toml",
                r"
                [profile.default]
                libs = ['node_modules', 'lib']
            ",
            )?;
            let config = Config::load();
            assert_eq!(config.install_lib_dir(), PathBuf::from("lib"));

            jail.create_file(
                "foundry.toml",
                r"
                [profile.default]
                libs = ['custom', 'node_modules', 'lib']
            ",
            )?;
            let config = Config::load();
            assert_eq!(config.install_lib_dir(), PathBuf::from("custom"));

            Ok(())
        });
    }

    #[test]
    fn test_figment_is_default() {
        figment::Jail::expect_with(|_| {
            let mut default: Config = Config::figment().extract().unwrap();
            default.profile = Config::default().profile;
            assert_eq!(default, Config::default());
            Ok(())
        });
    }

    #[test]
    fn test_default_round_trip() {
        figment::Jail::expect_with(|_| {
            let original = Config::figment();
            let roundtrip = Figment::from(Config::from_provider(&original));
            for figment in &[original, roundtrip] {
                let config = Config::from_provider(figment);
                assert_eq!(config, Config::default());
            }
            Ok(())
        });
    }

    #[test]
    fn ffi_env_disallowed() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("FOUNDRY_FFI", "true");
            jail.set_env("FFI", "true");
            jail.set_env("DAPP_FFI", "true");
            let config = Config::load();
            assert!(!config.ffi);

            Ok(())
        });
    }

    #[test]
    fn test_profile_env() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("FOUNDRY_PROFILE", "default");
            let figment = Config::figment();
            assert_eq!(figment.profile(), "default");

            jail.set_env("FOUNDRY_PROFILE", "hardhat");
            let figment: Figment = Config::hardhat().into();
            assert_eq!(figment.profile(), "hardhat");

            jail.create_file(
                "foundry.toml",
                r"
                [profile.default]
                libs = ['lib']
                [profile.local]
                libs = ['modules']
            ",
            )?;
            jail.set_env("FOUNDRY_PROFILE", "local");
            let config = Config::load();
            assert_eq!(config.libs, vec![PathBuf::from("modules")]);

            Ok(())
        });
    }

    #[test]
    fn test_default_test_path() {
        figment::Jail::expect_with(|_| {
            let config = Config::default();
            let paths_config = config.project_paths();
            assert_eq!(paths_config.tests, PathBuf::from(r"test"));
            Ok(())
        });
    }

    #[test]
    fn test_default_libs() {
        figment::Jail::expect_with(|jail| {
            let config = Config::load();
            assert_eq!(config.libs, vec![PathBuf::from("lib")]);

            fs::create_dir_all(jail.directory().join("node_modules")).unwrap();
            let config = Config::load();
            assert_eq!(config.libs, vec![PathBuf::from("node_modules")]);

            fs::create_dir_all(jail.directory().join("lib")).unwrap();
            let config = Config::load();
            assert_eq!(config.libs, vec![PathBuf::from("lib"), PathBuf::from("node_modules")]);

            Ok(())
        });
    }

    #[test]
    fn test_inheritance_from_default_test_path() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                test = "defaulttest"
                src  = "defaultsrc"
                libs = ['lib', 'node_modules']
                
                [profile.custom]
                src = "customsrc"
            "#,
            )?;

            let config = Config::load();
            assert_eq!(config.src, PathBuf::from("defaultsrc"));
            assert_eq!(config.libs, vec![PathBuf::from("lib"), PathBuf::from("node_modules")]);

            jail.set_env("FOUNDRY_PROFILE", "custom");
            let config = Config::load();

            assert_eq!(config.src, PathBuf::from("customsrc"));
            assert_eq!(config.test, PathBuf::from("defaulttest"));
            assert_eq!(config.libs, vec![PathBuf::from("lib"), PathBuf::from("node_modules")]);

            Ok(())
        });
    }

    #[test]
    fn test_custom_test_path() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                test = "mytest"
            "#,
            )?;

            let config = Config::load();
            let paths_config = config.project_paths();
            assert_eq!(paths_config.tests, PathBuf::from(r"mytest"));
            Ok(())
        });
    }

    #[test]
    fn test_remappings() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                src = "some-source"
                out = "some-out"
                cache = true
            "#,
            )?;
            let config = Config::load();
            assert!(config.remappings.is_empty());

            jail.create_file(
                "remappings.txt",
                r"
                file-ds-test/=lib/ds-test/
                file-other/=lib/other/
            ",
            )?;

            let config = Config::load();
            assert_eq!(
                config.remappings,
                vec![
                    Remapping::from_str("file-ds-test/=lib/ds-test/").unwrap().into(),
                    Remapping::from_str("file-other/=lib/other/").unwrap().into(),
                ],
            );

            jail.set_env("DAPP_REMAPPINGS", "ds-test=lib/ds-test/\nother/=lib/other/");
            let config = Config::load();

            assert_eq!(
                config.remappings,
                vec![
                    // From environment (should have precedence over remapping.txt)
                    Remapping::from_str("ds-test=lib/ds-test/").unwrap().into(),
                    Remapping::from_str("other/=lib/other/").unwrap().into(),
                    // From remapping.txt (should have less precedence than remapping.txt)
                    Remapping::from_str("file-ds-test/=lib/ds-test/").unwrap().into(),
                    Remapping::from_str("file-other/=lib/other/").unwrap().into(),
                ],
            );

            Ok(())
        });
    }

    #[test]
    fn test_remappings_override() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                src = "some-source"
                out = "some-out"
                cache = true
            "#,
            )?;
            let config = Config::load();
            assert!(config.remappings.is_empty());

            jail.create_file(
                "remappings.txt",
                r"
                ds-test/=lib/ds-test/
                other/=lib/other/
            ",
            )?;

            let config = Config::load();
            assert_eq!(
                config.remappings,
                vec![
                    Remapping::from_str("ds-test/=lib/ds-test/").unwrap().into(),
                    Remapping::from_str("other/=lib/other/").unwrap().into(),
                ],
            );

            jail.set_env("DAPP_REMAPPINGS", "ds-test/=lib/ds-test/src/\nenv-lib/=lib/env-lib/");
            let config = Config::load();

            // Remappings should now be:
            // - ds-test from environment (lib/ds-test/src/)
            // - other from remappings.txt (lib/other/)
            // - env-lib from environment (lib/env-lib/)
            assert_eq!(
                config.remappings,
                vec![
                    Remapping::from_str("ds-test/=lib/ds-test/src/").unwrap().into(),
                    Remapping::from_str("env-lib/=lib/env-lib/").unwrap().into(),
                    Remapping::from_str("other/=lib/other/").unwrap().into(),
                ],
            );

            // contains additional remapping to the source dir
            assert_eq!(
                config.get_all_remappings(),
                vec![
                    Remapping::from_str("ds-test/=lib/ds-test/src/").unwrap(),
                    Remapping::from_str("env-lib/=lib/env-lib/").unwrap(),
                    Remapping::from_str("other/=lib/other/").unwrap(),
                ],
            );

            Ok(())
        });
    }

    #[test]
    fn test_can_update_libs() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                libs = ["node_modules"]
            "#,
            )?;

            let mut config = Config::load();
            config.libs.push("libs".into());
            config.update_libs().unwrap();

            let config = Config::load();
            assert_eq!(config.libs, vec![PathBuf::from("node_modules"), PathBuf::from("libs"),]);
            Ok(())
        });
    }

    #[test]
    fn test_large_gas_limit() {
        figment::Jail::expect_with(|jail| {
            let gas = u64::MAX;
            jail.create_file(
                "foundry.toml",
                &format!(
                    r#"
                [profile.default]
                gas_limit = "{gas}"
            "#
                ),
            )?;

            let config = Config::load();
            assert_eq!(config, Config { gas_limit: gas.into(), ..Config::default() });

            Ok(())
        });
    }

    #[test]
    #[should_panic]
    fn test_toml_file_parse_failure() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                eth_rpc_url = "https://example.com/
            "#,
            )?;

            let _config = Config::load();

            Ok(())
        });
    }

    #[test]
    #[should_panic]
    fn test_toml_file_non_existing_config_var_failure() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("FOUNDRY_CONFIG", "this config does not exist");

            let _config = Config::load();

            Ok(())
        });
    }

    #[test]
    fn test_resolve_etherscan_with_chain() {
        figment::Jail::expect_with(|jail| {
            let env_key = "__BSC_ETHERSCAN_API_KEY";
            let env_value = "env value";
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]

                [etherscan]
                bsc = { key = "${__BSC_ETHERSCAN_API_KEY}", url = "https://api.bscscan.com/api" }
            "#,
            )?;

            let config = Config::load();
            assert!(config
                .get_etherscan_config_with_chain(Some(NamedChain::BinanceSmartChain.into()))
                .is_err());

            std::env::set_var(env_key, env_value);

            assert_eq!(
                config
                    .get_etherscan_config_with_chain(Some(NamedChain::BinanceSmartChain.into()))
                    .unwrap()
                    .unwrap()
                    .key,
                env_value
            );

            let mut with_key = config;
            with_key.etherscan_api_key = Some("via etherscan_api_key".to_string());

            assert_eq!(
                with_key
                    .get_etherscan_config_with_chain(Some(NamedChain::BinanceSmartChain.into()))
                    .unwrap()
                    .unwrap()
                    .key,
                "via etherscan_api_key"
            );

            std::env::remove_var(env_key);
            Ok(())
        });
    }

    #[test]
    fn test_resolve_etherscan() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]

                [etherscan]
                mainnet = { key = "FX42Z3BBJJEWXWGYV2X1CIPRSCN" }
                moonbeam = { key = "${_CONFIG_ETHERSCAN_MOONBEAM}" }
            "#,
            )?;

            let config = Config::load();

            assert!(config.etherscan.clone().resolved().has_unresolved());

            jail.set_env("_CONFIG_ETHERSCAN_MOONBEAM", "123456789");

            let configs = config.etherscan.resolved();
            assert!(!configs.has_unresolved());

            let mb_urls = Moonbeam.etherscan_urls().unwrap();
            let mainnet_urls = NamedChain::Mainnet.etherscan_urls().unwrap();
            assert_eq!(
                configs,
                ResolvedEtherscanConfigs::new([
                    (
                        "mainnet",
                        ResolvedEtherscanConfig {
                            api_url: mainnet_urls.0.to_string(),
                            chain: Some(NamedChain::Mainnet.into()),
                            browser_url: Some(mainnet_urls.1.to_string()),
                            key: "FX42Z3BBJJEWXWGYV2X1CIPRSCN".to_string(),
                        }
                    ),
                    (
                        "moonbeam",
                        ResolvedEtherscanConfig {
                            api_url: mb_urls.0.to_string(),
                            chain: Some(Moonbeam.into()),
                            browser_url: Some(mb_urls.1.to_string()),
                            key: "123456789".to_string(),
                        }
                    ),
                ])
            );

            Ok(())
        });
    }

    #[test]
    fn test_resolve_rpc_url() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                [rpc_endpoints]
                optimism = "https://example.com/"
                mainnet = "${_CONFIG_MAINNET}"
            "#,
            )?;
            jail.set_env("_CONFIG_MAINNET", "https://eth-mainnet.alchemyapi.io/v2/123455");

            let mut config = Config::load();
            assert_eq!("http://localhost:8545", config.get_rpc_url_or_localhost_http().unwrap());

            config.eth_rpc_url = Some("mainnet".to_string());
            assert_eq!(
                "https://eth-mainnet.alchemyapi.io/v2/123455",
                config.get_rpc_url_or_localhost_http().unwrap()
            );

            config.eth_rpc_url = Some("optimism".to_string());
            assert_eq!("https://example.com/", config.get_rpc_url_or_localhost_http().unwrap());

            Ok(())
        })
    }

    #[test]
    fn test_resolve_rpc_url_if_etherscan_set() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                etherscan_api_key = "dummy"
                [rpc_endpoints]
                optimism = "https://example.com/"
            "#,
            )?;

            let config = Config::load();
            assert_eq!("http://localhost:8545", config.get_rpc_url_or_localhost_http().unwrap());

            Ok(())
        })
    }

    #[test]
    fn test_resolve_rpc_url_alias() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                [rpc_endpoints]
                polygonMumbai = "https://polygon-mumbai.g.alchemy.com/v2/${_RESOLVE_RPC_ALIAS}"
            "#,
            )?;
            let mut config = Config::load();
            config.eth_rpc_url = Some("polygonMumbai".to_string());
            assert!(config.get_rpc_url().unwrap().is_err());

            jail.set_env("_RESOLVE_RPC_ALIAS", "123455");

            let mut config = Config::load();
            config.eth_rpc_url = Some("polygonMumbai".to_string());
            assert_eq!(
                "https://polygon-mumbai.g.alchemy.com/v2/123455",
                config.get_rpc_url().unwrap().unwrap()
            );

            Ok(())
        })
    }

    #[test]
    fn test_resolve_rpc_aliases() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
               [profile.default]
               [etherscan]
               arbitrum_alias = { key = "${TEST_RESOLVE_RPC_ALIAS_ARBISCAN}" }
               [rpc_endpoints]
               arbitrum_alias = "https://arb-mainnet.g.alchemy.com/v2/${TEST_RESOLVE_RPC_ALIAS_ARB_ONE}"
            "#,
            )?;

            jail.set_env("TEST_RESOLVE_RPC_ALIAS_ARB_ONE", "123455");
            jail.set_env("TEST_RESOLVE_RPC_ALIAS_ARBISCAN", "123455");

            let config = Config::load();

            let config = config.get_etherscan_config_with_chain(Some(NamedChain::Arbitrum.into()));
            assert!(config.is_err());
            assert_eq!(config.unwrap_err().to_string(), "At least one of `url` or `chain` must be present for Etherscan config with unknown alias `arbitrum_alias`");

            Ok(())
        });
    }

    #[test]
    fn test_resolve_rpc_config() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [rpc_endpoints]
                optimism = "https://example.com/"
                mainnet = { endpoint = "${_CONFIG_MAINNET}", retries = 3, retry_backoff = 1000, compute_units_per_second = 1000 }
            "#,
            )?;
            jail.set_env("_CONFIG_MAINNET", "https://eth-mainnet.alchemyapi.io/v2/123455");

            let config = Config::load();
            assert_eq!(
                RpcEndpoints::new([
                    (
                        "optimism",
                        RpcEndpointType::String(RpcEndpoint::Url(
                            "https://example.com/".to_string()
                        ))
                    ),
                    (
                        "mainnet",
                        RpcEndpointType::Config(RpcEndpointConfig {
                            endpoint: RpcEndpoint::Env("${_CONFIG_MAINNET}".to_string()),
                            retries: Some(3),
                            retry_backoff: Some(1000),
                            compute_units_per_second: Some(1000),
                        })
                    ),
                ]),
                config.rpc_endpoints
            );

            let resolved = config.rpc_endpoints.resolved();
            assert_eq!(
                RpcEndpoints::new([
                    ("optimism", RpcEndpoint::Url("https://example.com/".to_string())),
                    (
                        "mainnet",
                        RpcEndpoint::Url("https://eth-mainnet.alchemyapi.io/v2/123455".to_string())
                    ),
                ])
                .resolved(),
                resolved
            );
            Ok(())
        })
    }

    #[test]
    fn test_resolve_endpoints() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                eth_rpc_url = "optimism"
                [rpc_endpoints]
                optimism = "https://example.com/"
                mainnet = "${_CONFIG_MAINNET}"
                mainnet_2 = "https://eth-mainnet.alchemyapi.io/v2/${_CONFIG_API_KEY1}"
                mainnet_3 = "https://eth-mainnet.alchemyapi.io/v2/${_CONFIG_API_KEY1}/${_CONFIG_API_KEY2}"
            "#,
            )?;

            let config = Config::load();

            assert_eq!(config.get_rpc_url().unwrap().unwrap(), "https://example.com/");

            assert!(config.rpc_endpoints.clone().resolved().has_unresolved());

            jail.set_env("_CONFIG_MAINNET", "https://eth-mainnet.alchemyapi.io/v2/123455");
            jail.set_env("_CONFIG_API_KEY1", "123456");
            jail.set_env("_CONFIG_API_KEY2", "98765");

            let endpoints = config.rpc_endpoints.resolved();

            assert!(!endpoints.has_unresolved());

            assert_eq!(
                endpoints,
                RpcEndpoints::new([
                    ("optimism", RpcEndpoint::Url("https://example.com/".to_string())),
                    (
                        "mainnet",
                        RpcEndpoint::Url("https://eth-mainnet.alchemyapi.io/v2/123455".to_string())
                    ),
                    (
                        "mainnet_2",
                        RpcEndpoint::Url("https://eth-mainnet.alchemyapi.io/v2/123456".to_string())
                    ),
                    (
                        "mainnet_3",
                        RpcEndpoint::Url(
                            "https://eth-mainnet.alchemyapi.io/v2/123456/98765".to_string()
                        )
                    ),
                ])
                .resolved()
            );

            Ok(())
        });
    }

    #[test]
    fn test_extract_etherscan_config() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                etherscan_api_key = "optimism"

                [etherscan]
                optimism = { key = "https://etherscan-optimism.com/" }
                mumbai = { key = "https://etherscan-mumbai.com/" }
            "#,
            )?;

            let mut config = Config::load();

            let optimism = config.get_etherscan_api_key(Some(NamedChain::Optimism.into()));
            assert_eq!(optimism, Some("https://etherscan-optimism.com/".to_string()));

            config.etherscan_api_key = Some("mumbai".to_string());

            let mumbai = config.get_etherscan_api_key(Some(NamedChain::PolygonMumbai.into()));
            assert_eq!(mumbai, Some("https://etherscan-mumbai.com/".to_string()));

            Ok(())
        });
    }

    #[test]
    fn test_extract_etherscan_config_by_chain() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]

                [etherscan]
                mumbai = { key = "https://etherscan-mumbai.com/", chain = 80001 }
            "#,
            )?;

            let config = Config::load();

            let mumbai = config
                .get_etherscan_config_with_chain(Some(NamedChain::PolygonMumbai.into()))
                .unwrap()
                .unwrap();
            assert_eq!(mumbai.key, "https://etherscan-mumbai.com/".to_string());

            Ok(())
        });
    }

    #[test]
    fn test_extract_etherscan_config_by_chain_with_url() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]

                [etherscan]
                mumbai = { key = "https://etherscan-mumbai.com/", chain = 80001 , url =  "https://verifier-url.com/"}
            "#,
            )?;

            let config = Config::load();

            let mumbai = config
                .get_etherscan_config_with_chain(Some(NamedChain::PolygonMumbai.into()))
                .unwrap()
                .unwrap();
            assert_eq!(mumbai.key, "https://etherscan-mumbai.com/".to_string());
            assert_eq!(mumbai.api_url, "https://verifier-url.com/".to_string());

            Ok(())
        });
    }

    #[test]
    fn test_extract_etherscan_config_by_chain_and_alias() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                eth_rpc_url = "mumbai"

                [etherscan]
                mumbai = { key = "https://etherscan-mumbai.com/" }

                [rpc_endpoints]
                mumbai = "https://polygon-mumbai.g.alchemy.com/v2/mumbai"
            "#,
            )?;

            let config = Config::load();

            let mumbai = config.get_etherscan_config_with_chain(None).unwrap().unwrap();
            assert_eq!(mumbai.key, "https://etherscan-mumbai.com/".to_string());

            let mumbai_rpc = config.get_rpc_url().unwrap().unwrap();
            assert_eq!(mumbai_rpc, "https://polygon-mumbai.g.alchemy.com/v2/mumbai");
            Ok(())
        });
    }

    #[test]
    fn test_toml_file() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                src = "some-source"
                out = "some-out"
                cache = true
                eth_rpc_url = "https://example.com/"
                verbosity = 3
                remappings = ["ds-test=lib/ds-test/"]
                via_ir = true
                rpc_storage_caching = { chains = [1, "optimism", 999999], endpoints = "all"}
                use_literal_content = false
                bytecode_hash = "ipfs"
                cbor_metadata = true
                revert_strings = "strip"
                allow_paths = ["allow", "paths"]
                build_info_path = "build-info"

                [rpc_endpoints]
                optimism = "https://example.com/"
                mainnet = "${RPC_MAINNET}"
                mainnet_2 = "https://eth-mainnet.alchemyapi.io/v2/${API_KEY}"
                mainnet_3 = "https://eth-mainnet.alchemyapi.io/v2/${API_KEY}/${ANOTHER_KEY}"
            "#,
            )?;

            let config = Config::load();
            assert_eq!(
                config,
                Config {
                    src: "some-source".into(),
                    out: "some-out".into(),
                    cache: true,
                    eth_rpc_url: Some("https://example.com/".to_string()),
                    remappings: vec![Remapping::from_str("ds-test=lib/ds-test/").unwrap().into()],
                    verbosity: 3,
                    via_ir: true,
                    rpc_storage_caching: StorageCachingConfig {
                        chains: CachedChains::Chains(vec![
                            Chain::mainnet(),
                            Chain::optimism_mainnet(),
                            Chain::from_id(999999)
                        ]),
                        endpoints: CachedEndpoints::All
                    },
                    use_literal_content: false,
                    bytecode_hash: BytecodeHash::Ipfs,
                    cbor_metadata: true,
                    revert_strings: Some(RevertStrings::Strip),
                    allow_paths: vec![PathBuf::from("allow"), PathBuf::from("paths")],
                    rpc_endpoints: RpcEndpoints::new([
                        ("optimism", RpcEndpoint::Url("https://example.com/".to_string())),
                        ("mainnet", RpcEndpoint::Env("${RPC_MAINNET}".to_string())),
                        (
                            "mainnet_2",
                            RpcEndpoint::Env(
                                "https://eth-mainnet.alchemyapi.io/v2/${API_KEY}".to_string()
                            )
                        ),
                        (
                            "mainnet_3",
                            RpcEndpoint::Env(
                                "https://eth-mainnet.alchemyapi.io/v2/${API_KEY}/${ANOTHER_KEY}"
                                    .to_string()
                            )
                        ),
                    ]),
                    build_info_path: Some("build-info".into()),
                    ..Config::default()
                }
            );

            Ok(())
        });
    }

    #[test]
    fn test_load_remappings() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [profile.default]
                remappings = ['nested/=lib/nested/']
            ",
            )?;

            let config = Config::load_with_root(jail.directory());
            assert_eq!(
                config.remappings,
                vec![Remapping::from_str("nested/=lib/nested/").unwrap().into()]
            );

            Ok(())
        });
    }

    #[test]
    fn test_load_full_toml() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                auto_detect_solc = true
                block_base_fee_per_gas = 0
                block_coinbase = '0x0000000000000000000000000000000000000000'
                block_difficulty = 0
                block_prevrandao = '0x0000000000000000000000000000000000000000000000000000000000000000'
                block_number = 1
                block_timestamp = 1
                use_literal_content = false
                bytecode_hash = 'ipfs'
                cbor_metadata = true
                cache = true
                cache_path = 'cache'
                evm_version = 'london'
                extra_output = []
                extra_output_files = []
                ffi = false
                force = false
                gas_limit = 9223372036854775807
                gas_price = 0
                gas_reports = ['*']
                ignored_error_codes = [1878]
                deny_warnings = false
                initial_balance = '0xffffffffffffffffffffffff'
                libraries = []
                libs = ['lib']
                memory_limit = 134217728
                names = false
                no_storage_caching = false
                no_rpc_rate_limit = false
                offline = false
                optimizer = true
                optimizer_runs = 200
                out = 'out'
                remappings = ['nested/=lib/nested/']
                sender = '0x1804c8AB1F12E6bbf3894d4083f33e07309d1f38'
                sizes = false
                sparse_mode = false
                src = 'src'
                test = 'test'
                tx_origin = '0x1804c8AB1F12E6bbf3894d4083f33e07309d1f38'
                verbosity = 0
                via_ir = false
                
                [profile.default.rpc_storage_caching]
                chains = 'all'
                endpoints = 'all'

                [rpc_endpoints]
                optimism = "https://example.com/"
                mainnet = "${RPC_MAINNET}"
                mainnet_2 = "https://eth-mainnet.alchemyapi.io/v2/${API_KEY}"

                [fuzz]
                runs = 256
                seed = '0x3e8'
                max_test_rejects = 65536

                [invariant]
                runs = 256
                depth = 15
                fail_on_revert = false
                call_override = false
                shrink_sequence = true
            "#,
            )?;

            let config = Config::load_with_root(jail.directory());

            assert_eq!(config.fuzz.seed, Some(U256::from(1000)));
            assert_eq!(
                config.remappings,
                vec![Remapping::from_str("nested/=lib/nested/").unwrap().into()]
            );

            assert_eq!(
                config.rpc_endpoints,
                RpcEndpoints::new([
                    ("optimism", RpcEndpoint::Url("https://example.com/".to_string())),
                    ("mainnet", RpcEndpoint::Env("${RPC_MAINNET}".to_string())),
                    (
                        "mainnet_2",
                        RpcEndpoint::Env(
                            "https://eth-mainnet.alchemyapi.io/v2/${API_KEY}".to_string()
                        )
                    ),
                ]),
            );

            Ok(())
        });
    }

    #[test]
    fn test_solc_req() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                solc_version = "0.8.12"
            "#,
            )?;

            let config = Config::load();
            assert_eq!(config.solc, Some(SolcReq::Version("0.8.12".parse().unwrap())));

            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                solc = "0.8.12"
            "#,
            )?;

            let config = Config::load();
            assert_eq!(config.solc, Some(SolcReq::Version("0.8.12".parse().unwrap())));

            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                solc = "path/to/local/solc"
            "#,
            )?;

            let config = Config::load();
            assert_eq!(config.solc, Some(SolcReq::Local("path/to/local/solc".into())));

            jail.set_env("FOUNDRY_SOLC_VERSION", "0.6.6");
            let config = Config::load();
            assert_eq!(config.solc, Some(SolcReq::Version("0.6.6".parse().unwrap())));
            Ok(())
        });
    }

    // ensures the newer `solc` takes precedence over `solc_version`
    #[test]
    fn test_backwards_solc_version() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [default]
                solc = "0.8.12"
                solc_version = "0.8.20"
            "#,
            )?;

            let config = Config::load();
            assert_eq!(config.solc, Some(SolcReq::Version("0.8.12".parse().unwrap())));

            Ok(())
        });

        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [default]
                solc_version = "0.8.20"
            "#,
            )?;

            let config = Config::load();
            assert_eq!(config.solc, Some(SolcReq::Version("0.8.20".parse().unwrap())));

            Ok(())
        });
    }

    #[test]
    fn test_toml_casing_file() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                src = "some-source"
                out = "some-out"
                cache = true
                eth-rpc-url = "https://example.com/"
                evm-version = "berlin"
                auto-detect-solc = false
            "#,
            )?;

            let config = Config::load();
            assert_eq!(
                config,
                Config {
                    src: "some-source".into(),
                    out: "some-out".into(),
                    cache: true,
                    eth_rpc_url: Some("https://example.com/".to_string()),
                    auto_detect_solc: false,
                    evm_version: EvmVersion::Berlin,
                    ..Config::default()
                }
            );

            Ok(())
        });
    }

    #[test]
    fn test_output_selection() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                extra_output = ["metadata", "ir-optimized"]
                extra_output_files = ["metadata"]
            "#,
            )?;

            let config = Config::load();

            assert_eq!(
                config.extra_output,
                vec![ContractOutputSelection::Metadata, ContractOutputSelection::IrOptimized]
            );
            assert_eq!(config.extra_output_files, vec![ContractOutputSelection::Metadata]);

            Ok(())
        });
    }

    #[test]
    fn test_precedence() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                src = "mysrc"
                out = "myout"
                verbosity = 3
            "#,
            )?;

            let config = Config::load();
            assert_eq!(
                config,
                Config {
                    src: "mysrc".into(),
                    out: "myout".into(),
                    verbosity: 3,
                    ..Config::default()
                }
            );

            jail.set_env("FOUNDRY_SRC", r"other-src");
            let config = Config::load();
            assert_eq!(
                config,
                Config {
                    src: "other-src".into(),
                    out: "myout".into(),
                    verbosity: 3,
                    ..Config::default()
                }
            );

            jail.set_env("FOUNDRY_PROFILE", "foo");
            let val: Result<String, _> = Config::figment().extract_inner("profile");
            assert!(val.is_err());

            Ok(())
        });
    }

    #[test]
    fn test_extract_basic() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                src = "mysrc"
                out = "myout"
                verbosity = 3
                evm_version = 'berlin'

                [profile.other]
                src = "other-src"
            "#,
            )?;
            let loaded = Config::load();
            assert_eq!(loaded.evm_version, EvmVersion::Berlin);
            let base = loaded.into_basic();
            let default = Config::default();
            assert_eq!(
                base,
                BasicConfig {
                    profile: Config::DEFAULT_PROFILE,
                    src: "mysrc".into(),
                    out: "myout".into(),
                    libs: default.libs.clone(),
                    remappings: default.remappings.clone(),
                }
            );
            jail.set_env("FOUNDRY_PROFILE", r"other");
            let base = Config::figment().extract::<BasicConfig>().unwrap();
            assert_eq!(
                base,
                BasicConfig {
                    profile: Config::DEFAULT_PROFILE,
                    src: "other-src".into(),
                    out: "myout".into(),
                    libs: default.libs.clone(),
                    remappings: default.remappings,
                }
            );
            Ok(())
        });
    }

    #[test]
    #[should_panic]
    fn test_parse_invalid_fuzz_weight() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [fuzz]
                dictionary_weight = 101
            ",
            )?;
            let _config = Config::load();
            Ok(())
        });
    }

    #[test]
    fn test_fallback_provider() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [fuzz]
                runs = 1
                include_storage = false
                dictionary_weight = 99

                [invariant]
                runs = 420

                [profile.ci.fuzz]
                dictionary_weight = 5

                [profile.ci.invariant]
                runs = 400
            ",
            )?;

            let invariant_default = InvariantConfig::default();
            let config = Config::load();

            assert_ne!(config.invariant.runs, config.fuzz.runs);
            assert_eq!(config.invariant.runs, 420);

            assert_ne!(
                config.fuzz.dictionary.include_storage,
                invariant_default.dictionary.include_storage
            );
            assert_eq!(
                config.invariant.dictionary.include_storage,
                config.fuzz.dictionary.include_storage
            );

            assert_ne!(
                config.fuzz.dictionary.dictionary_weight,
                invariant_default.dictionary.dictionary_weight
            );
            assert_eq!(
                config.invariant.dictionary.dictionary_weight,
                config.fuzz.dictionary.dictionary_weight
            );

            jail.set_env("FOUNDRY_PROFILE", "ci");
            let ci_config = Config::load();
            assert_eq!(ci_config.fuzz.runs, 1);
            assert_eq!(ci_config.invariant.runs, 400);
            assert_eq!(ci_config.fuzz.dictionary.dictionary_weight, 5);
            assert_eq!(
                ci_config.invariant.dictionary.dictionary_weight,
                config.fuzz.dictionary.dictionary_weight
            );

            Ok(())
        })
    }

    #[test]
    fn test_standalone_profile_sections() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [fuzz]
                runs = 100

                [invariant]
                runs = 120

                [profile.ci.fuzz]
                runs = 420

                [profile.ci.invariant]
                runs = 500
            ",
            )?;

            let config = Config::load();
            assert_eq!(config.fuzz.runs, 100);
            assert_eq!(config.invariant.runs, 120);

            jail.set_env("FOUNDRY_PROFILE", "ci");
            let config = Config::load();
            assert_eq!(config.fuzz.runs, 420);
            assert_eq!(config.invariant.runs, 500);

            Ok(())
        });
    }

    #[test]
    fn can_handle_deviating_dapp_aliases() {
        figment::Jail::expect_with(|jail| {
            let addr = Address::random();
            jail.set_env("DAPP_TEST_NUMBER", 1337);
            jail.set_env("DAPP_TEST_ADDRESS", format!("{addr:?}"));
            jail.set_env("DAPP_TEST_FUZZ_RUNS", 420);
            jail.set_env("DAPP_TEST_DEPTH", 20);
            jail.set_env("DAPP_FORK_BLOCK", 100);
            jail.set_env("DAPP_BUILD_OPTIMIZE_RUNS", 999);
            jail.set_env("DAPP_BUILD_OPTIMIZE", 0);

            let config = Config::load();

            assert_eq!(config.block_number, 1337);
            assert_eq!(config.sender, addr);
            assert_eq!(config.fuzz.runs, 420);
            assert_eq!(config.invariant.depth, 20);
            assert_eq!(config.fork_block_number, Some(100));
            assert_eq!(config.optimizer_runs, 999);
            assert!(!config.optimizer);

            Ok(())
        });
    }

    #[test]
    fn can_parse_libraries() {
        figment::Jail::expect_with(|jail| {
            jail.set_env(
                "DAPP_LIBRARIES",
                "[src/DssSpell.sol:DssExecLib:0x8De6DDbCd5053d32292AAA0D2105A32d108484a6]",
            );
            let config = Config::load();
            assert_eq!(
                config.libraries,
                vec!["src/DssSpell.sol:DssExecLib:0x8De6DDbCd5053d32292AAA0D2105A32d108484a6"
                    .to_string()]
            );

            jail.set_env(
                "DAPP_LIBRARIES",
                "src/DssSpell.sol:DssExecLib:0x8De6DDbCd5053d32292AAA0D2105A32d108484a6",
            );
            let config = Config::load();
            assert_eq!(
                config.libraries,
                vec!["src/DssSpell.sol:DssExecLib:0x8De6DDbCd5053d32292AAA0D2105A32d108484a6"
                    .to_string(),]
            );

            jail.set_env(
                "DAPP_LIBRARIES",
                "src/DssSpell.sol:DssExecLib:0x8De6DDbCd5053d32292AAA0D2105A32d108484a6,src/DssSpell.sol:DssExecLib:0x8De6DDbCd5053d32292AAA0D2105A32d108484a6",
            );
            let config = Config::load();
            assert_eq!(
                config.libraries,
                vec![
                    "src/DssSpell.sol:DssExecLib:0x8De6DDbCd5053d32292AAA0D2105A32d108484a6"
                        .to_string(),
                    "src/DssSpell.sol:DssExecLib:0x8De6DDbCd5053d32292AAA0D2105A32d108484a6"
                        .to_string()
                ]
            );

            Ok(())
        });
    }

    #[test]
    fn test_parse_many_libraries() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [profile.default]
               libraries= [
                        './src/SizeAuctionDiscount.sol:Chainlink:0xffedba5e171c4f15abaaabc86e8bd01f9b54dae5',
                        './src/SizeAuction.sol:ChainlinkTWAP:0xffedba5e171c4f15abaaabc86e8bd01f9b54dae5',
                        './src/SizeAuction.sol:Math:0x902f6cf364b8d9470d5793a9b2b2e86bddd21e0c',
                        './src/test/ChainlinkTWAP.t.sol:ChainlinkTWAP:0xffedba5e171c4f15abaaabc86e8bd01f9b54dae5',
                        './src/SizeAuctionDiscount.sol:Math:0x902f6cf364b8d9470d5793a9b2b2e86bddd21e0c',
                    ]       
            ",
            )?;
            let config = Config::load();

            let libs = config.parsed_libraries().unwrap().libs;

            pretty_assertions::assert_eq!(
                libs,
                BTreeMap::from([
                    (
                        PathBuf::from("./src/SizeAuctionDiscount.sol"),
                        BTreeMap::from([
                            (
                                "Chainlink".to_string(),
                                "0xffedba5e171c4f15abaaabc86e8bd01f9b54dae5".to_string()
                            ),
                            (
                                "Math".to_string(),
                                "0x902f6cf364b8d9470d5793a9b2b2e86bddd21e0c".to_string()
                            )
                        ])
                    ),
                    (
                        PathBuf::from("./src/SizeAuction.sol"),
                        BTreeMap::from([
                            (
                                "ChainlinkTWAP".to_string(),
                                "0xffedba5e171c4f15abaaabc86e8bd01f9b54dae5".to_string()
                            ),
                            (
                                "Math".to_string(),
                                "0x902f6cf364b8d9470d5793a9b2b2e86bddd21e0c".to_string()
                            )
                        ])
                    ),
                    (
                        PathBuf::from("./src/test/ChainlinkTWAP.t.sol"),
                        BTreeMap::from([(
                            "ChainlinkTWAP".to_string(),
                            "0xffedba5e171c4f15abaaabc86e8bd01f9b54dae5".to_string()
                        )])
                    ),
                ])
            );

            Ok(())
        });
    }

    #[test]
    fn config_roundtrip() {
        figment::Jail::expect_with(|jail| {
            let default = Config::default();
            let basic = default.clone().into_basic();
            jail.create_file("foundry.toml", &basic.to_string_pretty().unwrap())?;

            let mut other = Config::load();
            clear_warning(&mut other);
            assert_eq!(default, other);

            let other = other.into_basic();
            assert_eq!(basic, other);

            jail.create_file("foundry.toml", &default.to_string_pretty().unwrap())?;
            let mut other = Config::load();
            clear_warning(&mut other);
            assert_eq!(default, other);

            Ok(())
        });
    }

    #[test]
    fn test_fs_permissions() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                fs_permissions = [{ access = "read-write", path = "./"}]
            "#,
            )?;
            let loaded = Config::load();

            assert_eq!(
                loaded.fs_permissions,
                FsPermissions::new(vec![PathPermission::read_write("./")])
            );

            jail.create_file(
                "foundry.toml",
                r#"
                [profile.default]
                fs_permissions = [{ access = "none", path = "./"}]
            "#,
            )?;
            let loaded = Config::load();
            assert_eq!(loaded.fs_permissions, FsPermissions::new(vec![PathPermission::none("./")]));

            Ok(())
        });
    }

    #[test]
    fn test_optimizer_settings_basic() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [profile.default]
                optimizer = true

                [profile.default.optimizer_details]
                yul = false

                [profile.default.optimizer_details.yulDetails]
                stackAllocation = true
            ",
            )?;
            let mut loaded = Config::load();
            clear_warning(&mut loaded);
            assert_eq!(
                loaded.optimizer_details,
                Some(OptimizerDetails {
                    yul: Some(false),
                    yul_details: Some(YulDetails {
                        stack_allocation: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            );

            let s = loaded.to_string_pretty().unwrap();
            jail.create_file("foundry.toml", &s)?;

            let mut reloaded = Config::load();
            clear_warning(&mut reloaded);
            assert_eq!(loaded, reloaded);

            Ok(())
        });
    }

    #[test]
    fn test_model_checker_settings_basic() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [profile.default]

                [profile.default.model_checker]
                contracts = { 'a.sol' = [ 'A1', 'A2' ], 'b.sol' = [ 'B1', 'B2' ] }
                engine = 'chc'
                targets = [ 'assert', 'outOfBounds' ]
                timeout = 10000
            ",
            )?;
            let mut loaded = Config::load();
            clear_warning(&mut loaded);
            assert_eq!(
                loaded.model_checker,
                Some(ModelCheckerSettings {
                    contracts: BTreeMap::from([
                        ("a.sol".to_string(), vec!["A1".to_string(), "A2".to_string()]),
                        ("b.sol".to_string(), vec!["B1".to_string(), "B2".to_string()]),
                    ]),
                    engine: Some(ModelCheckerEngine::CHC),
                    targets: Some(vec![
                        ModelCheckerTarget::Assert,
                        ModelCheckerTarget::OutOfBounds
                    ]),
                    timeout: Some(10000),
                    invariants: None,
                    show_unproved: None,
                    div_mod_with_slacks: None,
                    solvers: None,
                    show_unsupported: None,
                    show_proved_safe: None,
                })
            );

            let s = loaded.to_string_pretty().unwrap();
            jail.create_file("foundry.toml", &s)?;

            let mut reloaded = Config::load();
            clear_warning(&mut reloaded);
            assert_eq!(loaded, reloaded);

            Ok(())
        });
    }

    #[test]
    fn test_model_checker_settings_relative_paths() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [profile.default]

                [profile.default.model_checker]
                contracts = { 'a.sol' = [ 'A1', 'A2' ], 'b.sol' = [ 'B1', 'B2' ] }
                engine = 'chc'
                targets = [ 'assert', 'outOfBounds' ]
                timeout = 10000
            ",
            )?;
            let loaded = Config::load().sanitized();

            // NOTE(onbjerg): We have to canonicalize the path here using dunce because figment will
            // canonicalize the jail path using the standard library. The standard library *always*
            // transforms Windows paths to some weird extended format, which none of our code base
            // does.
            let dir = foundry_compilers::utils::canonicalize(jail.directory())
                .expect("Could not canonicalize jail path");
            assert_eq!(
                loaded.model_checker,
                Some(ModelCheckerSettings {
                    contracts: BTreeMap::from([
                        (
                            format!("{}", dir.join("a.sol").display()),
                            vec!["A1".to_string(), "A2".to_string()]
                        ),
                        (
                            format!("{}", dir.join("b.sol").display()),
                            vec!["B1".to_string(), "B2".to_string()]
                        ),
                    ]),
                    engine: Some(ModelCheckerEngine::CHC),
                    targets: Some(vec![
                        ModelCheckerTarget::Assert,
                        ModelCheckerTarget::OutOfBounds
                    ]),
                    timeout: Some(10000),
                    invariants: None,
                    show_unproved: None,
                    div_mod_with_slacks: None,
                    solvers: None,
                    show_unsupported: None,
                    show_proved_safe: None,
                })
            );

            Ok(())
        });
    }

    #[test]
    fn test_fmt_config() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [fmt]
                line_length = 100
                tab_width = 2
                bracket_spacing = true
            ",
            )?;
            let loaded = Config::load().sanitized();
            assert_eq!(
                loaded.fmt,
                FormatterConfig {
                    line_length: 100,
                    tab_width: 2,
                    bracket_spacing: true,
                    ..Default::default()
                }
            );

            Ok(())
        });
    }

    #[test]
    fn test_invariant_config() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [invariant]
                runs = 512
                depth = 10
            ",
            )?;

            let loaded = Config::load().sanitized();
            assert_eq!(
                loaded.invariant,
                InvariantConfig { runs: 512, depth: 10, ..Default::default() }
            );

            Ok(())
        });
    }

    #[test]
    fn test_standalone_sections_env() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [fuzz]
                runs = 100

                [invariant]
                depth = 1
            ",
            )?;

            jail.set_env("FOUNDRY_FMT_LINE_LENGTH", "95");
            jail.set_env("FOUNDRY_FUZZ_DICTIONARY_WEIGHT", "99");
            jail.set_env("FOUNDRY_INVARIANT_DEPTH", "5");

            let config = Config::load();
            assert_eq!(config.fmt.line_length, 95);
            assert_eq!(config.fuzz.dictionary.dictionary_weight, 99);
            assert_eq!(config.invariant.depth, 5);

            Ok(())
        });
    }

    #[test]
    fn test_parse_with_profile() {
        let foundry_str = r"
            [profile.default]
            src = 'src'
            out = 'out'
            libs = ['lib']

            # See more config options https://github.com/foundry-rs/foundry/blob/master/crates/config/README.md#all-options
        ";
        assert_eq!(
            parse_with_profile::<BasicConfig>(foundry_str).unwrap().unwrap(),
            (
                Config::DEFAULT_PROFILE,
                BasicConfig {
                    profile: Config::DEFAULT_PROFILE,
                    src: "src".into(),
                    out: "out".into(),
                    libs: vec!["lib".into()],
                    remappings: vec![]
                }
            )
        );
    }

    #[test]
    fn test_implicit_profile_loads() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [default]
                src = 'my-src'
                out = 'my-out'
            ",
            )?;
            let loaded = Config::load().sanitized();
            assert_eq!(loaded.src.file_name().unwrap(), "my-src");
            assert_eq!(loaded.out.file_name().unwrap(), "my-out");
            assert_eq!(
                loaded.__warnings,
                vec![Warning::UnknownSection {
                    unknown_section: Profile::new("default"),
                    source: Some("foundry.toml".into())
                }]
            );

            Ok(())
        });
    }

    #[test]
    fn test_eth_rpc_url_env_var() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [default]
            ",
            )?;
            jail.set_env("ETH_RPC_URL", "https://example.com/");
            let loaded = Config::load();
            assert_eq!(loaded.eth_rpc_url, Some("https://example.com/".to_string()));

            Ok(())
        });
    }

    // a test to print the config, mainly used to update the example config in the README
    #[test]
    #[ignore]
    fn print_config() {
        let config = Config {
            optimizer_details: Some(OptimizerDetails {
                peephole: None,
                inliner: None,
                jumpdest_remover: None,
                order_literals: None,
                deduplicate: None,
                cse: None,
                constant_optimizer: Some(true),
                yul: Some(true),
                yul_details: Some(YulDetails {
                    stack_allocation: None,
                    optimizer_steps: Some("dhfoDgvulfnTUtnIf".to_string()),
                }),
            }),
            ..Default::default()
        };
        println!("{}", config.to_string_pretty().unwrap());
    }

    #[test]
    fn can_use_impl_figment_macro() {
        #[derive(Default, Serialize)]
        struct MyArgs {
            #[serde(skip_serializing_if = "Option::is_none")]
            root: Option<PathBuf>,
        }
        impl_figment_convert!(MyArgs);

        impl Provider for MyArgs {
            fn metadata(&self) -> Metadata {
                Metadata::default()
            }

            fn data(&self) -> Result<Map<Profile, Dict>, Error> {
                let value = Value::serialize(self)?;
                let error = InvalidType(value.to_actual(), "map".into());
                let dict = value.into_dict().ok_or(error)?;
                Ok(Map::from([(Config::selected_profile(), dict)]))
            }
        }

        let _figment: Figment = From::from(&MyArgs::default());
        let _config: Config = From::from(&MyArgs::default());

        #[derive(Default)]
        struct Outer {
            start: MyArgs,
            other: MyArgs,
            another: MyArgs,
        }
        impl_figment_convert!(Outer, start, other, another);

        let _figment: Figment = From::from(&Outer::default());
        let _config: Config = From::from(&Outer::default());
    }

    #[test]
    fn list_cached_blocks() -> eyre::Result<()> {
        fn fake_block_cache(chain_path: &Path, block_number: &str, size_bytes: usize) {
            let block_path = chain_path.join(block_number);
            fs::create_dir(block_path.as_path()).unwrap();
            let file_path = block_path.join("storage.json");
            let mut file = File::create(file_path).unwrap();
            writeln!(file, "{}", vec![' '; size_bytes - 1].iter().collect::<String>()).unwrap();
        }

        let chain_dir = tempdir()?;

        fake_block_cache(chain_dir.path(), "1", 100);
        fake_block_cache(chain_dir.path(), "2", 500);
        // Pollution file that should not show up in the cached block
        let mut pol_file = File::create(chain_dir.path().join("pol.txt")).unwrap();
        writeln!(pol_file, "{}", [' '; 10].iter().collect::<String>()).unwrap();

        let result = Config::get_cached_blocks(chain_dir.path())?;

        assert_eq!(result.len(), 2);
        let block1 = &result.iter().find(|x| x.0 == "1").unwrap();
        let block2 = &result.iter().find(|x| x.0 == "2").unwrap();
        assert_eq!(block1.0, "1");
        assert_eq!(block1.1, 100);
        assert_eq!(block2.0, "2");
        assert_eq!(block2.1, 500);

        chain_dir.close()?;
        Ok(())
    }

    #[test]
    fn list_etherscan_cache() -> eyre::Result<()> {
        fn fake_etherscan_cache(chain_path: &Path, address: &str, size_bytes: usize) {
            let metadata_path = chain_path.join("sources");
            let abi_path = chain_path.join("abi");
            let _ = fs::create_dir(metadata_path.as_path());
            let _ = fs::create_dir(abi_path.as_path());

            let metadata_file_path = metadata_path.join(address);
            let mut metadata_file = File::create(metadata_file_path).unwrap();
            writeln!(metadata_file, "{}", vec![' '; size_bytes / 2 - 1].iter().collect::<String>())
                .unwrap();

            let abi_file_path = abi_path.join(address);
            let mut abi_file = File::create(abi_file_path).unwrap();
            writeln!(abi_file, "{}", vec![' '; size_bytes / 2 - 1].iter().collect::<String>())
                .unwrap();
        }

        let chain_dir = tempdir()?;

        fake_etherscan_cache(chain_dir.path(), "1", 100);
        fake_etherscan_cache(chain_dir.path(), "2", 500);

        let result = Config::get_cached_block_explorer_data(chain_dir.path())?;

        assert_eq!(result, 600);

        chain_dir.close()?;
        Ok(())
    }

    #[test]
    fn test_parse_error_codes() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [default]
                ignored_error_codes = ["license", "unreachable", 1337]
            "#,
            )?;

            let config = Config::load();
            assert_eq!(
                config.ignored_error_codes,
                vec![
                    SolidityErrorCode::SpdxLicenseNotProvided,
                    SolidityErrorCode::Unreachable,
                    SolidityErrorCode::Other(1337)
                ]
            );

            Ok(())
        });
    }

    #[test]
    fn test_parse_optimizer_settings() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r"
                [default]
                [profile.default.optimizer_details]
            ",
            )?;

            let config = Config::load();
            assert_eq!(config.optimizer_details, Some(OptimizerDetails::default()));

            Ok(())
        });
    }

    #[test]
    fn test_parse_labels() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                "foundry.toml",
                r#"
                [labels]
                0x1F98431c8aD98523631AE4a59f267346ea31F984 = "Uniswap V3: Factory"
                0xC36442b4a4522E871399CD717aBDD847Ab11FE88 = "Uniswap V3: Positions NFT"
            "#,
            )?;

            let config = Config::load();
            assert_eq!(
                config.labels,
                HashMap::from_iter(vec![
                    (
                        Address::from_str("0x1F98431c8aD98523631AE4a59f267346ea31F984").unwrap(),
                        "Uniswap V3: Factory".to_string()
                    ),
                    (
                        Address::from_str("0xC36442b4a4522E871399CD717aBDD847Ab11FE88").unwrap(),
                        "Uniswap V3: Positions NFT".to_string()
                    ),
                ])
            );

            Ok(())
        });
    }
}
