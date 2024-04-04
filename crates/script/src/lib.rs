#![cfg_attr(not(test), warn(unused_crate_dependencies))]

#[macro_use]
extern crate tracing;

use self::transaction::AdditionalContract;
use crate::runner::ScriptRunner;
use alloy_json_abi::{Function, JsonAbi};
use alloy_primitives::{Address, Bytes, Log, U256};
use broadcast::next_nonce;
use build::PreprocessedState;
use clap::{Parser, ValueHint};
use dialoguer::Confirm;
use eyre::{ContextCompat, Result};
use forge_verify::RetryArgs;
use foundry_cli::{opts::CoreBuildArgs, utils::LoadConfig};
use foundry_common::{
    abi::{encode_function_args, get_func},
    compile::SkipBuildFilter,
    errors::UnlinkedByteCode,
    evm::{Breakpoints, EvmArgs},
    provider::ethers::RpcUrl,
    shell, CONTRACT_MAX_SIZE, SELECTOR_LEN,
};
use foundry_compilers::{artifacts::ContractBytecodeSome, ArtifactId};
use foundry_config::{
    figment,
    figment::{
        value::{Dict, Map},
        Metadata, Profile, Provider,
    },
    Config,
};
use foundry_evm::{
    backend::Backend,
    constants::DEFAULT_CREATE2_DEPLOYER,
    debug::DebugArena,
    executors::ExecutorBuilder,
    inspectors::{
        cheatcodes::{BroadcastableTransactions, ScriptWallets},
        CheatsConfig,
    },
    opts::EvmOpts,
    traces::Traces,
};
use foundry_wallets::MultiWalletOpts;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use yansi::Paint;

mod artifacts;
mod broadcast;
mod build;
mod execute;
mod multi_sequence;
mod providers;
mod receipts;
mod runner;
mod sequence;
mod simulate;
mod transaction;
mod verify;

// Loads project's figment and merges the build cli arguments into it
foundry_config::merge_impl_figment_convert!(ScriptArgs, opts, evm_opts);

/// CLI arguments for `forge script`.
#[derive(Clone, Debug, Default, Parser)]
pub struct ScriptArgs {
    /// The contract you want to run. Either the file path or contract name.
    ///
    /// If multiple contracts exist in the same file you must specify the target contract with
    /// --target-contract.
    #[arg(value_hint = ValueHint::FilePath)]
    pub path: String,

    /// Arguments to pass to the script function.
    pub args: Vec<String>,

    /// The name of the contract you want to run.
    #[arg(long, visible_alias = "tc", value_name = "CONTRACT_NAME")]
    pub target_contract: Option<String>,

    /// The signature of the function you want to call in the contract, or raw calldata.
    #[arg(long, short, default_value = "run()")]
    pub sig: String,

