//! Test outcomes.

use crate::{
    fuzz::{BaseCounterExample, FuzzedCases},
    gas_report::GasReport,
};
use alloy_primitives::{
    map::{AddressHashMap, HashMap},
    Address, Log,
};
use eyre::Report;
use foundry_common::{evm::Breakpoints, get_contract_name, get_file_name, shell};
use foundry_evm::{
    coverage::HitMaps,
    decode::SkipReason,
    executors::{invariant::InvariantMetrics, RawCallResult},
    fuzz::{CounterExample, FuzzCase, FuzzFixtures, FuzzTestResult},
    traces::{CallTraceArena, CallTraceDecoder, TraceKind, Traces},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap as Map},
    fmt::{self, Write},
    time::Duration,
};
use yansi::Paint;

/// The aggregated result of a test run.
#[derive(Clone, Debug)]
pub struct TestOutcome {
    /// The results of all test suites by their identifier (`path:contract_name`).
    ///
    /// Essentially `identifier => signature => result`.
    pub results: BTreeMap<String, SuiteResult>,
    /// Whether to allow test failures without failing the entire test run.
    pub allow_failure: bool,
    /// The decoder used to decode traces and logs.
    ///
    /// This is `None` if traces and logs were not decoded.
    ///
    /// Note that `Address` fields only contain the last executed test case's data.
    pub last_run_decoder: Option<CallTraceDecoder>,
    /// The gas report, if requested.
    pub gas_report: Option<GasReport>,
}

impl TestOutcome {
    /// Creates a new test outcome with the given results.
    pub fn new(results: BTreeMap<String, SuiteResult>, allow_failure: bool) -> Self {
        Self { results, allow_failure, last_run_decoder: None, gas_report: None }
    }

    /// Creates a new empty test outcome.
    pub fn empty(allow_failure: bool) -> Self {
        Self::new(BTreeMap::new(), allow_failure)
    }

    /// Returns an iterator over all individual succeeding tests and their names.
    pub fn successes(&self) -> impl Iterator<Item = (&String, &TestResult)> {
        self.tests().filter(|(_, t)| t.status.is_success())
    }

    /// Returns an iterator over all individual skipped tests and their names.
    pub fn skips(&self) -> impl Iterator<Item = (&String, &TestResult)> {
        self.tests().filter(|(_, t)| t.status.is_skipped())
    }

    /// Returns an iterator over all individual failing tests and their names.
    pub fn failures(&self) -> impl Iterator<Item = (&String, &TestResult)> {
        self.tests().filter(|(_, t)| t.status.is_failure())
    }

    /// Returns an iterator over all individual tests and their names.
    pub fn tests(&self) -> impl Iterator<Item = (&String, &TestResult)> {
        self.results.values().flat_map(|suite| suite.tests())
    }

