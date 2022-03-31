//! Test command
use crate::{
    cmd::{
        forge::{build::BuildArgs, run::RunArgs},
        Cmd,
    },
    opts::evm::EvmArgs,
    utils,
};
use ansi_term::Colour;
use clap::{AppSettings, Parser};
pub use config::RunTestConfig;
use filter::Filter;
use forge::{
    decode::decode_console_logs,
    executor::opts::EvmOpts,
    gas_report::GasReport,
    trace::{identifier::LocalTraceIdentifier, CallTraceDecoder, TraceKind},
    MultiContractRunner, MultiContractRunnerBuilder, SuiteResult, TestFilter, TestKind, TestResult,
};
use foundry_config::{figment::Figment, Config};
use regex::Regex;
use std::{
    collections::BTreeMap, path::PathBuf, str::FromStr, sync::mpsc::channel, thread, time::Duration,
};

mod config;
mod filter;

// Loads project's figment and merges the build cli arguments into it
foundry_config::impl_figment_convert!(TestArgs, opts, evm_opts);

#[derive(Debug, Clone, Parser)]
#[clap(global_setting = AppSettings::DeriveDisplayOrder)]
pub struct TestArgs {
    #[clap(flatten)]
    filter: Filter,

    /// Run a test in the debugger.
    ///
    /// The argument passed to this flag is the name of the test function you want to run, and it
    /// works the same as --match-test.
    ///
    /// If more than one test matches your specified criteria, you must add additional filters
    /// until only one test is found (see --match-contract and --match-path).
    ///
    /// The matching test will be opened in the debugger regardless of the outcome of the test.
    ///
    /// If the matching test is a fuzz test, then it will open the debugger on the first failure
    /// case.
    /// If the fuzz test does not fail, it will open the debugger on the last fuzz case.
    ///
    /// For more fine-grained control of which fuzz case is run, see forge run.
    #[clap(long, value_name = "TEST FUNCTION")]
    debug: Option<Regex>,

    /// Print a gas report.
    #[clap(long, env = "FORGE_GAS_REPORT")]
    gas_report: bool,

    /// Force the process to exit with code 0, even if the tests fail.
    #[clap(long, env = "FORGE_ALLOW_FAILURE")]
    allow_failure: bool,

    /// Output test results in JSON format.
    #[clap(long, short)]
    json: bool,

    #[clap(flatten, next_help_heading = "EVM OPTIONS")]
    evm_opts: EvmArgs,

    #[clap(flatten, next_help_heading = "BUILD OPTIONS")]
    opts: BuildArgs,
}

impl TestArgs {
    /// Returns the flattened [`BuildArgs`]
    pub fn build_args(&self) -> &BuildArgs {
        &self.opts
    }

    /// Returns the flattened [`Filter`] arguments
    pub fn filter(&self) -> &Filter {
        &self.filter
    }
}

impl Cmd for TestArgs {
    type Output = TestOutcome;

    fn run(self) -> eyre::Result<Self::Output> {
        self.run_with(Default::default())
    }
}

impl TestArgs {
    /// Returns the [Config] and the [EvmOpts] extracted from the config
    ///
    /// See also [foundry_config::impl_figment_convert!]
    pub fn config(&self) -> (Config, EvmOpts) {
        let figment: Figment = From::from(self);
        let evm_opts = figment.extract::<EvmOpts>().expect("EvmOpts always subset");
        let config = Config::from_provider(figment).sanitized();
        (config, evm_opts)
    }

    /// Execute all tests
    pub fn run_with(mut self, test_config: RunTestConfig) -> eyre::Result<TestOutcome> {
        let RunTestConfig { include_fuzz_tests } = test_config;

        let (config, mut evm_opts) = self.config();

        let fuzzer = proptest::test_runner::TestRunner::new(proptest_config(&config));

        // Set up the project
        let project = config.project()?;
        let output = crate::cmd::compile(&project, false, false)?;

        // Determine print verbosity and executor verbosity
        let verbosity = evm_opts.verbosity;
        if self.gas_report && evm_opts.verbosity < 3 {
            evm_opts.verbosity = 3;
        }

        // Prepare the test builder
        let evm_spec = crate::utils::evm_spec(&config.evm_version);
        let mut runner = MultiContractRunnerBuilder::default()
            .fuzzer(fuzzer)
            .initial_balance(evm_opts.initial_balance)
            .evm_spec(evm_spec)
            .sender(evm_opts.sender)
            .with_fork(utils::get_fork(&evm_opts, &config.rpc_storage_caching))
            .build(project.paths.root, output, evm_opts)?;

        if self.debug.is_some() {
          return self.run_debug(runner)
        }

        let TestArgs { filter, json, allow_failure, gas_report, .. } = self;

        test(
            runner,
            verbosity,
            filter,
            json,
            allow_failure,
            include_fuzz_tests,
            (gas_report, config.gas_reports),
        )
    }

