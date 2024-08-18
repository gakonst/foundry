use super::{install, watch::WatchArgs};
use alloy_primitives::U256;
use clap::Parser;
use eyre::Result;
use forge::{
    decode::decode_console_logs,
    gas_report::GasReport,
    multi_runner::matches_contract,
    result::{SuiteResult, TestOutcome, TestStatus},
    traces::{
        debug::{ContractSources, DebugTraceIdentifier},
        decode_trace_arena,
        identifier::SignaturesIdentifier,
        render_trace_arena, CallTraceDecoderBuilder, InternalTraceMode, TraceKind,
    },
    MultiContractRunner, MultiContractRunnerBuilder, TestFilter, TestOptions, TestOptionsBuilder,
};
use foundry_cli::{
    opts::CoreBuildArgs,
    utils::{self, LoadConfig},
};
use foundry_common::{compile::ProjectCompiler, evm::EvmArgs, fs, shell};
use foundry_compilers::{
    artifacts::output_selection::OutputSelection,
    compilers::{multi::MultiCompilerLanguage, CompilerSettings, Language},
    utils::source_files_iter,
    ProjectCompileOutput,
};
use foundry_config::{
    figment,
    figment::{
        value::{Dict, Map},
        Metadata, Profile, Provider,
    },
    get_available_profiles, Config,
};
use foundry_debugger::Debugger;
use foundry_evm::traces::identifier::TraceIdentifiers;
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::{mpsc::channel, Arc},
    time::Instant,
};
use yansi::Paint;

mod filter;
mod summary;
use summary::TestSummaryReporter;

pub use filter::{FilterArgs, ProjectPathsAwareFilter};

// Loads project's figment and merges the build cli arguments into it
foundry_config::merge_impl_figment_convert!(TestArgs, opts, evm_opts);

/// CLI arguments for `forge test`.
#[derive(Clone, Debug, Parser)]
#[command(next_help_heading = "Test options")]
pub struct TestArgs {
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
    #[arg(long, value_name = "TEST_FUNCTION")]
    debug: Option<Regex>,

    /// Whether to identify internal functions in traces.
    ///
    /// If no argument is passed to this flag, it will trace internal functions scope and decode
    /// stack parameters, but parameters stored in memory (such as bytes or arrays) will not be
    /// decoded.
    ///
    /// To decode memory parameters, you should pass an argument with a test function name,
    /// similarly to --debug and --match-test.
    ///
    /// If more than one test matches your specified criteria, you must add additional filters
    /// until only one test is found (see --match-contract and --match-path).
    #[arg(long, value_name = "TEST_FUNCTION")]
    decode_internal: Option<Option<Regex>>,

    /// Print a gas report.
    #[arg(long, env = "FORGE_GAS_REPORT")]
    gas_report: bool,

    /// Exit with code 0 even if a test fails.
    #[arg(long, env = "FORGE_ALLOW_FAILURE")]
    allow_failure: bool,

    /// Output test results in JSON format.
    #[arg(long, help_heading = "Display options")]
    json: bool,

    /// Stop running tests after the first failure.
    #[arg(long)]
    pub fail_fast: bool,

    /// The Etherscan (or equivalent) API key.
    #[arg(long, env = "ETHERSCAN_API_KEY", value_name = "KEY")]
    etherscan_api_key: Option<String>,

    /// List tests instead of running them.
    #[arg(long, short, help_heading = "Display options")]
    list: bool,

    /// Set seed used to generate randomness during your fuzz runs.
    #[arg(long)]
    pub fuzz_seed: Option<U256>,

    #[arg(long, env = "FOUNDRY_FUZZ_RUNS", value_name = "RUNS")]
    pub fuzz_runs: Option<u64>,

    /// File to rerun fuzz failures from.
    #[arg(long)]
    pub fuzz_input_file: Option<String>,

    /// Max concurrent threads to use.
    /// Default value is the number of available CPUs.
    #[arg(long, short = 'j', visible_alias = "jobs")]
    pub threads: Option<usize>,

    /// Show test execution progress.
    #[arg(long)]
    pub show_progress: bool,

    #[command(flatten)]
    filter: FilterArgs,

    /// Re-run recorded test failures from last run.
    /// If no failure recorded then regular test run is performed.
    #[arg(long)]
    pub rerun: bool,

    #[command(flatten)]
    evm_opts: EvmArgs,

    #[command(flatten)]
    opts: CoreBuildArgs,