    /// Flattens the test outcome into a list of individual tests.
    // TODO: Replace this with `tests` and make it return `TestRef<'_>`
    pub fn into_tests_cloned(&self) -> impl Iterator<Item = SuiteTestResult> + '_ {
        self.results
            .iter()
            .flat_map(|(file, suite)| {
                suite
                    .test_results
                    .iter()
                    .map(move |(sig, result)| (file.clone(), sig.clone(), result.clone()))
            })
            .map(|(artifact_id, signature, result)| SuiteTestResult {
                artifact_id,
                signature,
                result,
            })
    }

    /// Flattens the test outcome into a list of individual tests.
    pub fn into_tests(self) -> impl Iterator<Item = SuiteTestResult> {
        self.results
            .into_iter()
            .flat_map(|(file, suite)| {
                suite.test_results.into_iter().map(move |t| (file.clone(), t))
            })
            .map(|(artifact_id, (signature, result))| SuiteTestResult {
                artifact_id,
                signature,
                result,
            })
    }

    /// Returns the number of tests that passed.
    pub fn passed(&self) -> usize {
        self.successes().count()
    }

    /// Returns the number of tests that were skipped.
    pub fn skipped(&self) -> usize {
        self.skips().count()
    }

    /// Returns the number of tests that failed.
    pub fn failed(&self) -> usize {
        self.failures().count()
    }

    /// Sums up all the durations of all individual test suites.
    ///
    /// Note that this is not necessarily the wall clock time of the entire test run.
    pub fn total_time(&self) -> Duration {
        self.results.values().map(|suite| suite.duration).sum()
    }

    /// Formats the aggregated summary of all test suites into a string (for printing).
    pub fn summary(&self, wall_clock_time: Duration) -> String {
        let num_test_suites = self.results.len();
        let suites = if num_test_suites == 1 { "suite" } else { "suites" };
        let total_passed = self.passed();
        let total_failed = self.failed();
        let total_skipped = self.skipped();
        let total_tests = total_passed + total_failed + total_skipped;
        format!(
            "\nRan {} test {} in {:.2?} ({:.2?} CPU time): {} tests passed, {} failed, {} skipped ({} total tests)",
            num_test_suites,
            suites,
            wall_clock_time,
            self.total_time(),
            total_passed.green(),
            total_failed.red(),
            total_skipped.yellow(),
            total_tests
        )
    }

    /// Checks if there are any failures and failures are disallowed.
    pub fn ensure_ok(&self, silent: bool) -> eyre::Result<()> {
        let outcome = self;
        let failures = outcome.failures().count();
        if outcome.allow_failure || failures == 0 {
            return Ok(());
        }

        if shell::is_quiet() || silent {
            // TODO: Avoid process::exit
            std::process::exit(1);
        }

        sh_println!("\nFailing tests:")?;
        for (suite_name, suite) in outcome.results.iter() {
            let failed = suite.failed();
            if failed == 0 {
                continue;
            }

            let term = if failed > 1 { "tests" } else { "test" };
            sh_println!("Encountered {failed} failing {term} in {suite_name}")?;
            for (name, result) in suite.failures() {
                sh_println!("{}", result.short_result(name))?;
            }
            sh_println!()?;
        }
        let successes = outcome.passed();
        sh_println!(
            "Encountered a total of {} failing tests, {} tests succeeded",
            failures.to_string().red(),
            successes.to_string().green()
        )?;

        // TODO: Avoid process::exit
        std::process::exit(1);
    }

    /// Removes first test result, if any.
    pub fn remove_first(&mut self) -> Option<(String, String, TestResult)> {
        self.results.iter_mut().find_map(|(suite_name, suite)| {
            if let Some(test_name) = suite.test_results.keys().next().cloned() {
                let result = suite.test_results.remove(&test_name).unwrap();
                Some((suite_name.clone(), test_name, result))
            } else {
                None
            }
        })
    }
}

/// A set of test results for a single test suite, which is all the tests in a single contract.
#[derive(Clone, Debug, Serialize)]
pub struct SuiteResult {
    /// Wall clock time it took to execute all tests in this suite.
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    /// Individual test results: `test fn signature -> TestResult`.
    pub test_results: BTreeMap<String, TestResult>,
    /// Generated warnings.
    pub warnings: Vec<String>,
}

impl SuiteResult {
    pub fn new(
        duration: Duration,
        test_results: BTreeMap<String, TestResult>,
        mut warnings: Vec<String>,
    ) -> Self {
        // Add deprecated cheatcodes warning, if any of them used in current test suite.
        let mut deprecated_cheatcodes = HashMap::new();
        for test_result in test_results.values() {
            deprecated_cheatcodes.extend(test_result.deprecated_cheatcodes.clone());
        }
        if !deprecated_cheatcodes.is_empty() {
            let mut warning =
                "the following cheatcode(s) are deprecated and will be removed in future versions:"
                    .to_string();
            for (cheatcode, reason) in deprecated_cheatcodes {
                write!(warning, "\n  {cheatcode}").unwrap();
                if let Some(reason) = reason {
                    write!(warning, ": {reason}").unwrap();
                }
            }
            warnings.push(warning);
        }

        let mut isolated_cheatcodes = BTreeSet::new();
        for test_result in test_results.values() {
            isolated_cheatcodes.extend(test_result.isolated_cheatcodes.clone());
        }
        if !isolated_cheatcodes.is_empty() {
            let mut warning =
                "the following cheatcode(s) require isolation mode and were skipped:".to_string();
            for cheatcode in isolated_cheatcodes {
                write!(warning, "\n  {cheatcode}").unwrap();
            }
            write!(warning, "\n\nTo enable isolation mode, pass the `--isolated` flag\n").unwrap();
            warnings.push(warning);
        }

        Self { duration, test_results, warnings }
    }

