use crate::{
    executors::{Executor, RawCallResult},
    inspectors::Fuzzer,
};
use alloy_primitives::{Address, Bytes, FixedBytes, Selector, U256};
use alloy_sol_types::{sol, SolCall};
use eyre::{eyre, ContextCompat, Result};
use foundry_common::contracts::{ContractsByAddress, ContractsByArtifact};
use foundry_config::InvariantConfig;
use foundry_evm_core::{
    constants::{
        CALLER, CHEATCODE_ADDRESS, DEFAULT_CREATE2_DEPLOYER, HARDHAT_CONSOLE_ADDRESS, MAGIC_ASSUME,
    },
    precompiles::PRECOMPILES,
};
use foundry_evm_fuzz::{
    invariant::{
        ArtifactFilters, BasicTxDetails, FuzzRunIdentifiedContracts, InvariantContract,
        RandomCallGenerator, SenderFilters, TargetedContract, TargetedContracts,
    },
    strategies::{invariant_strat, override_call_strat, EvmFuzzState},
    FuzzCase, FuzzFixtures, FuzzedCases,
};
use foundry_evm_traces::{CallTraceArena, SparsedTraceArena};
use indicatif::ProgressBar;
use parking_lot::RwLock;
use proptest::{
    strategy::{Strategy, ValueTree},
    test_runner::{TestCaseError, TestRunner},
};
use result::{assert_after_invariant, assert_invariants, can_continue};
use revm::primitives::HashMap;
use shrink::shrink_sequence;
use std::{cell::RefCell, collections::btree_map::Entry, sync::Arc};

mod error;
pub use error::{InvariantFailures, InvariantFuzzError};
use foundry_evm_coverage::HitMaps;

mod replay;
pub use replay::{replay_error, replay_run};

mod result;
pub use result::InvariantFuzzTestResult;

mod shrink;
use crate::executors::EvmError;
pub use shrink::check_sequence;

sol! {
    interface IInvariantTest {
        #[derive(Default)]
        struct FuzzSelector {
            address addr;
            bytes4[] selectors;
        }

        #[derive(Default)]
        struct FuzzArtifactSelector {
            string artifact;
            bytes4[] selectors;
        }

        #[derive(Default)]
        struct FuzzInterface {
            address addr;
            string[] artifacts;
        }

        function afterInvariant() external;

        #[derive(Default)]
        function excludeArtifacts() public view returns (string[] memory excludedArtifacts);

        #[derive(Default)]
        function excludeContracts() public view returns (address[] memory excludedContracts);

        #[derive(Default)]
        function excludeSelectors() public view returns (FuzzSelector[] memory excludedSelectors);

        #[derive(Default)]
        function excludeSenders() public view returns (address[] memory excludedSenders);

        #[derive(Default)]
        function targetArtifacts() public view returns (string[] memory targetedArtifacts);

        #[derive(Default)]
        function targetArtifactSelectors() public view returns (FuzzArtifactSelector[] memory targetedArtifactSelectors);

        #[derive(Default)]
        function targetContracts() public view returns (address[] memory targetedContracts);

        #[derive(Default)]
        function targetSelectors() public view returns (FuzzSelector[] memory targetedSelectors);

        #[derive(Default)]
        function targetSenders() public view returns (address[] memory targetedSenders);

        #[derive(Default)]
        function targetInterfaces() public view returns (FuzzInterface[] memory targetedInterfaces);
    }
}

/// Contains data collected during invariant test runs.
pub struct InvariantTestData {
    // Consumed gas and calldata of every successful fuzz call.
    pub fuzz_cases: Vec<FuzzedCases>,
    // Data related to reverts or failed assertions of the test.
    pub failures: InvariantFailures,
    // Calldata in the last invariant run.
    pub last_run_inputs: Vec<BasicTxDetails>,
    // Additional traces for gas report.
    pub gas_report_traces: Vec<Vec<CallTraceArena>>,
    // Last call results of the invariant test.
    pub last_call_results: Option<RawCallResult>,
    // Coverage information collected from all fuzzed calls.
    pub coverage: Option<HitMaps>,

    // Proptest runner to query for random values.
    // The strategy only comes with the first `input`. We fill the rest of the `inputs`
    // until the desired `depth` so we can use the evolving fuzz dictionary
    // during the run.
    pub branch_runner: TestRunner,
}