    #[command(flatten)]
    pub watch: WatchArgs,

    /// Print test summary table.
    #[arg(long, help_heading = "Display options")]
    pub summary: bool,

    /// Print detailed test summary table.
    #[arg(long, help_heading = "Display options", requires = "summary")]
    pub detailed: bool,
}

impl TestArgs {
    /// Returns the flattened [`CoreBuildArgs`].
    pub fn build_args(&self) -> &CoreBuildArgs {
        &self.opts
    }

    pub async fn run(self) -> Result<TestOutcome> {
        trace!(target: "forge::test", "executing test command");
        shell::set_shell(shell::Shell::from_args(self.opts.silent, self.json))?;
        self.execute_tests().await
    }

    /// Returns sources which include any tests to be executed.
    /// If no filters are provided, sources are filtered by existence of test/invariant methods in
    /// them, If filters are provided, sources are additionaly filtered by them.
    pub fn get_sources_to_compile(
        &self,
        config: &Config,
        filter: &ProjectPathsAwareFilter,
    ) -> Result<BTreeSet<PathBuf>> {
        let mut project = config.create_project(true, true)?;
        project.settings.update_output_selection(|selection| {
            *selection = OutputSelection::common_output_selection(["abi".to_string()]);
        });

        let output = project.compile()?;

        if output.has_compiler_errors() {
            println!("{output}");
            eyre::bail!("Compilation failed");
        }

        // ABIs of all sources
        let abis = output
            .into_artifacts()
            .filter_map(|(id, artifact)| artifact.abi.map(|abi| (id, abi)))
            .collect::<BTreeMap<_, _>>();

        // Filter sources by their abis and contract names.
        let mut test_sources = abis
            .iter()
            .filter(|(id, abi)| matches_contract(id, abi, filter))
            .map(|(id, _)| id.source.clone())
            .collect::<BTreeSet<_>>();

        if test_sources.is_empty() {
            if filter.is_empty() {
                println!(
                    "No tests found in project! \
                        Forge looks for functions that starts with `test`."
                );
            } else {
                println!("No tests match the provided pattern:");
                print!("{filter}");

                // Try to suggest a test when there's no match
                if let Some(test_pattern) = &filter.args().test_pattern {
                    let test_name = test_pattern.as_str();
                    let candidates = abis
                        .into_iter()
                        .filter(|(id, _)| {
                            filter.matches_path(&id.source) && filter.matches_contract(&id.name)
                        })
                        .flat_map(|(_, abi)| abi.functions.into_keys())
                        .collect::<Vec<_>>();
                    if let Some(suggestion) = utils::did_you_mean(test_name, candidates).pop() {
                        println!("\nDid you mean `{suggestion}`?");
                    }
                }
            }

            eyre::bail!("No tests to run");
        }

        // Always recompile all sources to ensure that `getCode` cheatcode can use any artifact.
        test_sources.extend(source_files_iter(
            &project.paths.sources,
            MultiCompilerLanguage::FILE_EXTENSIONS,
        ));

        Ok(test_sources)
    }