    /// Returns an iterator over all individual succeeding tests and their names.
    pub fn successes(&self) -> impl Iterator<Item = (&String, &TestResult)> {
        self.tests().filter(|(_, t)| t.status.is_success())
    }

    /// Returns an iterator over all individual skipped tests and their names.
    pub fn skips(&self) -> impl Iterator<Item = (&String, &TestResult)> {
        self.tests().filter(|(_, t)| t.status.is_skipped())
    }

    /// Returns an iterator over all individual failing tests and their names.
    pub fn failures(&self) -> impl Iterator<Item = (&String, &TestResult)> {
        self.tests().filter(|(_, t)| t.status.is_failure())
    }

    /// Returns the number of tests that passed.
    pub fn passed(&self) -> usize {
        self.successes().count()
    }

    /// Returns the number of tests that were skipped.
    pub fn skipped(&self) -> usize {
        self.skips().count()
    }

    /// Returns the number of tests that failed.
    pub fn failed(&self) -> usize {
        self.failures().count()
    }

    /// Iterator over all tests and their names
    pub fn tests(&self) -> impl Iterator<Item = (&String, &TestResult)> {
        self.test_results.iter()
    }

    /// Whether this test suite is empty.
    pub fn is_empty(&self) -> bool {
        self.test_results.is_empty()
    }

    /// The number of tests in this test suite.
    pub fn len(&self) -> usize {
        self.test_results.len()
    }

    /// Sums up all the durations of all individual tests in this suite.
    ///
    /// Note that this is not necessarily the wall clock time of the entire test suite.
    pub fn total_time(&self) -> Duration {
        self.test_results.values().map(|result| result.duration).sum()
    }

    /// Returns the summary of a single test suite.
    pub fn summary(&self) -> String {
        let failed = self.failed();
        let result = if failed == 0 { "ok".green() } else { "FAILED".red() };
        format!(
            "Suite result: {}. {} passed; {} failed; {} skipped; finished in {:.2?} ({:.2?} CPU time)",
            result,
            self.passed().green(),
            failed.red(),
            self.skipped().yellow(),
            self.duration,
            self.total_time(),
        )
    }
}

/// The result of a single test in a test suite.
///
/// This is flattened from a [`TestOutcome`].
#[derive(Clone, Debug)]
pub struct SuiteTestResult {
    /// The identifier of the artifact/contract in the form:
    /// `<artifact file name>:<contract name>`.
    pub artifact_id: String,
    /// The function signature of the Solidity test.
    pub signature: String,
    /// The result of the executed test.
    pub result: TestResult,
}

impl SuiteTestResult {
    /// Returns the gas used by the test.
    pub fn gas_used(&self) -> u64 {
        self.result.kind.report().gas()
    }

    /// Returns the contract name of the artifact ID.
    pub fn contract_name(&self) -> &str {
        get_contract_name(&self.artifact_id)
    }

    /// Returns the file name of the artifact ID.
    pub fn file_name(&self) -> &str {
        get_file_name(&self.artifact_id)
    }
}

/// The status of a test.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    Success,
    #[default]
    Failure,
    Skipped,
}

impl TestStatus {
    /// Returns `true` if the test was successful.
    #[inline]
    pub fn is_success(self) -> bool {
        matches!(self, Self::Success)
    }

    /// Returns `true` if the test failed.
    #[inline]
    pub fn is_failure(self) -> bool {
        matches!(self, Self::Failure)
    }

    /// Returns `true` if the test was skipped.
    #[inline]
    pub fn is_skipped(self) -> bool {
        matches!(self, Self::Skipped)
    }
}