/// Contains invariant test data.
pub struct InvariantTest {
    // Fuzz state of invariant test.
    pub fuzz_state: EvmFuzzState,
    // Contracts fuzzed by the invariant test.
    pub targeted_contracts: FuzzRunIdentifiedContracts,
    // Data collected during invariant runs.
    pub execution_data: RefCell<InvariantTestData>,
}

impl InvariantTest {
    /// Instantiates an invariant test.
    pub fn new(
        fuzz_state: EvmFuzzState,
        targeted_contracts: FuzzRunIdentifiedContracts,
        failures: InvariantFailures,
        last_call_results: Option<RawCallResult>,
        branch_runner: TestRunner,
    ) -> Self {
        let mut fuzz_cases = vec![];
        if last_call_results.is_none() {
            fuzz_cases.push(FuzzedCases::new(vec![]));
        }
        let execution_data = RefCell::new(InvariantTestData {
            fuzz_cases,
            failures,
            last_run_inputs: vec![],
            gas_report_traces: vec![],
            last_call_results,
            coverage: None,
            branch_runner,
        });
        Self { fuzz_state, targeted_contracts, execution_data }
    }

    /// Returns number of invariant test reverts.
    pub fn reverts(&self) -> usize {
        self.execution_data.borrow().failures.reverts
    }

    /// Whether invariant test has errors or not.
    pub fn has_errors(&self) -> bool {
        self.execution_data.borrow().failures.error.is_some()
    }

    /// Set invariant test error.
    pub fn set_error(&self, error: InvariantFuzzError) {
        self.execution_data.borrow_mut().failures.error = Some(error);
    }

    /// Set last invariant test call results.
    pub fn set_last_call_results(&self, call_result: Option<RawCallResult>) {
        self.execution_data.borrow_mut().last_call_results = call_result;
    }

    /// Set last invariant run call sequence.
    pub fn set_last_run_inputs(&self, inputs: &Vec<BasicTxDetails>) {
        self.execution_data.borrow_mut().last_run_inputs.clone_from(inputs);
    }

    /// Merge current collected coverage with the new coverage from last fuzzed call.
    pub fn merge_coverage(&self, new_coverage: Option<HitMaps>) {
        match &mut self.execution_data.borrow_mut().coverage {
            Some(prev) => prev.merge(new_coverage.unwrap()),
            opt => *opt = new_coverage,
        }
    }

    /// End invariant test run by collecting results, cleaning collected artifacts and reverting
    /// created fuzz state.
    pub fn end_run(&self, run: InvariantTestRun, gas_samples: usize) {
        // We clear all the targeted contracts created during this run.
        self.targeted_contracts.clear_created_contracts(run.created_contracts);

        let mut invariant_data = self.execution_data.borrow_mut();
        if invariant_data.gas_report_traces.len() < gas_samples {
            invariant_data
                .gas_report_traces
                .push(run.run_traces.into_iter().map(|arena| arena.arena).collect());
        }
        invariant_data.fuzz_cases.push(FuzzedCases::new(run.fuzz_runs));

        // Revert state to not persist values between runs.
        self.fuzz_state.revert();
    }
}

/// Contains data for an invariant test run.
pub struct InvariantTestRun {
    // Invariant run call sequence.
    pub inputs: Vec<BasicTxDetails>,
    // Current invariant run executor.
    pub executor: Executor,
    // Invariant run stat reports (eg. gas usage).
    pub fuzz_runs: Vec<FuzzCase>,
    // Contracts created during current invariant run.
    pub created_contracts: Vec<Address>,
    // Traces of each call of the invariant run call sequence.
    pub run_traces: Vec<SparsedTraceArena>,
    // Current depth of invariant run.
    pub depth: u32,
    // Current assume rejects of the invariant run.
    pub assume_rejects_counter: u32,
}

impl InvariantTestRun {
    /// Instantiates an invariant test run.
    pub fn new(first_input: BasicTxDetails, executor: Executor, depth: usize) -> Self {
        Self {
            inputs: vec![first_input],
            executor,
            fuzz_runs: Vec::with_capacity(depth),
            created_contracts: vec![],
            run_traces: vec![],
            depth: 0,
            assume_rejects_counter: 0,
        }
    }
}