    /// Max priority fee per gas for EIP1559 transactions.
    #[arg(
        long,
        env = "ETH_PRIORITY_GAS_PRICE",
        value_parser = foundry_cli::utils::parse_ether_value,
        value_name = "PRICE"
    )]
    pub priority_gas_price: Option<U256>,

    /// Use legacy transactions instead of EIP1559 ones.
    ///
    /// This is auto-enabled for common networks without EIP1559.
    #[arg(long)]
    pub legacy: bool,

    /// Broadcasts the transactions.
    #[arg(long)]
    pub broadcast: bool,

    /// Skips on-chain simulation.
    #[arg(long)]
    pub skip_simulation: bool,

    /// Relative percentage to multiply gas estimates by.
    #[arg(long, short, default_value = "130")]
    pub gas_estimate_multiplier: u64,

    /// Send via `eth_sendTransaction` using the `--from` argument or `$ETH_FROM` as sender
    #[arg(
        long,
        requires = "sender",
        conflicts_with_all = &["private_key", "private_keys", "froms", "ledger", "trezor", "aws"],
    )]
    pub unlocked: bool,

    /// Resumes submitting transactions that failed or timed-out previously.
    ///
    /// It DOES NOT simulate the script again and it expects nonces to have remained the same.
    ///
    /// Example: If transaction N has a nonce of 22, then the account should have a nonce of 22,
    /// otherwise it fails.
    #[arg(long)]
    pub resume: bool,

    /// If present, --resume or --verify will be assumed to be a multi chain deployment.
    #[arg(long)]
    pub multi: bool,

    /// Open the script in the debugger.
    ///
    /// Takes precedence over broadcast.
    #[arg(long)]
    pub debug: bool,

    /// Makes sure a transaction is sent,
    /// only after its previous one has been confirmed and succeeded.
    #[arg(long)]
    pub slow: bool,

    /// Disables interactive prompts that might appear when deploying big contracts.
    ///
    /// For more info on the contract size limit, see EIP-170: <https://eips.ethereum.org/EIPS/eip-170>
    #[arg(long)]
    pub non_interactive: bool,

    /// The Etherscan (or equivalent) API key
    #[arg(long, env = "ETHERSCAN_API_KEY", value_name = "KEY")]
    pub etherscan_api_key: Option<String>,

    /// Verifies all the contracts found in the receipts of a script, if any.
    #[arg(long)]
    pub verify: bool,

    /// Output results in JSON format.
    #[arg(long)]
    pub json: bool,

    /// Gas price for legacy transactions, or max fee per gas for EIP1559 transactions.
    #[arg(
        long,
        env = "ETH_GAS_PRICE",
        value_parser = foundry_cli::utils::parse_ether_value,
        value_name = "PRICE",
    )]
    pub with_gas_price: Option<U256>,

    /// Skip building files whose names contain the given filter.
    ///
    /// `test` and `script` are aliases for `.t.sol` and `.s.sol`.
    #[arg(long, num_args(1..))]
    pub skip: Option<Vec<SkipBuildFilter>>,

    #[command(flatten)]
    pub opts: CoreBuildArgs,

    #[command(flatten)]
    pub wallets: MultiWalletOpts,

    #[command(flatten)]
    pub evm_opts: EvmArgs,

    #[command(flatten)]
    pub verifier: forge_verify::VerifierArgs,

    #[command(flatten)]
    pub retry: RetryArgs,
}

// === impl ScriptArgs ===

impl ScriptArgs {
    async fn preprocess(self) -> Result<PreprocessedState> {
        let script_wallets =
            ScriptWallets::new(self.wallets.get_multi_wallet().await?, self.evm_opts.sender);

        let (config, evm_opts) = self.load_config_and_evm_opts_emit_warnings()?;

        let script_config = ScriptConfig::new(config, evm_opts).await?;

        Ok(PreprocessedState { args: self, script_config, script_wallets })
    }

    /// Executes the script
    pub async fn run_script(self) -> Result<()> {
        trace!(target: "script", "executing script command");

        let compiled = self.preprocess().await?.compile()?;

        // Move from `CompiledState` to `BundledState` either by resuming or executing and
        // simulating script.
        let bundled = if compiled.args.resume || (compiled.args.verify && !compiled.args.broadcast)
        {
            compiled.resume().await?
        } else {
            // Drive state machine to point at which we have everything needed for simulation.
            let pre_simulation = compiled
                .link()?
                .prepare_execution()
                .await?
                .execute()
                .await?
                .prepare_simulation()
                .await?;

            if pre_simulation.args.debug {
                pre_simulation.run_debugger()?;
            }

            if pre_simulation.args.json {
                pre_simulation.show_json()?;
            } else {
                pre_simulation.show_traces().await?;
            }

            // Ensure that we have transactions to simulate/broadcast, otherwise exit early to avoid
            // hard error.
            if pre_simulation
                .execution_result
                .transactions
                .as_ref()
                .map_or(true, |txs| txs.is_empty())
            {
                return Ok(());
            }

            // Check if there are any missing RPCs and exit early to avoid hard error.
            if pre_simulation.execution_artifacts.rpc_data.missing_rpc {
                shell::println("\nIf you wish to simulate on-chain transactions pass a RPC URL.")?;
                return Ok(());
            }

            pre_simulation.args.check_contract_sizes(
                &pre_simulation.execution_result,
                &pre_simulation.build_data.highlevel_known_contracts,
            )?;

            pre_simulation.fill_metadata().await?.bundle().await?
        };

        // Exit early in case user didn't provide any broadcast/verify related flags.
        if !bundled.args.broadcast && !bundled.args.resume && !bundled.args.verify {
            shell::println("\nSIMULATION COMPLETE. To broadcast these transactions, add --broadcast and wallet configuration(s) to the previous command. See forge script --help for more.")?;
            return Ok(());
        }

        // Exit early if something is wrong with verification options.
        if bundled.args.verify {
            bundled.verify_preflight_check()?;
        }

        // Wait for pending txes and broadcast others.
        let broadcasted = bundled.wait_for_pending().await?.broadcast().await?;

        if broadcasted.args.verify {
            broadcasted.verify().await?;
        }

        Ok(())
    }

