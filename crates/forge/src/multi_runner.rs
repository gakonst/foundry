//! Forge test runner for multiple contracts.

use crate::{
    progress::TestsProgress, result::SuiteResult, runner::LIBRARY_DEPLOYER, ContractRunner,
    TestFilter, TestOptions,
};
use alloy_json_abi::{Function, JsonAbi};
use alloy_primitives::{Address, Bytes, U256};
use eyre::Result;
use foundry_common::{get_contract_name, ContractsByArtifact, TestFunctionExt};
use foundry_compilers::{
    artifacts::Libraries, compilers::Compiler, Artifact, ArtifactId, ProjectCompileOutput,
};
use foundry_config::Config;
use foundry_evm::{
    backend::Backend,
    decode::RevertDecoder,
    executors::ExecutorBuilder,
    fork::CreateFork,
    inspectors::CheatsConfig,
    opts::EvmOpts,
    revm,
    traces::{InternalTraceMode, TraceMode},
};
use foundry_linking::{LinkOutput, Linker};
use rayon::prelude::*;
use revm::primitives::SpecId;
use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::Debug,
    path::Path,
    sync::{mpsc, Arc},
    time::Instant,
};

#[derive(Debug, Clone)]
pub struct TestContract {
    pub abi: JsonAbi,
    pub bytecode: Bytes,
}

pub type DeployableContracts = BTreeMap<ArtifactId, TestContract>;

/// A multi contract runner receives a set of contracts deployed in an EVM instance and proceeds
/// to run all test functions in these contracts.
pub struct MultiContractRunner {
    /// Mapping of contract name to JsonAbi, creation bytecode and library bytecode which
    /// needs to be deployed & linked against
    pub contracts: DeployableContracts,
    /// The EVM instance used in the test runner
    pub evm_opts: EvmOpts,
    /// The configured evm
    pub env: revm::primitives::Env,
    /// The EVM spec
    pub evm_spec: SpecId,
    /// Revert decoder. Contains all known errors and their selectors.
    pub revert_decoder: RevertDecoder,
    /// The address which will be used as the `from` field in all EVM calls
    pub sender: Option<Address>,
    /// The fork to use at launch
    pub fork: Option<CreateFork>,
    /// Project config.
    pub config: Arc<Config>,
    /// Whether to collect coverage info
    pub coverage: bool,
    /// Whether to collect debug info
    pub debug: bool,
    /// Whether to enable steps tracking in the tracer.
    pub decode_internal: InternalTraceMode,
    /// Settings related to fuzz and/or invariant tests
    pub test_options: TestOptions,
    /// Whether to enable call isolation
    pub isolation: bool,
    /// Whether to enable Alphanet features.
    pub alphanet: bool,
    /// Known contracts linked with computed library addresses.
    pub known_contracts: ContractsByArtifact,
    /// Libraries to deploy.
    pub libs_to_deploy: Vec<Bytes>,
    /// Library addresses used to link contracts.
    pub libraries: Libraries,
}