/// Wrapper around any [`Executor`] implementor which provides fuzzing support using [`proptest`].
///
/// After instantiation, calling `invariant_fuzz` will proceed to hammer the deployed smart
/// contracts with inputs, until it finds a counterexample sequence. The provided [`TestRunner`]
/// contains all the configuration which can be overridden via [environment
/// variables](proptest::test_runner::Config)
pub struct InvariantExecutor<'a> {
    pub executor: Executor,
    /// Proptest runner.
    runner: TestRunner,
    /// The invariant configuration
    config: InvariantConfig,
    /// Contracts deployed with `setUp()`
    setup_contracts: &'a ContractsByAddress,
    /// Contracts that are part of the project but have not been deployed yet. We need the bytecode
    /// to identify them from the stateset changes.
    project_contracts: &'a ContractsByArtifact,
    /// Filters contracts to be fuzzed through their artifact identifiers.
    artifact_filters: ArtifactFilters,
}

impl<'a> InvariantExecutor<'a> {
    /// Instantiates a fuzzed executor EVM given a testrunner
    pub fn new(
        executor: Executor,
        runner: TestRunner,
        config: InvariantConfig,
        setup_contracts: &'a ContractsByAddress,
        project_contracts: &'a ContractsByArtifact,
    ) -> Self {
        Self {
            executor,
            runner,
            config,
            setup_contracts,
            project_contracts,
            artifact_filters: ArtifactFilters::default(),
        }
    }

    /// Fuzzes any deployed contract and checks any broken invariant at `invariant_address`.
    pub fn invariant_fuzz(
        &mut self,
        invariant_contract: InvariantContract<'_>,
        fuzz_fixtures: &FuzzFixtures,
        progress: Option<&ProgressBar>,
    ) -> Result<InvariantFuzzTestResult> {
        // Throw an error to abort test run if the invariant function accepts input params
        if !invariant_contract.invariant_function.inputs.is_empty() {
            return Err(eyre!("Invariant test function should have no inputs"))
        }

        let (invariant_test, invariant_strategy) =
            self.prepare_test(&invariant_contract, fuzz_fixtures)?;

        let _ = self.runner.run(&invariant_strategy, |first_input| {
            // Create current invariant run data.
            let mut current_run = InvariantTestRun::new(
                first_input,
                // Before each run, we must reset the backend state.
                self.executor.clone(),
                self.config.depth as usize,
            );

            // We stop the run immediately if we have reverted, and `fail_on_revert` is set.
            if self.config.fail_on_revert && invariant_test.reverts() > 0 {
                return Err(TestCaseError::fail("Revert occurred."))
            }

            while current_run.depth < self.config.depth {
                let tx = current_run.inputs.last().ok_or_else(|| {
                    TestCaseError::fail("No input generated to call fuzzed target.")
                })?;
                if current_run.executor.get_balance(tx.sender)? < tx.value {
                    current_run.executor.set_balance(tx.sender, tx.value)?;
                }
                // Execute call from the randomly generated sequence and commit state changes.
                let call_result = current_run
                    .executor
                    .transact_raw(
                        tx.sender,
                        tx.call_details.target,
                        tx.call_details.calldata.clone(),
                        tx.value,
                    )
                    .map_err(|e| {
                        TestCaseError::fail(format!("Could not make raw evm call: {e}"))
                    })?;

                // Collect coverage from last fuzzed call.
                invariant_test.merge_coverage(call_result.coverage.clone());

                if call_result.result.as_ref() == MAGIC_ASSUME {
                    current_run.inputs.pop();
                    current_run.assume_rejects_counter += 1;
                    if current_run.assume_rejects_counter > self.config.max_assume_rejects {
                        invariant_test.set_error(InvariantFuzzError::MaxAssumeRejects(
                            self.config.max_assume_rejects,
                        ));
                        return Err(TestCaseError::fail("Max number of vm.assume rejects reached."))
                    }
                } else {
                    // Collect data for fuzzing from the state changeset.
                    let mut state_changeset = call_result.state_changeset.clone();

                    if !call_result.reverted {
                        collect_data(
                            &invariant_test,
                            &mut state_changeset,
                            tx,
                            &call_result,
                            self.config.depth,
                        );
                    }

                    // Collect created contracts and add to fuzz targets only if targeted contracts
                    // are updatable.
                    if let Err(error) =
                        &invariant_test.targeted_contracts.collect_created_contracts(
                            &state_changeset,
                            self.project_contracts,
                            self.setup_contracts,
                            &self.artifact_filters,
                            &mut current_run.created_contracts,
                        )
                    {
                        warn!(target: "forge::test", "{error}");
                    }

                    current_run.fuzz_runs.push(FuzzCase {
                        calldata: tx.call_details.calldata.clone(),
                        gas: call_result.gas_used,
                        stipend: call_result.stipend,
                    });

                    let result = can_continue(
                        &invariant_contract,
                        &invariant_test,
                        &mut current_run,
                        &self.config,
                        call_result,
                        &state_changeset,
                    )
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;

                    if !result.can_continue || current_run.depth == self.config.depth - 1 {
                        invariant_test.set_last_run_inputs(&current_run.inputs);
                    }

                    // If test cannot continue then stop current run and exit test suite.
                    if !result.can_continue {
                        return Err(TestCaseError::fail("Test cannot continue."))
                    }

                    invariant_test.set_last_call_results(result.call_result);
                    current_run.depth += 1;
                }

                // Generates the next call from the run using the recently updated
                // dictionary.
                current_run.inputs.push(
                    invariant_strategy
                        .new_tree(&mut invariant_test.execution_data.borrow_mut().branch_runner)
                        .map_err(|_| TestCaseError::Fail("Could not generate case".into()))?
                        .current(),
                );
            }

            // Call `afterInvariant` only if it is declared and test didn't fail already.
            if invariant_contract.call_after_invariant && !invariant_test.has_errors() {
                assert_after_invariant(
                    &invariant_contract,
                    &invariant_test,
                    &current_run,
                    &self.config,
                )
                .map_err(|_| TestCaseError::Fail("Failed to call afterInvariant".into()))?;
            }

            // End current invariant test run.
            invariant_test.end_run(current_run, self.config.gas_report_samples as usize);

            // If running with progress then increment completed runs.
            if let Some(progress) = progress {
                progress.inc(1);
            }

            Ok(())
        });

        trace!(?fuzz_fixtures);
        invariant_test.fuzz_state.log_stats();

        let result = invariant_test.execution_data.into_inner();
        Ok(InvariantFuzzTestResult {
            error: result.failures.error,
            cases: result.fuzz_cases,
            reverts: result.failures.reverts,
            last_run_inputs: result.last_run_inputs,
            gas_report_traces: result.gas_report_traces,
            coverage: result.coverage,
        })
    }