    /// Returns the Function and calldata based on the signature
    ///
    /// If the `sig` is a valid human-readable function we find the corresponding function in the
    /// `abi` If the `sig` is valid hex, we assume it's calldata and try to find the
    /// corresponding function by matching the selector, first 4 bytes in the calldata.
    ///
    /// Note: We assume that the `sig` is already stripped of its prefix, See [`ScriptArgs`]
    fn get_method_and_calldata(&self, abi: &JsonAbi) -> Result<(Function, Bytes)> {
        if let Ok(decoded) = hex::decode(&self.sig) {
            let selector = &decoded[..SELECTOR_LEN];
            let func =
                abi.functions().find(|func| selector == &func.selector()[..]).ok_or_else(|| {
                    eyre::eyre!(
                        "Function selector `{}` not found in the ABI",
                        hex::encode(selector)
                    )
                })?;
            return Ok((func.clone(), decoded.into()));
        }

        let func = if self.sig.contains('(') {
            let func = get_func(&self.sig)?;
            abi.functions()
                .find(|&abi_func| abi_func.selector() == func.selector())
                .wrap_err(format!("Function `{}` is not implemented in your script.", self.sig))?
        } else {
            let matching_functions =
                abi.functions().filter(|func| func.name == self.sig).collect::<Vec<_>>();
            match matching_functions.len() {
                0 => eyre::bail!("Function `{}` not found in the ABI", self.sig),
                1 => matching_functions[0],
                2.. => eyre::bail!(
                    "Multiple functions with the same name `{}` found in the ABI",
                    self.sig
                ),
            }
        };
        let data = encode_function_args(func, &self.args)?;

        Ok((func.clone(), data.into()))
    }