    /// Executes all the tests in the project.
    ///
    /// This will trigger the build process first. On success all test contracts that match the
    /// configured filter will be executed
    ///
    /// Returns the test results for all matching tests.
    pub async fn execute_tests(self) -> Result<TestOutcome> {
        // Merge all configs.
        let (mut config, mut evm_opts) = self.load_config_and_evm_opts_emit_warnings()?;

        // Set number of max threads to execute tests.
        // If not specified then the number of threads determined by rayon will be used.
        if let Some(test_threads) = config.threads {
            trace!(target: "forge::test", "execute tests with {} max threads", test_threads);
            rayon::ThreadPoolBuilder::new().num_threads(test_threads).build_global()?;
        }

        // Explicitly enable isolation for gas reports for more correct gas accounting.
        if self.gas_report {
            evm_opts.isolate = true;
        } else {
            // Do not collect gas report traces if gas report is not enabled.
            config.fuzz.gas_report_samples = 0;
            config.invariant.gas_report_samples = 0;
        }

        // Set up the project.
        let mut project = config.project()?;

        // Install missing dependencies.
        if install::install_missing_dependencies(&mut config, self.build_args().silent) &&
            config.auto_detect_remappings
        {
            // need to re-configure here to also catch additional remappings
            config = self.load_config();
            project = config.project()?;
        }

        let mut filter = self.filter(&config);
        trace!(target: "forge::test", ?filter, "using filter");

        let sources_to_compile = self.get_sources_to_compile(&config, &filter)?;

        let compiler = ProjectCompiler::new()
            .quiet_if(self.json || self.opts.silent)
            .files(sources_to_compile);

        let output = compiler.compile(&project)?;

        // Create test options from general project settings and compiler output.
        let project_root = &project.paths.root;
        let toml = config.get_config_path();
        let profiles = get_available_profiles(toml)?;

        let test_options: TestOptions = TestOptionsBuilder::default()
            .fuzz(config.fuzz.clone())
            .invariant(config.invariant.clone())
            .profiles(profiles)
            .build(&output, project_root)?;

        // Determine print verbosity and executor verbosity.
        let verbosity = evm_opts.verbosity;
        if self.gas_report && evm_opts.verbosity < 3 {
            evm_opts.verbosity = 3;
        }

        let env = evm_opts.evm_env().await?;

        // Choose the internal function tracing mode, if --decode-internal is provided.
        let decode_internal = if let Some(maybe_fn) = self.decode_internal.as_ref() {
            if maybe_fn.is_some() {
                // If function filter is provided, we enable full tracing.
                InternalTraceMode::Full
            } else {
                // If no function filter is provided, we enable simple tracing.
                InternalTraceMode::Simple
            }
        } else {
            InternalTraceMode::None
        };

        // Prepare the test builder.
        let should_debug = self.debug.is_some();
        let config = Arc::new(config);
        let runner = MultiContractRunnerBuilder::new(config.clone())
            .set_debug(should_debug)
            .set_decode_internal(decode_internal)
            .initial_balance(evm_opts.initial_balance)
            .evm_spec(config.evm_spec_id())
            .sender(evm_opts.sender)
            .with_fork(evm_opts.get_fork(&config, env.clone()))
            .with_test_options(test_options)
            .enable_isolation(evm_opts.isolate)
            .alphanet(evm_opts.alphanet)
            .build(project_root, &output, env, evm_opts)?;

        let mut maybe_override_mt = |flag, maybe_regex: Option<&Regex>| {
            if let Some(regex) = maybe_regex {
                let test_pattern = &mut filter.args_mut().test_pattern;
                if test_pattern.is_some() {
                    eyre::bail!(
                        "Cannot specify both --{flag} and --match-test. \
                        Use --match-contract and --match-path to further limit the search instead."
                    );
                }
                *test_pattern = Some(regex.clone());
            }

            Ok(())
        };

        maybe_override_mt("debug", self.debug.as_ref())?;
        maybe_override_mt(
            "decode-internal",
            self.decode_internal.as_ref().and_then(|v| v.as_ref()),
        )?;

        let libraries = runner.libraries.clone();
        let outcome = self.run_tests(runner, config, verbosity, &filter, &output).await?;

        if should_debug {
            // Get first non-empty suite result. We will have only one such entry.
            let Some((_, test_result)) = outcome
                .results
                .iter()
                .find(|(_, r)| !r.test_results.is_empty())
                .map(|(_, r)| (r, r.test_results.values().next().unwrap()))
            else {
                return Err(eyre::eyre!("no tests were executed"));
            };

            let sources =
                ContractSources::from_project_output(&output, project.root(), Some(&libraries))?;

            // Run the debugger.
            let mut builder = Debugger::builder()
                .traces(
                    test_result.traces.iter().filter(|(t, _)| t.is_execution()).cloned().collect(),
                )
                .sources(sources)
                .breakpoints(test_result.breakpoints.clone());

            if let Some(decoder) = &outcome.last_run_decoder {
                builder = builder.decoder(decoder);
            }

            let mut debugger = builder.build();
            debugger.try_run()?;
        }

        Ok(outcome)
    }