    /// Prepares certain structures to execute the invariant tests:
    /// * Invariant Fuzz Test.
    /// * Invariant Strategy
    fn prepare_test(
        &mut self,
        invariant_contract: &InvariantContract<'_>,
        fuzz_fixtures: &FuzzFixtures,
    ) -> Result<(InvariantTest, impl Strategy<Value = BasicTxDetails>)> {
        // Finds out the chosen deployed contracts and/or senders.
        self.select_contract_artifacts(invariant_contract.address)?;
        let (targeted_senders, targeted_contracts) =
            self.select_contracts_and_senders(invariant_contract.address)?;

        // Stores fuzz state for use with [fuzz_calldata_from_state].
        let fuzz_state =
            EvmFuzzState::new(self.executor.backend().mem_db(), self.config.dictionary);

        // Creates the invariant strategy.
        let strat = invariant_strat(
            fuzz_state.clone(),
            targeted_senders,
            targeted_contracts.clone(),
            self.config.dictionary.dictionary_weight,
            fuzz_fixtures.clone(),
        )
        .no_shrink();

        // Allows `override_call_strat` to use the address given by the Fuzzer inspector during
        // EVM execution.
        let mut call_generator = None;
        if self.config.call_override {
            let target_contract_ref = Arc::new(RwLock::new(Address::ZERO));

            call_generator = Some(RandomCallGenerator::new(
                invariant_contract.address,
                self.runner.clone(),
                override_call_strat(
                    fuzz_state.clone(),
                    targeted_contracts.clone(),
                    target_contract_ref.clone(),
                    fuzz_fixtures.clone(),
                ),
                target_contract_ref,
            ));
        }

        self.executor.inspector_mut().fuzzer =
            Some(Fuzzer { call_generator, fuzz_state: fuzz_state.clone(), collect: true });

        // Let's make sure the invariant is sound before actually starting the run:
        // We'll assert the invariant in its initial state, and if it fails, we'll
        // already know if we can early exit the invariant run.
        // This does not count as a fuzz run. It will just register the revert.
        let mut failures = InvariantFailures::new();
        let last_call_results = assert_invariants(
            invariant_contract,
            &self.config,
            &targeted_contracts,
            &self.executor,
            &[],
            &mut failures,
        )?;
        if let Some(error) = failures.error {
            return Err(eyre!(error.revert_reason().unwrap_or_default()))
        }

        Ok((
            InvariantTest::new(
                fuzz_state,
                targeted_contracts,
                failures,
                last_call_results,
                self.runner.clone(),
            ),
            strat,
        ))
    }