impl MultiContractRunner {
    /// Returns an iterator over all contracts that match the filter.
    pub fn matching_contracts<'a>(
        &'a self,
        filter: &'a dyn TestFilter,
    ) -> impl Iterator<Item = (&ArtifactId, &TestContract)> {
        self.contracts.iter().filter(|&(id, c)| matches_contract(id, &c.abi, filter))
    }

    /// Returns an iterator over all test functions that match the filter.
    pub fn matching_test_functions<'a>(
        &'a self,
        filter: &'a dyn TestFilter,
    ) -> impl Iterator<Item = &Function> {
        self.matching_contracts(filter)
            .flat_map(|(_, c)| c.abi.functions())
            .filter(|func| is_matching_test(func, filter))
    }

    /// Returns an iterator over all test functions in contracts that match the filter.
    pub fn all_test_functions<'a>(
        &'a self,
        filter: &'a dyn TestFilter,
    ) -> impl Iterator<Item = &Function> {
        self.contracts
            .iter()
            .filter(|(id, _)| filter.matches_path(&id.source) && filter.matches_contract(&id.name))
            .flat_map(|(_, c)| c.abi.functions())
            .filter(|func| func.is_any_test())
    }

    /// Returns all matching tests grouped by contract grouped by file (file -> (contract -> tests))
    pub fn list(&self, filter: &dyn TestFilter) -> BTreeMap<String, BTreeMap<String, Vec<String>>> {
        self.matching_contracts(filter)
            .map(|(id, c)| {
                let source = id.source.as_path().display().to_string();
                let name = id.name.clone();
                let tests = c
                    .abi
                    .functions()
                    .filter(|func| is_matching_test(func, filter))
                    .map(|func| func.name.clone())
                    .collect::<Vec<_>>();
                (source, name, tests)
            })
            .fold(BTreeMap::new(), |mut acc, (source, name, tests)| {
                acc.entry(source).or_default().insert(name, tests);
                acc
            })
    }

    /// Executes _all_ tests that match the given `filter`.
    ///
    /// The same as [`test`](Self::test), but returns the results instead of streaming them.
    ///
    /// Note that this method returns only when all tests have been executed.
    pub fn test_collect(&mut self, filter: &dyn TestFilter) -> BTreeMap<String, SuiteResult> {
        self.test_iter(filter).collect()
    }

    /// Executes _all_ tests that match the given `filter`.
    ///
    /// The same as [`test`](Self::test), but returns the results instead of streaming them.
    ///
    /// Note that this method returns only when all tests have been executed.
    pub fn test_iter(
        &mut self,
        filter: &dyn TestFilter,
    ) -> impl Iterator<Item = (String, SuiteResult)> {
        let (tx, rx) = mpsc::channel();
        self.test(filter, tx, false);
        rx.into_iter()
    }

    /// Executes _all_ tests that match the given `filter`.
    ///
    /// This will create the runtime based on the configured `evm` ops and create the `Backend`
    /// before executing all contracts and their tests in _parallel_.
    ///
    /// Each Executor gets its own instance of the `Backend`.
    pub fn test(
        &mut self,
        filter: &dyn TestFilter,
        tx: mpsc::Sender<(String, SuiteResult)>,
        show_progress: bool,
    ) {
        let tokio_handle = tokio::runtime::Handle::current();
        trace!("running all tests");

        // The DB backend that serves all the data.
        let db = Backend::spawn(self.fork.take());

        let find_timer = Instant::now();
        let contracts = self.matching_contracts(filter).collect::<Vec<_>>();
        let find_time = find_timer.elapsed();
        debug!(
            "Found {} test contracts out of {} in {:?}",
            contracts.len(),
            self.contracts.len(),
            find_time,
        );
      
        if show_progress {
            let tests_progress = TestsProgress::new(contracts.len(), rayon::current_num_threads());
            // Collect test suite results to stream at the end of test run.
            let results: Vec<(String, SuiteResult)> = contracts
                .par_iter()
                .map(|&(id, contract)| {
                    let _guard = tokio_handle.enter();
                    tests_progress.inner.lock().start_suite_progress(&id.identifier());

                    let result = self.run_test_suite(
                        id,
                        contract,
                        db.clone(),
                        filter,
                        &tokio_handle,
                        Some(&tests_progress),
                    );

                    tests_progress
                        .inner
                        .lock()
                        .end_suite_progress(&id.identifier(), result.summary());

                    (id.identifier(), result)
                })
                .collect();

            tests_progress.inner.lock().clear();

            results.iter().for_each(|result| {
                let _ = tx.send(result.to_owned());
            });
        } else {
            contracts.par_iter().for_each(|&(id, contract)| {
                let _guard = tokio_handle.enter();
                let result =
                    self.run_test_suite(id, contract, db.clone(), filter, &tokio_handle, None);
                let _ = tx.send((id.identifier(), result));
            })
        }
    }

    fn run_test_suite(
        &self,
        artifact_id: &ArtifactId,
        contract: &TestContract,
        db: Backend,
        filter: &dyn TestFilter,
        tokio_handle: &tokio::runtime::Handle,
        progress: Option<&TestsProgress>,
    ) -> SuiteResult {
        let identifier = artifact_id.identifier();
        let mut span_name = identifier.as_str();

        let cheats_config = CheatsConfig::new(
            &self.config,
            self.evm_opts.clone(),
            Some(self.known_contracts.clone()),
            None,
            Some(artifact_id.version.clone()),
        );

        let trace_mode = TraceMode::default()
            .with_debug(self.debug)
            .with_decode_internal(self.decode_internal)
            .with_verbosity(self.evm_opts.verbosity);

        let executor = ExecutorBuilder::new()
            .inspectors(|stack| {
                stack
                    .cheatcodes(Arc::new(cheats_config))
                    .trace_mode(trace_mode)
                    .coverage(self.coverage)
                    .enable_isolation(self.isolation)
                    .alphanet(self.alphanet)
            })
            .spec(self.evm_spec)
            .gas_limit(self.evm_opts.gas_limit())
            .legacy_assertions(self.config.legacy_assertions)
            .build(self.env.clone(), db);

        if !enabled!(tracing::Level::TRACE) {
            span_name = get_contract_name(&identifier);
        }
        let span = debug_span!("suite", name = %span_name);
        let span_local = span.clone();
        let _guard = span_local.enter();

        debug!("start executing all tests in contract");

        let runner = ContractRunner {
            name: &identifier,
            contract,
            libs_to_deploy: &self.libs_to_deploy,
            executor,
            revert_decoder: &self.revert_decoder,
            initial_balance: self.evm_opts.initial_balance,
            sender: self.sender.unwrap_or_default(),
            debug: self.debug,
            progress,
            tokio_handle,
            span,
        };
        let r = runner.run_tests(filter, &self.test_options, self.known_contracts.clone());

        debug!(duration=?r.duration, "executed all tests in contract");

        r
    }
}

