//! CLI arguments for configuring the EVM settings.

use alloy_primitives::{map::HashMap, Address, B256, U256};
use clap::Parser;
use eyre::ContextCompat;
use foundry_config::{
    figment::{
        self,
        error::Kind::InvalidType,
        value::{Dict, Map, Value},
        Metadata, Profile, Provider,
    },
    Chain, Config,
};
use serde::Serialize;

use crate::shell;

/// Map keyed by breakpoints char to their location (contract address, pc)
pub type Breakpoints = HashMap<char, (Address, usize)>;

/// `EvmArgs` and `EnvArgs` take the highest precedence in the Config/Figment hierarchy.
///
/// All vars are opt-in, their default values are expected to be set by the
/// [`foundry_config::Config`], and are always present ([`foundry_config::Config::default`])
///
/// Both have corresponding types in the `evm_adapters` crate which have mandatory fields.
/// The expected workflow is
///   1. load the [`foundry_config::Config`]
///   2. merge with `EvmArgs` into a `figment::Figment`
///   3. extract `evm_adapters::Opts` from the merged `Figment`
///
/// # Example
///
/// ```ignore
/// use foundry_config::Config;
/// use forge::executor::opts::EvmOpts;
/// use foundry_common::evm::EvmArgs;
/// # fn t(args: EvmArgs) {
/// let figment = Config::figment_with_root(".").merge(args);
/// let opts = figment.extract::<EvmOpts>().unwrap();
/// # }
/// ```
#[derive(Clone, Debug, Default, Serialize, Parser)]
#[command(next_help_heading = "EVM options", about = None, long_about = None)] // override doc
pub struct EvmArgs {
    /// Fetch state over a remote endpoint instead of starting from an empty state.
    ///
    /// If you want to fetch state from a specific block number, see --fork-block-number.
    #[arg(long, short, visible_alias = "rpc-url", env = "ETH_RPC_URL", value_name = "URL")]
    #[serde(rename = "eth_rpc_url", skip_serializing_if = "Option::is_none")]
    pub fork_url: Option<String>,

    /// Fetch state from a specific block number over a remote endpoint.
    ///
    /// See --fork-url.
    #[arg(long, requires = "fork_url", value_name = "BLOCK")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fork_block_number: Option<u64>,

    /// Number of retries.
    ///
    /// See --fork-url.
    #[arg(long, requires = "fork_url", value_name = "RETRIES")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fork_retries: Option<u32>,

    /// Initial retry backoff on encountering errors.
    ///
    /// See --fork-url.
    #[arg(long, requires = "fork_url", value_name = "BACKOFF")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fork_retry_backoff: Option<u64>,

    /// Explicitly disables the use of RPC caching.
    ///
    /// All storage slots are read entirely from the endpoint.
    ///
    /// This flag overrides the project's configuration file.
    ///
    /// See --fork-url.
    #[arg(long)]
    #[serde(skip)]
    pub no_storage_caching: bool,

    /// The initial balance of deployed test contracts.
    #[arg(long, value_name = "BALANCE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_balance: Option<U256>,

    /// The address which will be executing tests/scripts.
    #[arg(long, value_name = "ADDRESS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<Address>,

    /// Enable the FFI cheatcode.
    #[arg(long)]
    #[serde(skip)]
    pub ffi: bool,

    /// Use the create 2 factory in all cases including tests and non-broadcasting scripts.
    #[arg(long)]
    #[serde(skip)]
    pub always_use_create_2_factory: bool,

    /// The CREATE2 deployer address to use, this will override the one in the config.
    #[arg(long, value_name = "ADDRESS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create2_deployer: Option<Address>,

    /// Sets the number of assumed available compute units per second for this provider
    ///
    /// default value: 330
    ///
    /// See also --fork-url and <https://docs.alchemy.com/reference/compute-units#what-are-cups-compute-units-per-second>
    #[arg(long, alias = "cups", value_name = "CUPS", help_heading = "Fork config")]
    pub compute_units_per_second: Option<u64>,

    /// Disables rate limiting for this node's provider.
    ///
    /// See also --fork-url and <https://docs.alchemy.com/reference/compute-units#what-are-cups-compute-units-per-second>
    #[arg(
        long,
        value_name = "NO_RATE_LIMITS",
        help_heading = "Fork config",
        visible_alias = "no-rate-limit"
    )]
    #[serde(skip)]
    pub no_rpc_rate_limit: bool,

    /// All ethereum environment related arguments
    #[command(flatten)]
    #[serde(flatten)]
    pub env: EnvArgs,

    /// Whether to enable isolation of calls.
    /// In isolation mode all top-level calls are executed as a separate transaction in a separate
    /// EVM context, enabling more precise gas accounting and transaction state changes.
    #[arg(long)]
    #[serde(skip)]
    pub isolate: bool,

    /// Whether to enable Odyssey features.
    #[arg(long, alias = "alphanet")]
    #[serde(skip)]
    pub odyssey: bool,
}