    /// Fills the `InvariantExecutor` with the artifact identifier filters (in `path:name` string
    /// format). They will be used to filter contracts after the `setUp`, and more importantly,
    /// during the runs.
    ///
    /// Also excludes any contract without any mutable functions.
    ///
    /// Priority:
    ///
    /// targetArtifactSelectors > excludeArtifacts > targetArtifacts
    pub fn select_contract_artifacts(&mut self, invariant_address: Address) -> Result<()> {
        let result = self
            .executor
            .call_sol_default(invariant_address, &IInvariantTest::targetArtifactSelectorsCall {});

        // Insert them into the executor `targeted_abi`.
        for IInvariantTest::FuzzArtifactSelector { artifact, selectors } in
            result.targetedArtifactSelectors
        {
            let identifier = self.validate_selected_contract(artifact, &selectors)?;
            self.artifact_filters.targeted.entry(identifier).or_default().extend(selectors);
        }

        let selected = self
            .executor
            .call_sol_default(invariant_address, &IInvariantTest::targetArtifactsCall {});
        let excluded = self
            .executor
            .call_sol_default(invariant_address, &IInvariantTest::excludeArtifactsCall {});

        // Insert `excludeArtifacts` into the executor `excluded_abi`.
        for contract in excluded.excludedArtifacts {
            let identifier = self.validate_selected_contract(contract, &[])?;

            if !self.artifact_filters.excluded.contains(&identifier) {
                self.artifact_filters.excluded.push(identifier);
            }
        }

        // Exclude any artifact without mutable functions.
        for (artifact, contract) in self.project_contracts.iter() {
            if contract
                .abi
                .functions()
                .filter(|func| {
                    !matches!(
                        func.state_mutability,
                        alloy_json_abi::StateMutability::Pure |
                            alloy_json_abi::StateMutability::View
                    )
                })
                .count() ==
                0 &&
                !self.artifact_filters.excluded.contains(&artifact.identifier())
            {
                self.artifact_filters.excluded.push(artifact.identifier());
            }
        }

        // Insert `targetArtifacts` into the executor `targeted_abi`, if they have not been seen
        // before.
        for contract in selected.targetedArtifacts {
            let identifier = self.validate_selected_contract(contract, &[])?;

            if !self.artifact_filters.targeted.contains_key(&identifier) &&
                !self.artifact_filters.excluded.contains(&identifier)
            {
                self.artifact_filters.targeted.insert(identifier, vec![]);
            }
        }
        Ok(())
    }

    /// Makes sure that the contract exists in the project. If so, it returns its artifact
    /// identifier.
    fn validate_selected_contract(
        &mut self,
        contract: String,
        selectors: &[FixedBytes<4>],
    ) -> Result<String> {
        if let Some((artifact, contract_data)) =
            self.project_contracts.find_by_name_or_identifier(&contract)?
        {
            // Check that the selectors really exist for this contract.
            for selector in selectors {
                contract_data
                    .abi
                    .functions()
                    .find(|func| func.selector().as_slice() == selector.as_slice())
                    .wrap_err(format!("{contract} does not have the selector {selector:?}"))?;
            }

            return Ok(artifact.identifier())
        }
        eyre::bail!("{contract} not found in the project. Allowed format: `contract_name` or `contract_path:contract_name`.");
    }