    /// Runs the debugger
    ///
    /// # Errors
    ///
    /// This will fail if the configured [Filter] does _not_ exactly one! test
    fn run_debug(self, runner: MultiContractRunner) -> eyre::Result<TestOutcome> {
        let test_count = runner.count_filtered_tests(&self.filter);
        if test_count != 1 {
            eyre::bail!(
                    "{} tests matched your criteria, but exactly 1 test must match in order to run the debugger.\n
                        \n
                        Use --match-contract and --match-path to further limit the search.", test_count)
        }


        todo!()
    }
}

/// The result of a single test
#[derive(Debug, Clone)]
pub struct Test {
    /// The identifier of the artifact/contract in the form of `<artifact file name>:<contract
    /// name>`
    pub artifact_id: String,
    /// The signature of the solidity test
    pub signature: String,
    /// Result of the executed solidity test
    pub result: forge::TestResult,
}

impl Test {
    /// The gas consumed by this test
    pub fn gas_used(&self) -> u64 {
        self.result.kind.gas_used().gas()
    }

    /// Returns the contract name of the artifact id
    pub fn contract_name(&self) -> &str {
        utils::get_contract_name(&self.artifact_id)
    }

    /// Returns the file name of the artifact id
    pub fn file_name(&self) -> &str {
        utils::get_file_name(&self.artifact_id)
    }
}

/// Represents the bundled results of all tests and can be used to selectively ensure all tests are
/// good
pub struct TestOutcome {
    /// Whether failures are allowed
    pub allow_failure: bool,
    /// Results for each suite of tests `contract -> SuiteResult`
    pub results: BTreeMap<String, SuiteResult>,
}

impl TestOutcome {
    fn new(results: BTreeMap<String, SuiteResult>, allow_failure: bool) -> Self {
        Self { results, allow_failure }
    }

    /// Iterator over all succeeding tests and their names
    pub fn successes(&self) -> impl Iterator<Item = (&String, &forge::TestResult)> {
        self.tests().filter(|(_, t)| t.success)
    }

    /// Iterator over all failing tests and their names
    pub fn failures(&self) -> impl Iterator<Item = (&String, &forge::TestResult)> {
        self.tests().filter(|(_, t)| !t.success)
    }

    /// Iterator over all tests and their names
    pub fn tests(&self) -> impl Iterator<Item = (&String, &forge::TestResult)> {
        self.results.values().flat_map(|SuiteResult { test_results, .. }| test_results.iter())
    }

    /// Returns an iterator over all `Test`
    pub fn into_tests(self) -> impl Iterator<Item = Test> {
        self.results
            .into_iter()
            .flat_map(|(file, SuiteResult { test_results, .. })| {
                test_results.into_iter().map(move |t| (file.clone(), t))
            })
            .map(|(artifact_id, (signature, result))| Test { artifact_id, signature, result })
    }

    /// Checks if there are any failures and failures are disallowed
    ///
    /// This is a no-op if `allow_failure` is `true` otherwise this ensures that all tests are ok
    /// and exits the program otherwise.
    pub fn ensure_ok(&self) -> eyre::Result<()> {
        if !self.allow_failure {
            let failures = self.failures().count();
            if failures > 0 {
                println!();
                println!("Failed tests:");
                for (name, result) in self.failures() {
                    print_short_test_result(name, result);
                }
                println!();

                let successes = self.successes().count();
                println!(
                    "Encountered a total of {} failing tests, {} tests succeeded",
                    Colour::Red.paint(failures.to_string()),
                    Colour::Green.paint(successes.to_string())
                );
                std::process::exit(1);
            }
        }
        Ok(())
    }