    /// Checks if the transaction is a deployment with either a size above the `CONTRACT_MAX_SIZE`
    /// or specified `code_size_limit`.
    ///
    /// If `self.broadcast` is enabled, it asks confirmation of the user. Otherwise, it just warns
    /// the user.
    fn check_contract_sizes(
        &self,
        result: &ScriptResult,
        known_contracts: &BTreeMap<ArtifactId, ContractBytecodeSome>,
    ) -> Result<()> {
        // (name, &init, &deployed)[]
        let mut bytecodes: Vec<(String, &[u8], &[u8])> = vec![];

        // From artifacts
        for (artifact, bytecode) in known_contracts.iter() {
            if bytecode.bytecode.object.is_unlinked() {
                return Err(UnlinkedByteCode::Bytecode(artifact.identifier()).into());
            }
            let init_code = bytecode.bytecode.object.as_bytes().unwrap();
            // Ignore abstract contracts
            if let Some(ref deployed_code) = bytecode.deployed_bytecode.bytecode {
                if deployed_code.object.is_unlinked() {
                    return Err(UnlinkedByteCode::DeployedBytecode(artifact.identifier()).into());
                }
                let deployed_code = deployed_code.object.as_bytes().unwrap();
                bytecodes.push((artifact.name.clone(), init_code, deployed_code));
            }
        }

        // From traces
        let create_nodes = result.traces.iter().flat_map(|(_, traces)| {
            traces.nodes().iter().filter(|node| node.trace.kind.is_any_create())
        });
        let mut unknown_c = 0usize;
        for node in create_nodes {
            let init_code = &node.trace.data;
            let deployed_code = &node.trace.output;
            if !bytecodes.iter().any(|(_, b, _)| *b == init_code.as_ref()) {
                bytecodes.push((format!("Unknown{unknown_c}"), init_code, deployed_code));
                unknown_c += 1;
            }
            continue;
        }

        let mut prompt_user = false;
        let max_size = match self.evm_opts.env.code_size_limit {
            Some(size) => size,
            None => CONTRACT_MAX_SIZE,
        };

        for (data, to) in result.transactions.iter().flat_map(|txes| {
            txes.iter().filter_map(|tx| {
                tx.transaction
                    .input
                    .clone()
                    .into_input()
                    .filter(|data| data.len() > max_size)
                    .map(|data| (data, tx.transaction.to))
            })
        }) {
            let mut offset = 0;

            // Find if it's a CREATE or CREATE2. Otherwise, skip transaction.
            if let Some(to) = to {
                if to == DEFAULT_CREATE2_DEPLOYER {
                    // Size of the salt prefix.
                    offset = 32;
                }
            } else if to.is_some() {
                continue;
            }

            // Find artifact with a deployment code same as the data.
            if let Some((name, _, deployed_code)) =
                bytecodes.iter().find(|(_, init_code, _)| *init_code == &data[offset..])
            {
                let deployment_size = deployed_code.len();

                if deployment_size > max_size {
                    prompt_user = self.broadcast;
                    shell::println(format!(
                        "{}",
                        Paint::red(format!(
                            "`{name}` is above the contract size limit ({deployment_size} > {max_size})."
                        ))
                    ))?;
                }
            }
        }

        // Only prompt if we're broadcasting and we've not disabled interactivity.
        if prompt_user &&
            !self.non_interactive &&
            !Confirm::new().with_prompt("Do you wish to continue?".to_string()).interact()?
        {
            eyre::bail!("User canceled the script.");
        }

        Ok(())
    }
}

impl Provider for ScriptArgs {
    fn metadata(&self) -> Metadata {
        Metadata::named("Script Args Provider")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        let mut dict = Dict::default();
        if let Some(ref etherscan_api_key) =
            self.etherscan_api_key.as_ref().filter(|s| !s.trim().is_empty())
        {
            dict.insert(
                "etherscan_api_key".to_string(),
                figment::value::Value::from(etherscan_api_key.to_string()),
            );
        }
        Ok(Map::from([(Config::selected_profile(), dict)]))
    }
}

#[derive(Default)]
pub struct ScriptResult {
    pub success: bool,
    pub logs: Vec<Log>,
    pub traces: Traces,
    pub debug: Option<Vec<DebugArena>>,
    pub gas_used: u64,
    pub labeled_addresses: HashMap<Address, String>,
    pub transactions: Option<BroadcastableTransactions>,
    pub returned: Bytes,
    pub address: Option<Address>,
    pub breakpoints: Breakpoints,
}

