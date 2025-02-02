//! # foundry-script
//!
//! Smart contract scripting.

#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[macro_use]
extern crate foundry_common;

#[macro_use]
extern crate tracing;

use crate::{broadcast::get_block_gas_limit, runner::ScriptRunner};
use alloy_json_abi::{Function, JsonAbi};
use alloy_primitives::{
    hex,
    map::{AddressHashMap, HashMap},
    Address, Bytes, Log, TxKind, U256,
};
use alloy_signer::Signer;
use broadcast::next_nonce;
use build::PreprocessedState;
use clap::{Parser, ValueHint};
use dialoguer::Confirm;
use eyre::{ContextCompat, Result};
use forge_script_sequence::{AdditionalContract, NestedValue};
use forge_verify::{RetryArgs, VerifierArgs};
use foundry_cli::{
    opts::{BuildOpts, GlobalArgs},
    utils::LoadConfig,
};
use foundry_common::{
    abi::{encode_function_args, get_func},
    evm::{Breakpoints, EvmArgs},
    shell, ContractsByArtifact, CONTRACT_MAX_SIZE, SELECTOR_LEN,
};
use foundry_compilers::ArtifactId;
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
    executors::ExecutorBuilder,
    inspectors::{
        cheatcodes::{BroadcastableTransactions, Wallets},
        CheatsConfig,
    },
    opts::EvmOpts,
    traces::{TraceMode, Traces},
};
use foundry_wallets::MultiWalletOpts;
use serde::Serialize;
use std::path::PathBuf;

mod broadcast;
mod build;
mod execute;
mod multi_sequence;
mod progress;
mod providers;
mod receipts;
mod runner;
mod sequence;
mod simulate;
mod transaction;
mod verify;

// Loads project's figment and merges the build cli arguments into it
foundry_config::merge_impl_figment_convert!(ScriptArgs, build, evm);

/// CLI arguments for `forge script`.
#[derive(Clone, Debug, Default, Parser)]
pub struct ScriptArgs {
    // Include global options for users of this struct.
    #[command(flatten)]
    pub global: GlobalArgs,

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

    /// Batch size of transactions.
    ///
    /// This is ignored and set to 1 if batching is not available or `--slow` is enabled.
    #[arg(long, default_value = "100")]
    pub batch_size: usize,

    /// Skips on-chain simulation.
    #[arg(long)]
    pub skip_simulation: bool,

    /// Relative percentage to multiply gas estimates by.
    #[arg(long, short, default_value = "130")]
    pub gas_estimate_multiplier: u64,

    /// Send via `eth_sendTransaction` using the `--from` argument or `$ETH_FROM` as sender
    #[arg(
        long,
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

    /// Dumps all debugger steps to file.
    #[arg(
        long,
        requires = "debug",
        value_hint = ValueHint::FilePath,
        value_name = "PATH"
    )]
    pub dump: Option<PathBuf>,

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

    /// Gas price for legacy transactions, or max fee per gas for EIP1559 transactions, either
    /// specified in wei, or as a string with a unit type.
    ///
    /// Examples: 1ether, 10gwei, 0.01ether
    #[arg(
        long,
        env = "ETH_GAS_PRICE",
        value_parser = foundry_cli::utils::parse_ether_value,
        value_name = "PRICE",
    )]
    pub with_gas_price: Option<U256>,

    /// Timeout to use for broadcasting transactions.
    #[arg(long, env = "ETH_TIMEOUT")]
    pub timeout: Option<u64>,

    #[command(flatten)]
    pub build: BuildOpts,

    #[command(flatten)]
    pub wallets: MultiWalletOpts,

    #[command(flatten)]
    pub evm: EvmArgs,

    #[command(flatten)]
    pub verifier: VerifierArgs,

    #[command(flatten)]
    pub retry: RetryArgs,
}