/// The result of an executed test.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TestResult {
    /// The test status, indicating whether the test case succeeded, failed, or was marked as
    /// skipped. This means that the transaction executed properly, the test was marked as
    /// skipped with vm.skip(), or that there was a revert and that the test was expected to
    /// fail (prefixed with `testFail`)
    pub status: TestStatus,

    /// If there was a revert, this field will be populated. Note that the test can
    /// still be successful (i.e self.success == true) when it's expected to fail.
    pub reason: Option<String>,

    /// Minimal reproduction test case for failing test
    pub counterexample: Option<CounterExample>,

    /// Any captured & parsed as strings logs along the test's execution which should
    /// be printed to the user.
    pub logs: Vec<Log>,

    /// The decoded DSTest logging events and Hardhat's `console.log` from [logs](Self::logs).
    /// Used for json output.
    pub decoded_logs: Vec<String>,

    /// What kind of test this was
    pub kind: TestKind,

    /// Traces
    pub traces: Traces,

    /// Additional traces to use for gas report.
    #[serde(skip)]
    pub gas_report_traces: Vec<Vec<CallTraceArena>>,

    /// Raw coverage info
    #[serde(skip)]
    pub coverage: Option<HitMaps>,

    /// Labeled addresses
    pub labeled_addresses: AddressHashMap<String>,

    pub duration: Duration,

    /// pc breakpoint char map
    pub breakpoints: Breakpoints,

    /// Any captured gas snapshots along the test's execution which should be accumulated.
    pub gas_snapshots: BTreeMap<String, BTreeMap<String, String>>,

    /// Deprecated cheatcodes (mapped to their replacements, if any) used in current test.
    #[serde(skip)]
    pub deprecated_cheatcodes: HashMap<&'static str, Option<&'static str>>,

    /// Isolated cheatcodes used in current test.
    #[serde(skip)]
    pub isolated_cheatcodes: Vec<&'static str>,
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            TestStatus::Success => "[PASS]".green().fmt(f),
            TestStatus::Skipped => {
                let mut s = String::from("[SKIP");
                if let Some(reason) = &self.reason {
                    write!(s, ": {reason}").unwrap();
                }
                s.push(']');
                s.yellow().fmt(f)
            }
            TestStatus::Failure => {
                let mut s = String::from("[FAIL");
                if self.reason.is_some() || self.counterexample.is_some() {
                    if let Some(reason) = &self.reason {
                        write!(s, ": {reason}").unwrap();
                    }

                    if let Some(counterexample) = &self.counterexample {
                        match counterexample {
                            CounterExample::Single(ex) => {
                                write!(s, "; counterexample: {ex}]").unwrap();
                            }
                            CounterExample::Sequence(sequence) => {
                                s.push_str("]\n\t[Sequence]\n");
                                for ex in sequence {
                                    writeln!(s, "\t\t{ex}").unwrap();
                                }
                            }
                        }
                    } else {
                        s.push(']');
                    }
                } else {
                    s.push(']');
                }
                s.red().fmt(f)
            }
        }
    }
}

impl TestResult {
    /// Creates a new test result starting from test setup results.
    pub fn new(setup: &TestSetup) -> Self {
        Self {
            labeled_addresses: setup.labels.clone(),
            logs: setup.logs.clone(),
            traces: setup.traces.clone(),
            coverage: setup.coverage.clone(),
            ..Default::default()
        }
    }

    /// Creates a failed test result with given reason.
    pub fn fail(reason: String) -> Self {
        Self { status: TestStatus::Failure, reason: Some(reason), ..Default::default() }
    }

    /// Creates a test setup result.
    pub fn setup_result(setup: TestSetup) -> Self {
        Self {
            status: if setup.skipped { TestStatus::Skipped } else { TestStatus::Failure },
            reason: setup.reason,
            logs: setup.logs,
            traces: setup.traces,
            coverage: setup.coverage,
            labeled_addresses: setup.labels,
            ..Default::default()
        }
    }

    /// Returns the skipped result for single test (used in skipped fuzz test too).
    pub fn single_skip(&mut self, reason: SkipReason) {
        self.status = TestStatus::Skipped;
        self.reason = reason.0;
    }

    /// Returns the failed result with reason for single test.
    pub fn single_fail(&mut self, reason: Option<String>) {
        self.status = TestStatus::Failure;
        self.reason = reason;
    }

    /// Returns the result for single test. Merges execution results (logs, labeled addresses,
    /// traces and coverages) in initial setup results.
    pub fn single_result(
        &mut self,
        success: bool,
        reason: Option<String>,
        raw_call_result: RawCallResult,
    ) {
        self.kind =
            TestKind::Unit { gas: raw_call_result.gas_used.wrapping_sub(raw_call_result.stipend) };

        // Record logs, labels, traces and merge coverages.
        self.logs.extend(raw_call_result.logs);
        self.labeled_addresses.extend(raw_call_result.labels);
        self.traces.extend(raw_call_result.traces.map(|traces| (TraceKind::Execution, traces)));
        self.merge_coverages(raw_call_result.coverage);

        self.status = match success {
            true => TestStatus::Success,
            false => TestStatus::Failure,
        };
        self.reason = reason;
        self.duration = Duration::default();
        self.gas_report_traces = Vec::new();

        if let Some(cheatcodes) = raw_call_result.cheatcodes {
            self.breakpoints = cheatcodes.breakpoints;
            self.gas_snapshots = cheatcodes.gas_snapshots;
            self.deprecated_cheatcodes = cheatcodes.deprecated_cheatcodes;
            self.isolated_cheatcodes = cheatcodes.isolated_cheatcodes;
        }
    }