    /// Run all tests that matches the filter predicate from a test runner
    pub async fn run_tests(
        &self,
        mut runner: MultiContractRunner,
        config: Arc<Config>,
        verbosity: u8,
        filter: &ProjectPathsAwareFilter,
        output: &ProjectCompileOutput,
    ) -> eyre::Result<TestOutcome> {
        if self.list {
            return list(runner, filter, self.json);
        }

        trace!(target: "forge::test", "running all tests");

        let num_filtered = runner.matching_test_functions(filter).count();
        if (self.debug.is_some() || self.decode_internal.as_ref().map_or(false, |v| v.is_some())) &&
            num_filtered != 1
        {
            eyre::bail!(
                "{num_filtered} tests matched your criteria, but exactly 1 test must match in order to run the debugger.\n\n\
                 Use --match-contract and --match-path to further limit the search.\n\
                 Filter used:\n{filter}"
            );
        }

        if self.json {
            let results = runner.test_collect(filter);
            println!("{}", serde_json::to_string(&results)?);
            return Ok(TestOutcome::new(results, self.allow_failure));
        }

        let remote_chain_id = runner.evm_opts.get_remote_chain_id().await;
        let known_contracts = runner.known_contracts.clone();

        let libraries = runner.libraries.clone();

        // Run tests.
        let (tx, rx) = channel::<(String, SuiteResult)>();
        let timer = Instant::now();
        let show_progress = config.show_progress;
        let handle = tokio::task::spawn_blocking({
            let filter = filter.clone();
            move || runner.test(&filter, tx, show_progress)
        });

        // Set up trace identifiers.
        let mut identifier = TraceIdentifiers::new().with_local(&known_contracts);

        // Avoid using etherscan for gas report as we decode more traces and this will be
        // expensive.
        if !self.gas_report {
            identifier = identifier.with_etherscan(&config, remote_chain_id)?;
        }

        // Build the trace decoder.
        let mut builder = CallTraceDecoderBuilder::new()
            .with_known_contracts(&known_contracts)
            .with_verbosity(verbosity);
        // Signatures are of no value for gas reports.
        if !self.gas_report {
            builder = builder.with_signature_identifier(SignaturesIdentifier::new(
                Config::foundry_cache_dir(),
                config.offline,
            )?);
        }

        if self.decode_internal.is_some() {
            let sources =
                ContractSources::from_project_output(output, &config.root, Some(&libraries))?;
            builder = builder.with_debug_identifier(DebugTraceIdentifier::new(sources));
        }
        let mut decoder = builder.build();

        let mut gas_report = self
            .gas_report
            .then(|| GasReport::new(config.gas_reports.clone(), config.gas_reports_ignore.clone()));

        let mut outcome = TestOutcome::empty(self.allow_failure);

        let mut any_test_failed = false;
        for (contract_name, suite_result) in rx {
            let tests = &suite_result.test_results;

            // Clear the addresses and labels from previous test.
            decoder.clear_addresses();

            // We identify addresses if we're going to print *any* trace or gas report.
            let identify_addresses = verbosity >= 3 || self.gas_report || self.debug.is_some();

            // Print suite header.
            println!();
            for warning in suite_result.warnings.iter() {
                eprintln!("{} {warning}", "Warning:".yellow().bold());
            }
            if !tests.is_empty() {
                let len = tests.len();
                let tests = if len > 1 { "tests" } else { "test" };
                println!("Ran {len} {tests} for {contract_name}");
            }

            // Process individual test results, printing logs and traces when necessary.
            for (name, result) in tests {
                shell::println(result.short_result(name))?;

                // We only display logs at level 2 and above
                if verbosity >= 2 {
                    // We only decode logs from Hardhat and DS-style console events
                    let console_logs = decode_console_logs(&result.logs);
                    if !console_logs.is_empty() {
                        println!("Logs:");
                        for log in console_logs {
                            println!("  {log}");
                        }
                        println!();
                    }
                }

                // We shouldn't break out of the outer loop directly here so that we finish
                // processing the remaining tests and print the suite summary.
                any_test_failed |= result.status == TestStatus::Failure;

                // Clear the addresses and labels from previous runs.
                decoder.clear_addresses();
                decoder
                    .labels
                    .extend(result.labeled_addresses.iter().map(|(k, v)| (*k, v.clone())));

                // Identify addresses and decode traces.
                let mut decoded_traces = Vec::with_capacity(result.traces.len());
                for (kind, arena) in &mut result.traces.clone() {
                    if identify_addresses {
                        decoder.identify(arena, &mut identifier);
                    }

                    // verbosity:
                    // - 0..3: nothing
                    // - 3: only display traces for failed tests
                    // - 4: also display the setup trace for failed tests
                    // - 5..: display all traces for all tests
                    let should_include = match kind {
                        TraceKind::Execution => {
                            (verbosity == 3 && result.status.is_failure()) || verbosity >= 4
                        }
                        TraceKind::Setup => {
                            (verbosity == 4 && result.status.is_failure()) || verbosity >= 5
                        }
                        TraceKind::Deployment => false,
                    };

                    if should_include {
                        decode_trace_arena(arena, &decoder).await?;
                        decoded_traces.push(render_trace_arena(arena));
                    }
                }

                if !decoded_traces.is_empty() {
                    shell::println("Traces:")?;
                    for trace in &decoded_traces {
                        shell::println(trace)?;
                    }
                }

                if let Some(gas_report) = &mut gas_report {
                    gas_report
                        .analyze(result.traces.iter().map(|(_, arena)| arena), &decoder)
                        .await;

                    for trace in result.gas_report_traces.iter() {
                        decoder.clear_addresses();

                        // Re-execute setup and deployment traces to collect identities created in
                        // setUp and constructor.
                        for (kind, arena) in &result.traces {
                            if !matches!(kind, TraceKind::Execution) {
                                decoder.identify(arena, &mut identifier);
                            }
                        }

                        for arena in trace {
                            decoder.identify(arena, &mut identifier);
                            gas_report.analyze([arena], &decoder).await;
                        }
                    }
                }
            }

            // Print suite summary.
            shell::println(suite_result.summary())?;

            // Add the suite result to the outcome.
            outcome.results.insert(contract_name, suite_result);

            // Stop processing the remaining suites if any test failed and `fail_fast` is set.
            if self.fail_fast && any_test_failed {
                break;
            }
        }
        outcome.last_run_decoder = Some(decoder);
        let duration = timer.elapsed();

        trace!(target: "forge::test", len=outcome.results.len(), %any_test_failed, "done with results");

        if let Some(gas_report) = gas_report {
            let finalized = gas_report.finalize();
            shell::println(&finalized)?;
            outcome.gas_report = Some(finalized);
        }

        if !outcome.results.is_empty() {
            shell::println(outcome.summary(duration))?;

            if self.summary {
                let mut summary_table = TestSummaryReporter::new(self.detailed);
                shell::println("\n\nTest Summary:")?;
                summary_table.print_summary(&outcome);
            }
        }

        // Reattach the task.
        if let Err(e) = handle.await {
            match e.try_into_panic() {
                Ok(payload) => std::panic::resume_unwind(payload),
                Err(e) => return Err(e.into()),
            }
        }

        // Persist test run failures to enable replaying.
        persist_run_failures(&config, &outcome);

        Ok(outcome)
    }