// Make this set of options a `figment::Provider` so that it can be merged into the `Config`
impl Provider for EvmArgs {
    fn metadata(&self) -> Metadata {
        Metadata::named("Evm Opts Provider")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        let value = Value::serialize(self)?;
        let error = InvalidType(value.to_actual(), "map".into());
        let mut dict = value.into_dict().ok_or(error)?;

        if shell::verbosity() > 0 {
            // need to merge that manually otherwise `from_occurrences` does not work
            dict.insert("verbosity".to_string(), shell::verbosity().into());
        }

        if self.ffi {
            dict.insert("ffi".to_string(), self.ffi.into());
        }

        if self.isolate {
            dict.insert("isolate".to_string(), self.isolate.into());
        }

        if self.odyssey {
            dict.insert("odyssey".to_string(), self.odyssey.into());
        }

        if self.always_use_create_2_factory {
            dict.insert(
                "always_use_create_2_factory".to_string(),
                self.always_use_create_2_factory.into(),
            );
        }

        if self.no_storage_caching {
            dict.insert("no_storage_caching".to_string(), self.no_storage_caching.into());
        }

        if self.no_rpc_rate_limit {
            dict.insert("no_rpc_rate_limit".to_string(), self.no_rpc_rate_limit.into());
        }

        if let Some(fork_url) = &self.fork_url {
            dict.insert("eth_rpc_url".to_string(), fork_url.clone().into());
        }

        Ok(Map::from([(Config::selected_profile(), dict)]))
    }
}

/// Configures the executor environment during tests.
#[derive(Clone, Debug, Default, Serialize, Parser)]
#[command(next_help_heading = "Executor environment config")]
pub struct EnvArgs {
    /// EIP-170: Contract code size limit in bytes. Useful to increase this because of tests. By
    /// default, it is 0x6000 (~25kb).
    #[arg(long, value_name = "CODE_SIZE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_size_limit: Option<usize>,

    /// The chain name or EIP-155 chain ID.
    #[arg(long, visible_alias = "chain-id", value_name = "CHAIN")]
    #[serde(rename = "chain_id", skip_serializing_if = "Option::is_none", serialize_with = "id")]
    pub chain: Option<Chain>,

    /// The gas price.
    #[arg(long, value_name = "GAS_PRICE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<u64>,

    /// The base fee in a block.
    #[arg(long, visible_alias = "base-fee", value_name = "FEE")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_base_fee_per_gas: Option<u64>,

    /// The transaction origin.
    #[arg(long, value_name = "ADDRESS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_origin: Option<Address>,

    /// The coinbase of the block.
    #[arg(long, value_name = "ADDRESS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_coinbase: Option<Address>,

    /// The timestamp of the block.
    #[arg(long, value_name = "TIMESTAMP")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_timestamp: Option<u64>,

    /// The block number.
    #[arg(long, value_name = "BLOCK")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,

    /// The block difficulty.
    #[arg(long, value_name = "DIFFICULTY")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_difficulty: Option<u64>,

    /// The block prevrandao value. NOTE: Before merge this field was mix_hash.
    #[arg(long, value_name = "PREVRANDAO")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_prevrandao: Option<B256>,

    /// The block gas limit.
    #[arg(long, visible_alias = "gas-limit", value_name = "GAS_LIMIT")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_gas_limit: Option<u64>,

    /// The memory limit per EVM execution in bytes.
    /// If this limit is exceeded, a `MemoryLimitOOG` result is thrown.
    ///
    /// The default is 128MiB.
    #[arg(long, value_name = "MEMORY_LIMIT")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit: Option<u64>,

    /// Whether to disable the block gas limit checks.
    #[arg(long, visible_alias = "no-gas-limit")]
    pub disable_block_gas_limit: bool,
}

impl EvmArgs {
    /// Ensures that fork url exists and returns its reference.
    pub fn ensure_fork_url(&self) -> eyre::Result<&String> {
        self.fork_url.as_ref().wrap_err("Missing `--fork-url` field.")
    }
}

/// We have to serialize chain IDs and not names because when extracting an EVM `Env`, it expects
/// `chain_id` to be `u64`.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn id<S: serde::Serializer>(chain: &Option<Chain>, s: S) -> Result<S::Ok, S::Error> {
    if let Some(chain) = chain {
        s.serialize_u64(chain.id())
    } else {
        // skip_serializing_if = "Option::is_none" should prevent this branch from being taken
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundry_config::NamedChain;

    #[test]
    fn can_parse_chain_id() {
        let args = EvmArgs {
            env: EnvArgs { chain: Some(NamedChain::Mainnet.into()), ..Default::default() },
            ..Default::default()
        };
        let config = Config::from_provider(Config::figment().merge(args));
        assert_eq!(config.chain, Some(NamedChain::Mainnet.into()));

        let env = EnvArgs::parse_from(["foundry-common", "--chain-id", "goerli"]);
        assert_eq!(env.chain, Some(NamedChain::Goerli.into()));
    }

    #[test]
    fn test_memory_limit() {
        let args = EvmArgs {
            env: EnvArgs { chain: Some(NamedChain::Mainnet.into()), ..Default::default() },
            ..Default::default()
        };
        let config = Config::from_provider(Config::figment().merge(args));
        assert_eq!(config.memory_limit, Config::default().memory_limit);

        let env = EnvArgs::parse_from(["foundry-common", "--memory-limit", "100"]);
        assert_eq!(env.memory_limit, Some(100));
    }

    #[test]
    fn test_chain_id() {
        let env = EnvArgs::parse_from(["foundry-common", "--chain-id", "1"]);
        assert_eq!(env.chain, Some(Chain::mainnet()));

        let env = EnvArgs::parse_from(["foundry-common", "--chain-id", "mainnet"]);
        assert_eq!(env.chain, Some(Chain::mainnet()));
        let args = EvmArgs { env, ..Default::default() };
        let config = Config::from_provider(Config::figment().merge(args));
        assert_eq!(config.chain, Some(Chain::mainnet()));
    }
}