    /// Selects senders and contracts based on the contract methods `targetSenders() -> address[]`,
    /// `targetContracts() -> address[]` and `excludeContracts() -> address[]`.
    pub fn select_contracts_and_senders(
        &self,
        to: Address,
    ) -> Result<(SenderFilters, FuzzRunIdentifiedContracts)> {
        let targeted_senders = self
            .executor
            .call_sol_default(to, &IInvariantTest::targetSendersCall {})
            .targetedSenders;
        let mut excluded_senders = self
            .executor
            .call_sol_default(to, &IInvariantTest::excludeSendersCall {})
            .excludedSenders;
        // Extend with default excluded addresses - https://github.com/foundry-rs/foundry/issues/4163
        excluded_senders.extend([
            CHEATCODE_ADDRESS,
            HARDHAT_CONSOLE_ADDRESS,
            DEFAULT_CREATE2_DEPLOYER,
        ]);
        // Extend with precompiles - https://github.com/foundry-rs/foundry/issues/4287
        excluded_senders.extend(PRECOMPILES);
        let sender_filters = SenderFilters::new(targeted_senders, excluded_senders);

        let selected = self
            .executor
            .call_sol_default(to, &IInvariantTest::targetContractsCall {})
            .targetedContracts;
        let excluded = self
            .executor
            .call_sol_default(to, &IInvariantTest::excludeContractsCall {})
            .excludedContracts;

        let contracts = self
            .setup_contracts
            .iter()
            .filter(|&(addr, (identifier, _))| {
                *addr != to &&
                    *addr != CHEATCODE_ADDRESS &&
                    *addr != HARDHAT_CONSOLE_ADDRESS &&
                    (selected.is_empty() || selected.contains(addr)) &&
                    (excluded.is_empty() || !excluded.contains(addr)) &&
                    self.artifact_filters.matches(identifier)
            })
            .map(|(addr, (identifier, abi))| {
                (*addr, TargetedContract::new(identifier.clone(), abi.clone()))
            })
            .collect();
        let mut contracts = TargetedContracts { inner: contracts };

        self.target_interfaces(to, &mut contracts)?;

        self.select_selectors(to, &mut contracts)?;

        // There should be at least one contract identified as target for fuzz runs.
        if contracts.is_empty() {
            eyre::bail!("No contracts to fuzz.");
        }

        Ok((sender_filters, FuzzRunIdentifiedContracts::new(contracts, selected.is_empty())))
    }

    /// Extends the contracts and selectors to fuzz with the addresses and ABIs specified in
    /// `targetInterfaces() -> (address, string[])[]`. Enables targeting of addresses that are
    /// not deployed during `setUp` such as when fuzzing in a forked environment. Also enables
    /// targeting of delegate proxies and contracts deployed with `create` or `create2`.
    pub fn target_interfaces(
        &self,
        invariant_address: Address,
        targeted_contracts: &mut TargetedContracts,
    ) -> Result<()> {
        let interfaces = self
            .executor
            .call_sol_default(invariant_address, &IInvariantTest::targetInterfacesCall {})
            .targetedInterfaces;

        // Since `targetInterfaces` returns a tuple array there is no guarantee
        // that the addresses are unique this map is used to merge functions of
        // the specified interfaces for the same address. For example:
        // `[(addr1, ["IERC20", "IOwnable"])]` and `[(addr1, ["IERC20"]), (addr1, ("IOwnable"))]`
        // should be equivalent.
        let mut combined = TargetedContracts::new();

        // Loop through each address and its associated artifact identifiers.
        // We're borrowing here to avoid taking full ownership.
        for IInvariantTest::FuzzInterface { addr, artifacts } in &interfaces {
            // Identifiers are specified as an array, so we loop through them.
            for identifier in artifacts {
                // Try to find the contract by name or identifier in the project's contracts.
                if let Some((_, contract)) =
                    self.project_contracts.find_by_name_or_identifier(identifier)?
                {
                    combined
                        // Check if there's an entry for the given key in the 'combined' map.
                        .entry(*addr)
                        // If the entry exists, extends its ABI with the function list.
                        .and_modify(|entry| {
                            // Extend the ABI's function list with the new functions.
                            entry.abi.functions.extend(contract.abi.functions.clone());
                        })
                        // Otherwise insert it into the map.
                        .or_insert_with(|| {
                            TargetedContract::new(identifier.to_string(), contract.abi.clone())
                        });
                }
            }
        }

        targeted_contracts.extend(combined.inner);

        Ok(())
    }