    /// Returns the flattened [`FilterArgs`] arguments merged with [`Config`].
    /// Loads and applies filter from file if only last test run failures performed.
    pub fn filter(&self, config: &Config) -> ProjectPathsAwareFilter {
        let mut filter = self.filter.clone();
        if self.rerun {
            filter.test_pattern = last_run_failures(config);
        }
        filter.merge_with_config(config)
    }

    /// Returns whether `BuildArgs` was configured with `--watch`
    pub fn is_watch(&self) -> bool {
        self.watch.watch.is_some()
    }

    /// Returns the [`watchexec::InitConfig`] and [`watchexec::RuntimeConfig`] necessary to
    /// bootstrap a new [`watchexe::Watchexec`] loop.
    pub(crate) fn watchexec_config(&self) -> Result<watchexec::Config> {
        self.watch.watchexec_config(|| {
            let config = Config::from(self);
            [config.src, config.test]
        })
    }
}

impl Provider for TestArgs {
    fn metadata(&self) -> Metadata {
        Metadata::named("Core Build Args Provider")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        let mut dict = Dict::default();

        let mut fuzz_dict = Dict::default();
        if let Some(fuzz_seed) = self.fuzz_seed {
            fuzz_dict.insert("seed".to_string(), fuzz_seed.to_string().into());
        }
        if let Some(fuzz_runs) = self.fuzz_runs {
            fuzz_dict.insert("runs".to_string(), fuzz_runs.into());
        }
        if let Some(fuzz_input_file) = self.fuzz_input_file.clone() {
            fuzz_dict.insert("failure_persist_file".to_string(), fuzz_input_file.into());
        }
        dict.insert("fuzz".to_string(), fuzz_dict.into());

        if let Some(etherscan_api_key) =
            self.etherscan_api_key.as_ref().filter(|s| !s.trim().is_empty())
        {
            dict.insert("etherscan_api_key".to_string(), etherscan_api_key.to_string().into());
        }

        if self.show_progress {
            dict.insert("show_progress".to_string(), true.into());
        }

        if let Some(threads) = self.threads {
            dict.insert("threads".to_string(), threads.into());
        }

        Ok(Map::from([(Config::selected_profile(), dict)]))
    }
}