    /// Returns the accumulated duration of all tests
    pub fn duration(&self) -> Duration {
        self.results
            .values()
            .fold(Duration::ZERO, |acc, SuiteResult { duration, .. }| acc + *duration)
    }

    /// Returns a report of all test results depending on their status.
    pub fn summary(&self) -> String {
        let failed = self.failures().count();
        let result =
            if failed == 0 { Colour::Green.paint("ok") } else { Colour::Red.paint("FAILED") };
        format!(
            "Test result: {}. {} passed; {} failed; finished in {:.2?}",
            result,
            self.successes().count(),
            failed,
            self.duration()
        )
    }
}

/// Computes the test result report and prints int to stdout.
fn print_short_test_result(name: &str, result: &forge::TestResult) {
    let status = if result.success {
        Colour::Green.paint("[PASS]")
    } else {
        let txt = match (&result.reason, &result.counterexample) {
            (Some(ref reason), Some(ref counterexample)) => {
                format!("[FAIL. Reason: {}. Counterexample: {}]", reason, counterexample)
            }
            (None, Some(ref counterexample)) => {
                format!("[FAIL. Counterexample: {}]", counterexample)
            }
            (Some(ref reason), None) => {
                format!("[FAIL. Reason: {}]", reason)
            }
            (None, None) => "[FAIL]".to_string(),
        };

        Colour::Red.paint(txt)
    };

    println!("{} {} {}", status, name, result.kind.gas_used());
}

pub fn custom_run(mut args: TestArgs, include_fuzz_tests: bool) -> eyre::Result<TestOutcome> {
    // Merge all configs
    let figment: Figment = From::from(&args);
    let mut evm_opts = figment.extract::<EvmOpts>()?;
    let config = Config::from_provider(figment).sanitized();

    // Setup the fuzzer
    // TODO: Add CLI Options to modify the persistence
    let cfg = proptest::test_runner::Config {
        failure_persistence: None,
        cases: config.fuzz_runs,
        max_local_rejects: config.fuzz_max_local_rejects,
        max_global_rejects: config.fuzz_max_global_rejects,
        ..Default::default()
    };
    let fuzzer = proptest::test_runner::TestRunner::new(cfg);

    // Set up the project
    let project = config.project()?;
    let output = crate::cmd::compile(&project, false, false)?;

    // Determine print verbosity and executor verbosity
    let verbosity = evm_opts.verbosity;
    if args.gas_report && evm_opts.verbosity < 3 {
        evm_opts.verbosity = 3;
    }

    // Prepare the test builder
    let evm_spec = crate::utils::evm_spec(&config.evm_version);
    let mut runner = MultiContractRunnerBuilder::default()
        .fuzzer(fuzzer)
        .initial_balance(evm_opts.initial_balance)
        .evm_spec(evm_spec)
        .sender(evm_opts.sender)
        .with_fork(utils::get_fork(&evm_opts, &config.rpc_storage_caching))
        .build(project.paths.root, output, evm_opts)?;

    if args.debug.is_some() {
        args.filter.test_pattern = args.debug;
        match runner.count_filtered_tests(&args.filter) {
                1 => {
                    // Run the test
                    let results = runner.test(&args.filter, None, true)?;

                    // Get the result of the single test
                    let (id, sig, test_kind, counterexample) = results.iter().map(|(id, SuiteResult{ test_results, .. })| {
                        let (sig, result) = test_results.iter().next().unwrap();

                        (id.clone(), sig.clone(), result.kind.clone(), result.counterexample.clone())
                    }).next().unwrap();

                    // Build debugger args if this is a fuzz test
                    let sig = match test_kind {
                        TestKind::Fuzz(cases) => {
                            if let Some(counterexample) = counterexample {
                                counterexample.calldata.to_string()
                            } else {
                                cases.cases().first().expect("no fuzz cases run").calldata.to_string()
                            }
                        },
                        _ => sig,
                    };

                    // Run the debugger
                    let debugger = RunArgs {
                        path: PathBuf::from(runner.source_paths.get(&id).unwrap()),
                        target_contract: Some(utils::get_contract_name(&id).to_string()),
                        sig,
                        args: Vec::new(),
                        debug: true,
                        opts: args.opts,
                        evm_opts: args.evm_opts,
                    };
                    debugger.run()?;

                    Ok(TestOutcome::new(results, args.allow_failure))
                }
                n =>
                    Err(
                    eyre::eyre!("{} tests matched your criteria, but exactly 1 test must match in order to run the debugger.\n
                        \n
                        Use --match-contract and --match-path to further limit the search.", n))
            }
    } else {
        let TestArgs { filter, .. } = args;
        test(
            runner,
            verbosity,
            filter,
            args.json,
            args.allow_failure,
            include_fuzz_tests,
            (args.gas_report, config.gas_reports),
        )
    }
}

/// Runs all the tests
fn test(
    mut runner: MultiContractRunner,
    verbosity: u8,
    filter: Filter,
    json: bool,
    allow_failure: bool,
    include_fuzz_tests: bool,
    (gas_reporting, gas_reports): (bool, Vec<String>),
) -> eyre::Result<TestOutcome> {
    if json {
        let results = runner.test(&filter, None, include_fuzz_tests)?;
        println!("{}", serde_json::to_string(&results)?);
        Ok(TestOutcome::new(results, allow_failure))
    } else {
        let local_identifier = LocalTraceIdentifier::new(&runner.known_contracts);
        let (tx, rx) = channel::<(String, SuiteResult)>();

        let handle =
            thread::spawn(move || runner.test(&filter, Some(tx), include_fuzz_tests).unwrap());

        let mut results: BTreeMap<String, SuiteResult> = BTreeMap::new();
        let mut gas_report = GasReport::new(gas_reports);
        for (contract_name, suite_result) in rx {
            let mut tests = suite_result.test_results.clone();
            println!();
            if !tests.is_empty() {
                let term = if tests.len() > 1 { "tests" } else { "test" };
                println!("Running {} {} for {}", tests.len(), term, contract_name);
            }
            for (name, result) in &mut tests {
                print_short_test_result(name, result);

                // We only display logs at level 2 and above
                if verbosity >= 2 {
                    // We only decode logs from Hardhat and DS-style console events
                    let console_logs = decode_console_logs(&result.logs);
                    if !console_logs.is_empty() {
                        println!("Logs:");
                        for log in console_logs {
                            println!("  {}", log);
                        }
                        println!();
                    }
                }

                if !result.traces.is_empty() {
                    // Identify addresses in each trace
                    let mut decoder =
                        CallTraceDecoder::new_with_labels(result.labeled_addresses.clone());

                    // Decode the traces
                    let mut decoded_traces = Vec::new();
                    for (kind, trace) in &mut result.traces {
                        decoder.identify(trace, &local_identifier);

                        let should_include = match kind {
                            // At verbosity level 3, we only display traces for failed tests
                            // At verbosity level 4, we also display the setup trace for failed
                            // tests At verbosity level 5, we display
                            // all traces for all tests
                            TraceKind::Setup => {
                                (verbosity >= 5) || (verbosity == 4 && !result.success)
                            }
                            TraceKind::Execution => {
                                verbosity > 3 || (verbosity == 3 && !result.success)
                            }
                            _ => false,
                        };

                        // We decode the trace if we either need to build a gas report or we need
                        // to print it
                        if should_include || gas_reporting {
                            decoder.decode(trace);
                        }

                        if should_include {
                            decoded_traces.push(trace.to_string());
                        }
                    }

                    if !decoded_traces.is_empty() {
                        println!("Traces:");
                        decoded_traces.into_iter().for_each(|trace| println!("{}", trace));
                    }

                    if gas_reporting {
                        gas_report.analyze(&result.traces);
                    }
                }
            }
            let block_outcome = TestOutcome::new(
                [(contract_name.clone(), suite_result.clone())].into(),
                allow_failure,
            );
            println!("{}", block_outcome.summary());
            results.insert(contract_name, suite_result);
        }

        if gas_reporting {
            println!("{}", gas_report.finalize());
        }

        // reattach the thread
        let _ = handle.join();

        Ok(TestOutcome::new(results, allow_failure))
    }
}

/// Returns the fuzzer config
pub fn proptest_config(config: &Config) -> proptest::test_runner::Config {
    // TODO: Add CLI Options to modify the persistence
    proptest::test_runner::Config {
        failure_persistence: None,
        cases: config.fuzz_runs,
        max_local_rejects: config.fuzz_max_local_rejects,
        max_global_rejects: config.fuzz_max_global_rejects,
        ..Default::default()
    }
}