impl ScriptArgs {
    pub async fn preprocess(self) -> Result<PreprocessedState> {
        let script_wallets = Wallets::new(self.wallets.get_multi_wallet().await?, self.evm.sender);

        let (config, mut evm_opts) = self.load_config_and_evm_opts()?;

        if let Some(sender) = self.maybe_load_private_key()? {
            evm_opts.sender = sender;
        }

        let script_config = ScriptConfig::new(config, evm_opts).await?;

        Ok(PreprocessedState { args: self, script_config, script_wallets })
    }

    /// Executes the script
    pub async fn run_script(self) -> Result<()> {
        trace!(target: "script", "executing script command");

        let state = self.preprocess().await?;
        let create2_deployer = state.script_config.evm_opts.create2_deployer;
        let compiled = state.compile()?;

        // Move from `CompiledState` to `BundledState` either by resuming or executing and
        // simulating script.
        let bundled = if compiled.args.resume || (compiled.args.verify && !compiled.args.broadcast)
        {
            compiled.resume().await?
        } else {
            // Drive state machine to point at which we have everything needed for simulation.
            let pre_simulation = compiled
                .link()
                .await?
                .prepare_execution()
                .await?
                .execute()
                .await?
                .prepare_simulation()
                .await?;

            if pre_simulation.args.debug {
                return match pre_simulation.args.dump.clone() {
                    Some(path) => pre_simulation.dump_debugger(&path),
                    None => pre_simulation.run_debugger(),
                };
            }

            if shell::is_json() {
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
                .is_none_or(|txs| txs.is_empty())
            {
                return Ok(());
            }

            // Check if there are any missing RPCs and exit early to avoid hard error.
            if pre_simulation.execution_artifacts.rpc_data.missing_rpc {
                if !shell::is_json() {
                    sh_println!("\nIf you wish to simulate on-chain transactions pass a RPC URL.")?;
                }

                return Ok(());
            }

            pre_simulation.args.check_contract_sizes(
                &pre_simulation.execution_result,
                &pre_simulation.build_data.known_contracts,
                create2_deployer,
            )?;

            pre_simulation.fill_metadata().await?.bundle().await?
        };

        // Exit early in case user didn't provide any broadcast/verify related flags.
        if !bundled.args.should_broadcast() {
            if !shell::is_json() {
                if shell::verbosity() >= 4 {
                    sh_println!("\n=== Transactions that will be broadcast ===\n")?;
                    bundled.sequence.show_transactions()?;
                }

                sh_println!("\nSIMULATION COMPLETE. To broadcast these transactions, add --broadcast and wallet configuration(s) to the previous command. See forge script --help for more.")?;
            }
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

    /// In case the user has loaded *only* one private-key, we can assume that he's using it as the
    /// `--sender`
    fn maybe_load_private_key(&self) -> Result<Option<Address>> {
        let maybe_sender = self
            .wallets
            .private_keys()?
            .filter(|pks| pks.len() == 1)
            .map(|pks| pks.first().unwrap().address());
        Ok(maybe_sender)
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
        known_contracts: &ContractsByArtifact,
        create2_deployer: Address,
    ) -> Result<()> {
        // (name, &init, &deployed)[]
        let mut bytecodes: Vec<(String, &[u8], &[u8])> = vec![];

        // From artifacts
        for (artifact, contract) in known_contracts.iter() {
            let Some(bytecode) = contract.bytecode() else { continue };
            let Some(deployed_bytecode) = contract.deployed_bytecode() else { continue };
            bytecodes.push((artifact.name.clone(), bytecode, deployed_bytecode));
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
        let max_size = match self.evm.env.code_size_limit {
            Some(size) => size,
            None => CONTRACT_MAX_SIZE,
        };

        for (data, to) in result.transactions.iter().flat_map(|txes| {
            txes.iter().filter_map(|tx| {
                tx.transaction
                    .input()
                    .filter(|data| data.len() > max_size)
                    .map(|data| (data, tx.transaction.to()))
            })
        }) {
            let mut offset = 0;

            // Find if it's a CREATE or CREATE2. Otherwise, skip transaction.
            if let Some(TxKind::Call(to)) = to {
                if to == create2_deployer {
                    // Size of the salt prefix.
                    offset = 32;
                } else {
                    continue;
                }
            } else if let Some(TxKind::Create) = to {
                // Pass
            }

            // Find artifact with a deployment code same as the data.
            if let Some((name, _, deployed_code)) =
                bytecodes.iter().find(|(_, init_code, _)| *init_code == &data[offset..])
            {
                let deployment_size = deployed_code.len();

                if deployment_size > max_size {
                    prompt_user = self.should_broadcast();
                    sh_err!(
                        "`{name}` is above the contract size limit ({deployment_size} > {max_size})."
                    )?;
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

    /// We only broadcast transactions if --broadcast or --resume was passed.
    fn should_broadcast(&self) -> bool {
        self.broadcast || self.resume
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
        if let Some(timeout) = self.timeout {
            dict.insert("transaction_timeout".to_string(), timeout.into());
        }
        Ok(Map::from([(Config::selected_profile(), dict)]))
    }
}

#[derive(Default, Serialize)]
pub struct ScriptResult {
    pub success: bool,
    #[serde(rename = "raw_logs")]
    pub logs: Vec<Log>,
    pub traces: Traces,
    pub gas_used: u64,
    pub labeled_addresses: AddressHashMap<String>,
    #[serde(skip)]
    pub transactions: Option<BroadcastableTransactions>,
    pub returned: Bytes,
    pub address: Option<Address>,
    #[serde(skip)]
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

#[derive(Serialize)]
struct JsonResult<'a> {
    logs: Vec<String>,
    returns: &'a HashMap<String, NestedValue>,
    #[serde(flatten)]
    result: &'a ScriptResult,
}

#[derive(Clone, Debug)]
pub struct ScriptConfig {
    pub config: Config,
    pub evm_opts: EvmOpts,
    pub sender_nonce: u64,
    /// Maps a rpc url to a backend
    pub backends: HashMap<String, Backend>,
}

impl ScriptConfig {
    pub async fn new(config: Config, mut evm_opts: EvmOpts) -> Result<Self> {
        let sender_nonce = if let Some(fork_url) = evm_opts.fork_url.as_ref() {
            next_nonce(evm_opts.sender, fork_url, evm_opts.fork_block_number).await?
        } else {
            // dapptools compatibility
            1
        };

        if let Some(fork_url) = evm_opts.fork_url.as_ref() {
            evm_opts.env.gas_limit =
                get_block_gas_limit(fork_url, evm_opts.fork_block_number).await?.into();
        }

        Ok(Self { config, evm_opts, sender_nonce, backends: HashMap::default() })
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
        known_contracts: ContractsByArtifact,
        script_wallets: Wallets,
        debug: bool,
        target: ArtifactId,
    ) -> Result<ScriptRunner> {
        self._get_runner(Some((known_contracts, script_wallets, target)), debug).await
    }

    async fn _get_runner(
        &mut self,
        cheats_data: Option<(ContractsByArtifact, Wallets, ArtifactId)>,
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
            .inspectors(|stack| {
                stack
                    .trace_mode(if debug { TraceMode::Debug } else { TraceMode::Call })
                    .odyssey(self.evm_opts.odyssey)
                    .create2_deployer(self.evm_opts.create2_deployer)
            })
            .spec_id(self.config.evm_spec_id())
            .gas_limit(self.evm_opts.gas_limit())
            .legacy_assertions(self.config.legacy_assertions);

        if let Some((known_contracts, script_wallets, target)) = cheats_data {
            builder = builder.inspectors(|stack| {
                stack
                    .cheatcodes(
                        CheatsConfig::new(
                            &self.config,
                            self.evm_opts.clone(),
                            Some(known_contracts),
                            Some(target),
                        )
                        .into(),
                    )
                    .wallets(script_wallets)
                    .enable_isolation(self.evm_opts.isolate)
            });
        }

        Ok(ScriptRunner::new(builder.build(env, db), self.evm_opts.clone()))
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
        let config = args.load_config().unwrap();
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
        assert_eq!(args.evm.env.code_size_limit, Some(50000));
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

        let config = args.load_config().unwrap();
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