    /// Returns the result for a fuzzed test. Merges fuzz execution results (logs, labeled
    /// addresses, traces and coverages) in initial setup results.
    pub fn fuzz_result(&mut self, result: FuzzTestResult) {
        self.kind = TestKind::Fuzz {
            median_gas: result.median_gas(false),
            mean_gas: result.mean_gas(false),
            first_case: result.first_case,
            runs: result.gas_by_case.len(),
        };

        // Record logs, labels, traces and merge coverages.
        self.logs.extend(result.logs);
        self.labeled_addresses.extend(result.labeled_addresses);
        self.traces.extend(result.traces.map(|traces| (TraceKind::Execution, traces)));
        self.merge_coverages(result.coverage);

        self.status = if result.skipped {
            TestStatus::Skipped
        } else if result.success {
            TestStatus::Success
        } else {
            TestStatus::Failure
        };
        self.reason = result.reason;
        self.counterexample = result.counterexample;
        self.duration = Duration::default();
        self.gas_report_traces = result.gas_report_traces.into_iter().map(|t| vec![t]).collect();
        self.breakpoints = result.breakpoints.unwrap_or_default();
        self.deprecated_cheatcodes = result.deprecated_cheatcodes;
    }

    /// Returns the skipped result for invariant test.
    pub fn invariant_skip(&mut self, reason: SkipReason) {
        self.kind =
            TestKind::Invariant { runs: 1, calls: 1, reverts: 1, metrics: HashMap::default() };
        self.status = TestStatus::Skipped;
        self.reason = reason.0;
    }

    /// Returns the fail result for replayed invariant test.
    pub fn invariant_replay_fail(
        &mut self,
        replayed_entirely: bool,
        invariant_name: &String,
        call_sequence: Vec<BaseCounterExample>,
    ) {
        self.kind =
            TestKind::Invariant { runs: 1, calls: 1, reverts: 1, metrics: HashMap::default() };
        self.status = TestStatus::Failure;
        self.reason = if replayed_entirely {
            Some(format!("{invariant_name} replay failure"))
        } else {
            Some(format!("{invariant_name} persisted failure revert"))
        };
        self.counterexample = Some(CounterExample::Sequence(call_sequence));
    }

    /// Returns the fail result for invariant test setup.
    pub fn invariant_setup_fail(&mut self, e: Report) {
        self.kind =
            TestKind::Invariant { runs: 0, calls: 0, reverts: 0, metrics: HashMap::default() };
        self.status = TestStatus::Failure;
        self.reason = Some(format!("failed to set up invariant testing environment: {e}"));
    }

    /// Returns the invariant test result.
    #[allow(clippy::too_many_arguments)]
    pub fn invariant_result(
        &mut self,
        gas_report_traces: Vec<Vec<CallTraceArena>>,
        success: bool,
        reason: Option<String>,
        counterexample: Option<CounterExample>,
        cases: Vec<FuzzedCases>,
        reverts: usize,
        metrics: Map<String, InvariantMetrics>,
    ) {
        self.kind = TestKind::Invariant {
            runs: cases.len(),
            calls: cases.iter().map(|sequence| sequence.cases().len()).sum(),
            reverts,
            metrics,
        };
        self.status = match success {
            true => TestStatus::Success,
            false => TestStatus::Failure,
        };
        self.reason = reason;
        self.counterexample = counterexample;
        self.gas_report_traces = gas_report_traces;
    }

    /// Returns `true` if this is the result of a fuzz test
    pub fn is_fuzz(&self) -> bool {
        matches!(self.kind, TestKind::Fuzz { .. })
    }

    /// Formats the test result into a string (for printing).
    pub fn short_result(&self, name: &str) -> String {
        format!("{self} {name} {}", self.kind.report())
    }