impl ScriptResult {
    pub fn get_created_contracts(&self) -> Vec<AdditionalContract> {
        self.traces
            .iter()
            .flat_map(|(_, traces)| {
                traces.nodes().iter().filter_map(|node| {
                    if node.trace.kind.is_any_create() {
                        return Some(AdditionalContract {
                            opcode: node.trace.kind,
                            address: node.trace.address,
                            init_code: node.trace.data.clone(),
                        });
                    }
                    None
                })
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize)]
struct JsonResult {
    logs: Vec<String>,
    gas_used: u64,
    returns: HashMap<String, NestedValue>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NestedValue {
    pub internal_type: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct ScriptConfig {
    pub config: Config,
    pub evm_opts: EvmOpts,
    pub sender_nonce: u64,
    /// Maps a rpc url to a backend
    pub backends: HashMap<RpcUrl, Backend>,
}

impl ScriptConfig {
    pub async fn new(config: Config, evm_opts: EvmOpts) -> Result<Self> {
        let sender_nonce = if let Some(fork_url) = evm_opts.fork_url.as_ref() {
            next_nonce(evm_opts.sender, fork_url, None).await?
        } else {
            // dapptools compatibility
            1
        };
        Ok(Self { config, evm_opts, sender_nonce, backends: HashMap::new() })
    }

    pub async fn update_sender(&mut self, sender: Address) -> Result<()> {
        self.sender_nonce = if let Some(fork_url) = self.evm_opts.fork_url.as_ref() {
            next_nonce(sender, fork_url, None).await?
        } else {
            // dapptools compatibility
            1
        };
        self.evm_opts.sender = sender;
        Ok(())
    }

    async fn get_runner(&mut self) -> Result<ScriptRunner> {
        self._get_runner(None, false).await
    }

    async fn get_runner_with_cheatcodes(
        &mut self,
        script_wallets: ScriptWallets,
        debug: bool,
    ) -> Result<ScriptRunner> {
        self._get_runner(Some(script_wallets), debug).await
    }

    async fn _get_runner(
        &mut self,
        script_wallets: Option<ScriptWallets>,
        debug: bool,
    ) -> Result<ScriptRunner> {
        trace!("preparing script runner");
        let env = self.evm_opts.evm_env().await?;

        let db = if let Some(fork_url) = self.evm_opts.fork_url.as_ref() {
            match self.backends.get(fork_url) {
                Some(db) => db.clone(),
                None => {
                    let fork = self.evm_opts.get_fork(&self.config, env.clone());
                    let backend = Backend::spawn(fork);
                    self.backends.insert(fork_url.clone(), backend.clone());
                    backend
                }
            }
        } else {
            // It's only really `None`, when we don't pass any `--fork-url`. And if so, there is
            // no need to cache it, since there won't be any onchain simulation that we'd need
            // to cache the backend for.
            Backend::spawn(None)
        };

        // We need to enable tracing to decode contract names: local or external.
        let mut builder = ExecutorBuilder::new()
            .inspectors(|stack| stack.trace(true))
            .spec(self.config.evm_spec_id())
            .gas_limit(self.evm_opts.gas_limit());

        if let Some(script_wallets) = script_wallets {
            let mut inner = script_wallets.inner.lock();
            // if a sender is provided it will be set on evm_opts already
            // otherwise if we have exactly one signer we can assume it is the sender
            // else we throw an error to avoid landing on the DEFAULT_SENDER.
            if inner.provided_sender.is_none() {
                // may unlock some pending keystores
                let signers = inner.multi_wallet.signers()?;
                if signers.len() == 1 {
                    let address = signers.keys().next().unwrap();
                    self.evm_opts.sender = *address;
                }
                if signers.len() > 1 {
                    eyre::bail!("multiple signers found, please provide a --sender address.");
                }
            }
            // we can drop the inner wallet now that we have the sender
            // and release the lock for the runner.
            drop(inner);
            builder = builder.inspectors(|stack| {
                stack
                    .debug(debug)
                    .cheatcodes(
                        CheatsConfig::new(
                            &self.config,
                            self.evm_opts.clone(),
                            Some(script_wallets),
                        )
                        .into(),
                    )
                    .enable_isolation(self.evm_opts.isolate)
            });
        }

        Ok(ScriptRunner::new(
            builder.build(env, db),
            self.evm_opts.initial_balance,
            self.evm_opts.sender,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundry_config::{NamedChain, UnresolvedEnvVarError};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn can_parse_sig() {
        let sig = "0x522bb704000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfFFb92266";
        let args = ScriptArgs::parse_from(["foundry-cli", "Contract.sol", "--sig", sig]);
        assert_eq!(args.sig, sig);
    }

    #[test]
    fn can_parse_unlocked() {
        let args = ScriptArgs::parse_from([
            "foundry-cli",
            "Contract.sol",
            "--sender",
            "0x4e59b44847b379578588920ca78fbf26c0b4956c",
            "--unlocked",
        ]);
        assert!(args.unlocked);

        let key = U256::ZERO;
        let args = ScriptArgs::try_parse_from([
            "foundry-cli",
            "Contract.sol",
            "--sender",
            "0x4e59b44847b379578588920ca78fbf26c0b4956c",
            "--unlocked",
            "--private-key",
            key.to_string().as_str(),
        ]);
        assert!(args.is_err());
    }

    #[test]
    fn can_merge_script_config() {
        let args = ScriptArgs::parse_from([
            "foundry-cli",
            "Contract.sol",
            "--etherscan-api-key",
            "goerli",
        ]);
        let config = args.load_config();
        assert_eq!(config.etherscan_api_key, Some("goerli".to_string()));
    }

    #[test]
    fn can_parse_verifier_url() {
        let args = ScriptArgs::parse_from([
            "foundry-cli",
            "script",
            "script/Test.s.sol:TestScript",
            "--fork-url",
            "http://localhost:8545",
            "--verifier-url",
            "http://localhost:3000/api/verify",
            "--etherscan-api-key",
            "blacksmith",
            "--broadcast",
            "--verify",
            "-vvvvv",
        ]);
        assert_eq!(
            args.verifier.verifier_url,
            Some("http://localhost:3000/api/verify".to_string())
        );
    }

    #[test]
    fn can_extract_code_size_limit() {
        let args = ScriptArgs::parse_from([
            "foundry-cli",
            "script",
            "script/Test.s.sol:TestScript",
            "--fork-url",
            "http://localhost:8545",
            "--broadcast",
            "--code-size-limit",
            "50000",
        ]);
        assert_eq!(args.evm_opts.env.code_size_limit, Some(50000));
    }

    #[test]
    fn can_extract_script_etherscan_key() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        let config = r#"
                [profile.default]
                etherscan_api_key = "mumbai"

                [etherscan]
                mumbai = { key = "https://etherscan-mumbai.com/" }
            "#;

        let toml_file = root.join(Config::FILE_NAME);
        fs::write(toml_file, config).unwrap();
        let args = ScriptArgs::parse_from([
            "foundry-cli",
            "Contract.sol",
            "--etherscan-api-key",
            "mumbai",
            "--root",
            root.as_os_str().to_str().unwrap(),
        ]);

        let config = args.load_config();
        let mumbai = config.get_etherscan_api_key(Some(NamedChain::PolygonMumbai.into()));
        assert_eq!(mumbai, Some("https://etherscan-mumbai.com/".to_string()));
    }

    #[test]
    fn can_extract_script_rpc_alias() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        let config = r#"
                [profile.default]

                [rpc_endpoints]
                polygonMumbai = "https://polygon-mumbai.g.alchemy.com/v2/${_CAN_EXTRACT_RPC_ALIAS}"
            "#;

        let toml_file = root.join(Config::FILE_NAME);
        fs::write(toml_file, config).unwrap();
        let args = ScriptArgs::parse_from([
            "foundry-cli",
            "DeployV1",
            "--rpc-url",
            "polygonMumbai",
            "--root",
            root.as_os_str().to_str().unwrap(),
        ]);

        let err = args.load_config_and_evm_opts().unwrap_err();

        assert!(err.downcast::<UnresolvedEnvVarError>().is_ok());

        std::env::set_var("_CAN_EXTRACT_RPC_ALIAS", "123456");
        let (config, evm_opts) = args.load_config_and_evm_opts().unwrap();
        assert_eq!(config.eth_rpc_url, Some("polygonMumbai".to_string()));
        assert_eq!(
            evm_opts.fork_url,
            Some("https://polygon-mumbai.g.alchemy.com/v2/123456".to_string())
        );
    }

    #[test]
    fn can_extract_script_rpc_and_etherscan_alias() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        let config = r#"
            [profile.default]

            [rpc_endpoints]
            mumbai = "https://polygon-mumbai.g.alchemy.com/v2/${_EXTRACT_RPC_ALIAS}"

            [etherscan]
            mumbai = { key = "${_POLYSCAN_API_KEY}", chain = 80001, url = "https://api-testnet.polygonscan.com/" }
        "#;

        let toml_file = root.join(Config::FILE_NAME);
        fs::write(toml_file, config).unwrap();
        let args = ScriptArgs::parse_from([
            "foundry-cli",
            "DeployV1",
            "--rpc-url",
            "mumbai",
            "--etherscan-api-key",
            "mumbai",
            "--root",
            root.as_os_str().to_str().unwrap(),
        ]);
        let err = args.load_config_and_evm_opts().unwrap_err();

        assert!(err.downcast::<UnresolvedEnvVarError>().is_ok());

        std::env::set_var("_EXTRACT_RPC_ALIAS", "123456");
        std::env::set_var("_POLYSCAN_API_KEY", "polygonkey");
        let (config, evm_opts) = args.load_config_and_evm_opts().unwrap();
        assert_eq!(config.eth_rpc_url, Some("mumbai".to_string()));
        assert_eq!(
            evm_opts.fork_url,
            Some("https://polygon-mumbai.g.alchemy.com/v2/123456".to_string())
        );
        let etherscan = config.get_etherscan_api_key(Some(80001u64.into()));
        assert_eq!(etherscan, Some("polygonkey".to_string()));
        let etherscan = config.get_etherscan_api_key(None);
        assert_eq!(etherscan, Some("polygonkey".to_string()));
    }

    #[test]
    fn can_extract_script_rpc_and_sole_etherscan_alias() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        let config = r#"
                [profile.default]

               [rpc_endpoints]
                mumbai = "https://polygon-mumbai.g.alchemy.com/v2/${_SOLE_EXTRACT_RPC_ALIAS}"

                [etherscan]
                mumbai = { key = "${_SOLE_POLYSCAN_API_KEY}" }
            "#;

        let toml_file = root.join(Config::FILE_NAME);
        fs::write(toml_file, config).unwrap();
        let args = ScriptArgs::parse_from([
            "foundry-cli",
            "DeployV1",
            "--rpc-url",
            "mumbai",
            "--root",
            root.as_os_str().to_str().unwrap(),
        ]);
        let err = args.load_config_and_evm_opts().unwrap_err();

        assert!(err.downcast::<UnresolvedEnvVarError>().is_ok());

        std::env::set_var("_SOLE_EXTRACT_RPC_ALIAS", "123456");
        std::env::set_var("_SOLE_POLYSCAN_API_KEY", "polygonkey");
        let (config, evm_opts) = args.load_config_and_evm_opts().unwrap();
        assert_eq!(
            evm_opts.fork_url,
            Some("https://polygon-mumbai.g.alchemy.com/v2/123456".to_string())
        );
        let etherscan = config.get_etherscan_api_key(Some(80001u64.into()));
        assert_eq!(etherscan, Some("polygonkey".to_string()));
        let etherscan = config.get_etherscan_api_key(None);
        assert_eq!(etherscan, Some("polygonkey".to_string()));
    }

    // <https://github.com/foundry-rs/foundry/issues/5923>
    #[test]
    fn test_5923() {
        let args =
            ScriptArgs::parse_from(["foundry-cli", "DeployV1", "--priority-gas-price", "100"]);
        assert!(args.priority_gas_price.is_some());
    }

    // <https://github.com/foundry-rs/foundry/issues/5910>
    #[test]
    fn test_5910() {
        let args = ScriptArgs::parse_from([
            "foundry-cli",
            "--broadcast",
            "--with-gas-price",
            "0",
            "SolveTutorial",
        ]);
        assert!(args.with_gas_price.unwrap().is_zero());
    }
}