/// Builder used for instantiating the multi-contract runner
#[derive(Clone, Debug)]
#[must_use = "builders do nothing unless you call `build` on them"]
pub struct MultiContractRunnerBuilder {
    /// The address which will be used to deploy the initial contracts and send all
    /// transactions
    pub sender: Option<Address>,
    /// The initial balance for each one of the deployed smart contracts
    pub initial_balance: U256,
    /// The EVM spec to use
    pub evm_spec: Option<SpecId>,
    /// The fork to use at launch
    pub fork: Option<CreateFork>,
    /// Project config.
    pub config: Arc<Config>,
    /// Whether or not to collect coverage info
    pub coverage: bool,
    /// Whether or not to collect debug info
    pub debug: bool,
    /// Whether to enable steps tracking in the tracer.
    pub decode_internal: InternalTraceMode,
    /// Whether to enable call isolation
    pub isolation: bool,
    /// Whether to enable Alphanet features.
    pub alphanet: bool,
    /// Settings related to fuzz and/or invariant tests
    pub test_options: Option<TestOptions>,
}

impl MultiContractRunnerBuilder {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            sender: Default::default(),
            initial_balance: Default::default(),
            evm_spec: Default::default(),
            fork: Default::default(),
            coverage: Default::default(),
            debug: Default::default(),
            isolation: Default::default(),
            test_options: Default::default(),
            decode_internal: Default::default(),
            alphanet: Default::default(),
        }
    }

    pub fn sender(mut self, sender: Address) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn initial_balance(mut self, initial_balance: U256) -> Self {
        self.initial_balance = initial_balance;
        self
    }

    pub fn evm_spec(mut self, spec: SpecId) -> Self {
        self.evm_spec = Some(spec);
        self
    }

    pub fn with_fork(mut self, fork: Option<CreateFork>) -> Self {
        self.fork = fork;
        self
    }

    pub fn with_test_options(mut self, test_options: TestOptions) -> Self {
        self.test_options = Some(test_options);
        self
    }

    pub fn set_coverage(mut self, enable: bool) -> Self {
        self.coverage = enable;
        self
    }

    pub fn set_debug(mut self, enable: bool) -> Self {
        self.debug = enable;
        self
    }

    pub fn set_decode_internal(mut self, mode: InternalTraceMode) -> Self {
        self.decode_internal = mode;
        self
    }

    pub fn enable_isolation(mut self, enable: bool) -> Self {
        self.isolation = enable;
        self
    }

    pub fn alphanet(mut self, enable: bool) -> Self {
        self.alphanet = enable;
        self
    }

    /// Given an EVM, proceeds to return a runner which is able to execute all tests
    /// against that evm
    pub fn build<C: Compiler>(
        self,
        root: &Path,
        output: &ProjectCompileOutput<C>,
        env: revm::primitives::Env,
        evm_opts: EvmOpts,
    ) -> Result<MultiContractRunner> {
        let contracts = output
            .artifact_ids()
            .map(|(id, v)| (id.with_stripped_file_prefixes(root), v))
            .collect();
        let linker = Linker::new(root, contracts);

        // Build revert decoder from ABIs of all artifacts.
        let abis = linker
            .contracts
            .iter()
            .filter_map(|(_, contract)| contract.abi.as_ref().map(|abi| abi.borrow()));
        let revert_decoder = RevertDecoder::new().with_abis(abis);

        let LinkOutput { libraries, libs_to_deploy } = linker.link_with_nonce_or_address(
            Default::default(),
            LIBRARY_DEPLOYER,
            0,
            linker.contracts.keys(),
        )?;

        let linked_contracts = linker.get_linked_artifacts(&libraries)?;

        // Create a mapping of name => (abi, deployment code, Vec<library deployment code>)
        let mut deployable_contracts = DeployableContracts::default();

        for (id, contract) in linked_contracts.iter() {
            let Some(abi) = &contract.abi else { continue };

            // if it's a test, link it and add to deployable contracts
            if abi.constructor.as_ref().map(|c| c.inputs.is_empty()).unwrap_or(true) &&
                abi.functions().any(|func| func.name.is_any_test())
            {
                let Some(bytecode) =
                    contract.get_bytecode_bytes().map(|b| b.into_owned()).filter(|b| !b.is_empty())
                else {
                    continue;
                };

                deployable_contracts
                    .insert(id.clone(), TestContract { abi: abi.clone(), bytecode });
            }
        }

        let known_contracts = ContractsByArtifact::new(linked_contracts);

        Ok(MultiContractRunner {
            contracts: deployable_contracts,
            evm_opts,
            env,
            evm_spec: self.evm_spec.unwrap_or(SpecId::MERGE),
            sender: self.sender,
            revert_decoder,
            fork: self.fork,
            config: self.config,
            coverage: self.coverage,
            debug: self.debug,
            decode_internal: self.decode_internal,
            test_options: self.test_options.unwrap_or_default(),
            isolation: self.isolation,
            alphanet: self.alphanet,
            known_contracts,
            libs_to_deploy,
            libraries,
        })
    }
}

pub fn matches_contract(id: &ArtifactId, abi: &JsonAbi, filter: &dyn TestFilter) -> bool {
    (filter.matches_path(&id.source) && filter.matches_contract(&id.name)) &&
        abi.functions().any(|func| is_matching_test(func, filter))
}

/// Returns `true` if the function is a test function that matches the given filter.
pub(crate) fn is_matching_test(func: &Function, filter: &dyn TestFilter) -> bool {
    func.is_any_test() && filter.matches_test(&func.signature())
}