    /// Merges the given raw call result into `self`.
    pub fn extend(&mut self, call_result: RawCallResult) {
        self.logs.extend(call_result.logs);
        self.labeled_addresses.extend(call_result.labels);
        self.traces.extend(call_result.traces.map(|traces| (TraceKind::Execution, traces)));
        self.merge_coverages(call_result.coverage);
    }

    /// Merges the given coverage result into `self`.
    pub fn merge_coverages(&mut self, other_coverage: Option<HitMaps>) {
        HitMaps::merge_opt(&mut self.coverage, other_coverage);
    }
}

/// Data report by a test.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestKindReport {
    Unit { gas: u64 },
    Fuzz { runs: usize, mean_gas: u64, median_gas: u64 },
    Invariant { runs: usize, calls: usize, reverts: usize, metrics: Map<String, InvariantMetrics> },
}

impl fmt::Display for TestKindReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit { gas } => {
                write!(f, "(gas: {gas})")
            }
            Self::Fuzz { runs, mean_gas, median_gas } => {
                write!(f, "(runs: {runs}, μ: {mean_gas}, ~: {median_gas})")
            }
            Self::Invariant { runs, calls, reverts, metrics: _ } => {
                write!(f, "(runs: {runs}, calls: {calls}, reverts: {reverts})")
            }
        }
    }
}

impl TestKindReport {
    /// Returns the main gas value to compare against
    pub fn gas(&self) -> u64 {
        match *self {
            Self::Unit { gas } => gas,
            // We use the median for comparisons
            Self::Fuzz { median_gas, .. } => median_gas,
            // We return 0 since it's not applicable
            Self::Invariant { .. } => 0,
        }
    }
}

/// Various types of tests
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TestKind {
    /// A unit test.
    Unit { gas: u64 },
    /// A fuzz test.
    Fuzz {
        /// we keep this for the debugger
        first_case: FuzzCase,
        runs: usize,
        mean_gas: u64,
        median_gas: u64,
    },
    /// An invariant test.
    Invariant { runs: usize, calls: usize, reverts: usize, metrics: Map<String, InvariantMetrics> },
}

impl Default for TestKind {
    fn default() -> Self {
        Self::Unit { gas: 0 }
    }
}

impl TestKind {
    /// The gas consumed by this test
    pub fn report(&self) -> TestKindReport {
        match self {
            Self::Unit { gas } => TestKindReport::Unit { gas: *gas },
            Self::Fuzz { first_case: _, runs, mean_gas, median_gas } => {
                TestKindReport::Fuzz { runs: *runs, mean_gas: *mean_gas, median_gas: *median_gas }
            }
            Self::Invariant { runs, calls, reverts, metrics: _ } => TestKindReport::Invariant {
                runs: *runs,
                calls: *calls,
                reverts: *reverts,
                metrics: HashMap::default(),
            },
        }
    }
}

/// The result of a test setup.
///
/// Includes the deployment of the required libraries and the test contract itself, and the call to
/// the `setUp()` function.
#[derive(Clone, Debug, Default)]
pub struct TestSetup {
    /// The address at which the test contract was deployed.
    pub address: Address,
    /// Defined fuzz test fixtures.
    pub fuzz_fixtures: FuzzFixtures,

    /// The logs emitted during setup.
    pub logs: Vec<Log>,
    /// Addresses labeled during setup.
    pub labels: AddressHashMap<String>,
    /// Call traces of the setup.
    pub traces: Traces,
    /// Coverage info during setup.
    pub coverage: Option<HitMaps>,
    /// Addresses of external libraries deployed during setup.
    pub deployed_libs: Vec<Address>,

    /// The reason the setup failed, if it did.
    pub reason: Option<String>,
    /// Whether setup and entire test suite is skipped.
    pub skipped: bool,
}

impl TestSetup {
    pub fn failed(reason: String) -> Self {
        Self { reason: Some(reason), ..Default::default() }
    }

    pub fn skipped(reason: String) -> Self {
        Self { reason: Some(reason), skipped: true, ..Default::default() }
    }

    pub fn extend(&mut self, raw: RawCallResult, trace_kind: TraceKind) {
        self.logs.extend(raw.logs);
        self.labels.extend(raw.labels);
        self.traces.extend(raw.traces.map(|traces| (trace_kind, traces)));
        HitMaps::merge_opt(&mut self.coverage, raw.coverage);
    }
}