/// Lists all matching tests
fn list(
    runner: MultiContractRunner,
    filter: &ProjectPathsAwareFilter,
    json: bool,
) -> Result<TestOutcome> {
    let results = runner.list(filter);

    if json {
        println!("{}", serde_json::to_string(&results)?);
    } else {
        for (file, contracts) in results.iter() {
            println!("{file}");
            for (contract, tests) in contracts.iter() {
                println!("  {contract}");
                println!("    {}\n", tests.join("\n    "));
            }
        }
    }
    Ok(TestOutcome::empty(false))
}

/// Load persisted filter (with last test run failures) from file.
fn last_run_failures(config: &Config) -> Option<regex::Regex> {
    match fs::read_to_string(&config.test_failures_file) {
        Ok(filter) => Some(Regex::new(&filter).unwrap()),
        Err(_) => None,
    }
}

/// Persist filter with last test run failures (only if there's any failure).
fn persist_run_failures(config: &Config, outcome: &TestOutcome) {
    if outcome.failed() > 0 && fs::create_file(&config.test_failures_file).is_ok() {
        let mut filter = String::new();
        let mut failures = outcome.failures().peekable();
        while let Some((test_name, _)) = failures.next() {
            if let Some(test_match) = test_name.split("(").next() {
                filter.push_str(test_match);
                if failures.peek().is_some() {
                    filter.push('|');
                }
            }
        }
        let _ = fs::write(&config.test_failures_file, filter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundry_config::{Chain, InvariantConfig};
    use foundry_test_utils::forgetest_async;

    #[test]
    fn watch_parse() {
        let args: TestArgs = TestArgs::parse_from(["foundry-cli", "-vw"]);
        assert!(args.watch.watch.is_some());
    }

    #[test]
    fn fuzz_seed() {
        let args: TestArgs = TestArgs::parse_from(["foundry-cli", "--fuzz-seed", "0x10"]);
        assert!(args.fuzz_seed.is_some());
    }

    // <https://github.com/foundry-rs/foundry/issues/5913>
    #[test]
    fn fuzz_seed_exists() {
        let args: TestArgs =
            TestArgs::parse_from(["foundry-cli", "-vvv", "--gas-report", "--fuzz-seed", "0x10"]);
        assert!(args.fuzz_seed.is_some());
    }

    #[test]
    fn extract_chain() {
        let test = |arg: &str, expected: Chain| {
            let args = TestArgs::parse_from(["foundry-cli", arg]);
            assert_eq!(args.evm_opts.env.chain, Some(expected));
            let (config, evm_opts) = args.load_config_and_evm_opts().unwrap();
            assert_eq!(config.chain, Some(expected));
            assert_eq!(evm_opts.env.chain_id, Some(expected.id()));
        };
        test("--chain-id=1", Chain::mainnet());
        test("--chain-id=42", Chain::from_id(42));
    }

    forgetest_async!(gas_report_fuzz_invariant, |prj, _cmd| {
        // speed up test by running with depth of 15
        let config = Config {
            invariant: { InvariantConfig { depth: 15, ..Default::default() } },
            ..Default::default()
        };
        prj.write_config(config);

        prj.insert_ds_test();
        prj.add_source(
            "Contracts.sol",
            r#"
//SPDX-license-identifier: MIT

import "./test.sol";

contract Foo {
    function foo() public {}
}

contract Bar {
    function bar() public {}
}


contract FooBarTest is DSTest {
    Foo public targetContract;

    function setUp() public {
        targetContract = new Foo();
    }

    function invariant_dummy() public {
        assertTrue(true);
    }

    function testFuzz_bar(uint256 _val) public {
        (new Bar()).bar();
    }
}
        "#,
        )
        .unwrap();

        let args = TestArgs::parse_from([
            "foundry-cli",
            "--gas-report",
            "--root",
            &prj.root().to_string_lossy(),
            "--silent",
        ]);

        let outcome = args.run().await.unwrap();
        let gas_report = outcome.gas_report.unwrap();

        assert_eq!(gas_report.contracts.len(), 3);
        let call_cnts = gas_report
            .contracts
            .values()
            .flat_map(|c| c.functions.values().flat_map(|f| f.values().map(|v| v.calls.len())))
            .collect::<Vec<_>>();
        // assert that all functions were called at least 100 times
        assert!(call_cnts.iter().all(|c| *c > 100));
    });
}