    /// Selects the functions to fuzz based on the contract method `targetSelectors()` and
    /// `targetArtifactSelectors()`.
    pub fn select_selectors(
        &self,
        address: Address,
        targeted_contracts: &mut TargetedContracts,
    ) -> Result<()> {
        for (address, (identifier, _)) in self.setup_contracts.iter() {
            if let Some(selectors) = self.artifact_filters.targeted.get(identifier) {
                if selectors.is_empty() {
                    continue;
                }
                self.add_address_with_functions(*address, selectors, false, targeted_contracts)?;
            }
        }

        // Collect contract functions marked as target for fuzzing campaign.
        let selectors =
            self.executor.call_sol_default(address, &IInvariantTest::targetSelectorsCall {});
        for IInvariantTest::FuzzSelector { addr, selectors } in selectors.targetedSelectors {
            self.add_address_with_functions(addr, &selectors, false, targeted_contracts)?;
        }

        // Collect contract functions excluded from fuzzing campaign.
        let selectors =
            self.executor.call_sol_default(address, &IInvariantTest::excludeSelectorsCall {});
        for IInvariantTest::FuzzSelector { addr, selectors } in selectors.excludedSelectors {
            self.add_address_with_functions(addr, &selectors, true, targeted_contracts)?;
        }

        Ok(())
    }

    /// Adds the address and fuzzed or excluded functions to `TargetedContracts`.
    fn add_address_with_functions(
        &self,
        address: Address,
        selectors: &[Selector],
        should_exclude: bool,
        targeted_contracts: &mut TargetedContracts,
    ) -> eyre::Result<()> {
        let contract = match targeted_contracts.entry(address) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let (identifier, abi) = self.setup_contracts.get(&address).ok_or_else(|| {
                    eyre::eyre!(
                        "[{}] address does not have an associated contract: {}",
                        if should_exclude { "excludeSelectors" } else { "targetSelectors" },
                        address
                    )
                })?;
                entry.insert(TargetedContract::new(identifier.clone(), abi.clone()))
            }
        };
        contract.add_selectors(selectors.iter().copied(), should_exclude)?;
        Ok(())
    }
}

/// Collects data from call for fuzzing. However, it first verifies that the sender is not an EOA
/// before inserting it into the dictionary. Otherwise, we flood the dictionary with
/// randomly generated addresses.
fn collect_data(
    invariant_test: &InvariantTest,
    state_changeset: &mut HashMap<Address, revm::primitives::Account>,
    tx: &BasicTxDetails,
    call_result: &RawCallResult,
    run_depth: u32,
) {
    // Verify it has no code.
    let mut has_code = false;
    if let Some(Some(code)) =
        state_changeset.get(&tx.sender).map(|account| account.info.code.as_ref())
    {
        has_code = !code.is_empty();
    }

    // We keep the nonce changes to apply later.
    let mut sender_changeset = None;
    if !has_code {
        sender_changeset = state_changeset.remove(&tx.sender);
    }

    // Collect values from fuzzed call result and add them to fuzz dictionary.
    invariant_test.fuzz_state.collect_values_from_call(
        &invariant_test.targeted_contracts,
        tx,
        &call_result.result,
        &call_result.logs,
        &*state_changeset,
        run_depth,
    );

    // Re-add changes
    if let Some(changed) = sender_changeset {
        state_changeset.insert(tx.sender, changed);
    }
}

/// Calls the `afterInvariant()` function on a contract.
/// Returns call result and if call succeeded.
/// The state after the call is not persisted.
pub(crate) fn call_after_invariant_function(
    executor: &Executor,
    to: Address,
) -> std::result::Result<(RawCallResult, bool), EvmError> {
    let calldata = Bytes::from_static(&IInvariantTest::afterInvariantCall::SELECTOR);
    let mut call_result = executor.call_raw(CALLER, to, calldata, U256::ZERO)?;
    let success = executor.is_raw_call_mut_success(to, &mut call_result, false);
    Ok((call_result, success))
}

/// Calls the invariant function and returns call result and if succeeded.
pub(crate) fn call_invariant_function(
    executor: &Executor,
    address: Address,
    calldata: Bytes,
) -> Result<(RawCallResult, bool)> {
    let mut call_result = executor.call_raw(CALLER, address, calldata, U256::ZERO)?;
    let success = executor.is_raw_call_mut_success(address, &mut call_result, false);
    Ok((call_result, success))
}
