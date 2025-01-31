use super::{install, test::filter::ProjectPathsAwareFilter, watch::WatchArgs};
use alloy_primitives::U256;
use chrono::Utc;
use clap::{Parser, ValueHint};
use eyre::{Context, OptionExt, Result};
use forge::{
    decode::decode_console_logs,
    gas_report::GasReport,
    multi_runner::matches_contract,
    result::{SuiteResult, TestOutcome, TestStatus},
    traces::{
        debug::{ContractSources, DebugTraceIdentifier},
        decode_trace_arena, folded_stack_trace,
        identifier::SignaturesIdentifier,
        CallTraceDecoderBuilder, InternalTraceMode, TraceKind,
    },
    MultiContractRunner, MultiContractRunnerBuilder, TestFilter,
};
use foundry_cli::{
    opts::{BuildOpts, GlobalArgs},
    utils::{self, LoadConfig},
};
use foundry_common::{compile::ProjectCompiler, evm::EvmArgs, fs, shell, TestFunctionExt};
use foundry_compilers::{
    artifacts::output_selection::OutputSelection,
    compilers::{
        multi::{MultiCompiler, MultiCompilerLanguage},
        Language,
    },
    utils::source_files_iter,
    ProjectCompileOutput,
};
use foundry_config::{
    figment,
    figment::{
        value::{Dict, Map},
        Metadata, Profile, Provider,
    },
    filter::GlobMatcher,
    Config,
};
use foundry_debugger::Debugger;
use foundry_evm::traces::identifier::TraceIdentifiers;
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    path::PathBuf,
    sync::{mpsc::channel, Arc},
    time::{Duration, Instant},
};
use yansi::Paint;

mod filter;
mod summary;
pub use filter::FilterArgs;
use forge::{result::TestKind, traces::render_trace_arena_inner};
use quick_junit::{NonSuccessKind, Report, TestCase, TestCaseStatus, TestSuite};
use summary::{format_invariant_metrics_table, TestSummaryReport};

// Loads project's figment and merges the build cli arguments into it
foundry_config::merge_impl_figment_convert!(TestArgs, build, evm);

/// CLI arguments for `forge test`.
#[derive(Clone, Debug, Parser)]
#[command(next_help_heading = "Test options")]
pub struct TestArgs {
    // Include global options for users of this struct.
    #[command(flatten)]
    pub global: GlobalArgs,

    /// The contract file you want to test, it's a shortcut for --match-path.
    #[arg(value_hint = ValueHint::FilePath)]
    pub path: Option<GlobMatcher>,

    /// Run a single test in the debugger.
    ///
    /// The matching test will be opened in the debugger regardless of the outcome of the test.
    ///
    /// If the matching test is a fuzz test, then it will open the debugger on the first failure
    /// case. If the fuzz test does not fail, it will open the debugger on the last fuzz case.
    #[arg(long, conflicts_with_all = ["flamegraph", "flamechart", "decode_internal", "rerun"])]
    debug: bool,

    /// Generate a flamegraph for a single test. Implies `--decode-internal`.
    ///
    /// A flame graph is used to visualize which functions or operations within the smart contract
    /// are consuming the most gas overall in a sorted manner.
    #[arg(long)]
    flamegraph: bool,

    /// Generate a flamechart for a single test. Implies `--decode-internal`.
    ///
    /// A flame chart shows the gas usage over time, illustrating when each function is
    /// called (execution order) and how much gas it consumes at each point in the timeline.
    #[arg(long, conflicts_with = "flamegraph")]
    flamechart: bool,

    /// Identify internal functions in traces.
    ///
    /// This will trace internal functions and decode stack parameters.
    ///
    /// Parameters stored in memory (such as bytes or arrays) are currently decoded only when a
    /// single function is matched, similarly to `--debug`, for performance reasons.
    #[arg(long)]
    decode_internal: bool,

    /// Dumps all debugger steps to file.
    #[arg(
        long,
        requires = "debug",
        value_hint = ValueHint::FilePath,
        value_name = "PATH"
    )]
    dump: Option<PathBuf>,

    /// Print a gas report.
    #[arg(long, env = "FORGE_GAS_REPORT")]
    gas_report: bool,

    /// Check gas snapshots against previous runs.
    #[arg(long, env = "FORGE_SNAPSHOT_CHECK")]
    gas_snapshot_check: Option<bool>,

    /// Exit with code 0 even if a test fails.
    #[arg(long, env = "FORGE_ALLOW_FAILURE")]
    allow_failure: bool,

    /// Output test results as JUnit XML report.
    #[arg(long, conflicts_with_all = ["quiet", "json", "gas_report", "summary", "list", "show_progress"], help_heading = "Display options")]
    pub junit: bool,

    /// Stop running tests after the first failure.
    #[arg(long)]
    pub fail_fast: bool,

    /// The Etherscan (or equivalent) API key.
    #[arg(long, env = "ETHERSCAN_API_KEY", value_name = "KEY")]
    etherscan_api_key: Option<String>,

    /// List tests instead of running them.
    #[arg(long, short, conflicts_with_all = ["show_progress", "decode_internal", "summary"], help_heading = "Display options")]
    list: bool,

    /// Set seed used to generate randomness during your fuzz runs.
    #[arg(long)]
    pub fuzz_seed: Option<U256>,

    #[arg(long, env = "FOUNDRY_FUZZ_RUNS", value_name = "RUNS")]
    pub fuzz_runs: Option<u64>,

    /// Timeout for each fuzz run in seconds.
    #[arg(long, env = "FOUNDRY_FUZZ_TIMEOUT", value_name = "TIMEOUT")]
    pub fuzz_timeout: Option<u64>,

    /// File to rerun fuzz failures from.
    #[arg(long)]
    pub fuzz_input_file: Option<String>,

    /// Show test execution progress.
    #[arg(long, conflicts_with_all = ["quiet", "json"], help_heading = "Display options")]
    pub show_progress: bool,

    /// Re-run recorded test failures from last run.
    /// If no failure recorded then regular test run is performed.
    #[arg(long)]
    pub rerun: bool,

    /// Print test summary table.
    #[arg(long, help_heading = "Display options")]
    pub summary: bool,

    /// Print detailed test summary table.
    #[arg(long, help_heading = "Display options", requires = "summary")]
    pub detailed: bool,

    #[command(flatten)]
    filter: FilterArgs,

    #[command(flatten)]
    evm: EvmArgs,

    #[command(flatten)]
    pub build: BuildOpts,

    #[command(flatten)]
    pub watch: WatchArgs,
}

impl TestArgs {
    pub async fn run(self) -> Result<TestOutcome> {
        trace!(target: "forge::test", "executing test command");
        self.execute_tests().await
    }

    /// Returns sources which include any tests to be executed.
    /// If no filters are provided, sources are filtered by existence of test/invariant methods in
    /// them, If filters are provided, sources are additionally filtered by them.
    pub fn get_sources_to_compile(
        &self,
        config: &Config,
        filter: &ProjectPathsAwareFilter,
    ) -> Result<BTreeSet<PathBuf>> {
        let mut project = config.create_project(true, true)?;
        project.update_output_selection(|selection| {
            *selection = OutputSelection::common_output_selection(["abi".to_string()]);
        });

        let output = project.compile()?;

        if output.has_compiler_errors() {
            sh_println!("{output}")?;
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
                sh_println!(
                    "No tests found in project! \
                        Forge looks for functions that starts with `test`."
                )?;
            } else {
                sh_println!("No tests match the provided pattern:")?;
                sh_print!("{filter}")?;

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
                        sh_println!("\nDid you mean `{suggestion}`?")?;
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
    pub async fn execute_tests(mut self) -> Result<TestOutcome> {
        // Merge all configs.
        let (mut config, mut evm_opts) = self.load_config_and_evm_opts()?;

        // Explicitly enable isolation for gas reports for more correct gas accounting.
        if self.gas_report {
            evm_opts.isolate = true;
        } else {
            // Do not collect gas report traces if gas report is not enabled.
            config.fuzz.gas_report_samples = 0;
            config.invariant.gas_report_samples = 0;
        }

        // Install missing dependencies.
        if install::install_missing_dependencies(&mut config) && config.auto_detect_remappings {
            // need to re-configure here to also catch additional remappings
            config = self.load_config()?;
        }

        // Set up the project.
        let project = config.project()?;

        let filter = self.filter(&config);
        trace!(target: "forge::test", ?filter, "using filter");

        let sources_to_compile = self.get_sources_to_compile(&config, &filter)?;

        let compiler =
            ProjectCompiler::new().quiet(shell::is_json() || self.junit).files(sources_to_compile);

        let output = compiler.compile(&project)?;

        // Create test options from general project settings and compiler output.
        let project_root = &project.paths.root;

        let should_debug = self.debug;
        let should_draw = self.flamegraph || self.flamechart;

        // Determine print verbosity and executor verbosity.
        let verbosity = evm_opts.verbosity;
        if (self.gas_report && evm_opts.verbosity < 3) || self.flamegraph || self.flamechart {
            evm_opts.verbosity = 3;
        }

        let env = evm_opts.evm_env().await?;

        // Enable internal tracing for more informative flamegraph.
        if should_draw && !self.decode_internal {
            self.decode_internal = true;
        }

        // Choose the internal function tracing mode, if --decode-internal is provided.
        let decode_internal = if self.decode_internal {
            // If more than one function matched, we enable simple tracing.
            // If only one function matched, we enable full tracing. This is done in `run_tests`.
            InternalTraceMode::Simple
        } else {
            InternalTraceMode::None
        };

        // Prepare the test builder.
        let config = Arc::new(config);
        let runner = MultiContractRunnerBuilder::new(config.clone())
            .set_debug(should_debug)
            .set_decode_internal(decode_internal)
            .initial_balance(evm_opts.initial_balance)
            .evm_spec(config.evm_spec_id())
            .sender(evm_opts.sender)
            .with_fork(evm_opts.get_fork(&config, env.clone()))
            .enable_isolation(evm_opts.isolate)
            .odyssey(evm_opts.odyssey)
            .build::<MultiCompiler>(project_root, &output, env, evm_opts)?;

        let libraries = runner.libraries.clone();
        let mut outcome = self.run_tests(runner, config, verbosity, &filter, &output).await?;

        if should_draw {
            let (suite_name, test_name, mut test_result) =
                outcome.remove_first().ok_or_eyre("no tests were executed")?;

            let (_, arena) = test_result
                .traces
                .iter_mut()
                .find(|(kind, _)| *kind == TraceKind::Execution)
                .unwrap();

            // Decode traces.
            let decoder = outcome.last_run_decoder.as_ref().unwrap();
            decode_trace_arena(arena, decoder).await?;
            let mut fst = folded_stack_trace::build(arena);

            let label = if self.flamegraph { "flamegraph" } else { "flamechart" };
            let contract = suite_name.split(':').next_back().unwrap();
            let test_name = test_name.trim_end_matches("()");
            let file_name = format!("cache/{label}_{contract}_{test_name}.svg");
            let file = std::fs::File::create(&file_name).wrap_err("failed to create file")?;
            let file = std::io::BufWriter::new(file);

            let mut options = inferno::flamegraph::Options::default();
            options.title = format!("{label} {contract}::{test_name}");
            options.count_name = "gas".to_string();
            if self.flamechart {
                options.flame_chart = true;
                fst.reverse();
            }

            // Generate SVG.
            inferno::flamegraph::from_lines(&mut options, fst.iter().map(String::as_str), file)
                .wrap_err("failed to write svg")?;
            sh_println!("Saved to {file_name}")?;

            // Open SVG in default program.
            if let Err(e) = opener::open(&file_name) {
                sh_err!("Failed to open {file_name}; please open it manually: {e}")?;
            }
        }

        if should_debug {
            // Get first non-empty suite result. We will have only one such entry.
            let (_, _, test_result) =
                outcome.remove_first().ok_or_eyre("no tests were executed")?;

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
            if let Some(dump_path) = self.dump {
                debugger.dump_to_file(&dump_path)?;
            } else {
                debugger.try_run_tui()?;
            }
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
            return list(runner, filter);
        }

        trace!(target: "forge::test", "running all tests");

        // If we need to render to a serialized format, we should not print anything else to stdout.
        let silent = self.gas_report && shell::is_json() || self.summary && shell::is_json();

        let num_filtered = runner.matching_test_functions(filter).count();
        if num_filtered != 1 && (self.debug || self.flamegraph || self.flamechart) {
            let action = if self.flamegraph {
                "generate a flamegraph"
            } else if self.flamechart {
                "generate a flamechart"
            } else {
                "run the debugger"
            };
            let filter = if filter.is_empty() {
                String::new()
            } else {
                format!("\n\nFilter used:\n{filter}")
            };
            eyre::bail!(
                "{num_filtered} tests matched your criteria, but exactly 1 test must match in order to {action}.\n\n\
                 Use --match-contract and --match-path to further limit the search.{filter}",
            );
        }

        // If exactly one test matched, we enable full tracing.
        if num_filtered == 1 && self.decode_internal {
            runner.decode_internal = InternalTraceMode::Full;
        }

        // Run tests in a non-streaming fashion and collect results for serialization.
        if !self.gas_report && !self.summary && shell::is_json() {
            let mut results = runner.test_collect(filter);
            results.values_mut().for_each(|suite_result| {
                for test_result in suite_result.test_results.values_mut() {
                    if verbosity >= 2 {
                        // Decode logs at level 2 and above.
                        test_result.decoded_logs = decode_console_logs(&test_result.logs);
                    } else {
                        // Empty logs for non verbose runs.
                        test_result.logs = vec![];
                    }
                }
            });
            sh_println!("{}", serde_json::to_string(&results)?)?;
            return Ok(TestOutcome::new(results, self.allow_failure));
        }

        if self.junit {
            let results = runner.test_collect(filter);
            sh_println!("{}", junit_xml_report(&results, verbosity).to_string()?)?;
            return Ok(TestOutcome::new(results, self.allow_failure));
        }

        let remote_chain_id = runner.evm_opts.get_remote_chain_id().await;
        let known_contracts = runner.known_contracts.clone();

        let libraries = runner.libraries.clone();

        // Run tests in a streaming fashion.
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

        if self.decode_internal {
            let sources =
                ContractSources::from_project_output(output, &config.root, Some(&libraries))?;
            builder = builder.with_debug_identifier(DebugTraceIdentifier::new(sources));
        }
        let mut decoder = builder.build();

        let mut gas_report = self.gas_report.then(|| {
            GasReport::new(
                config.gas_reports.clone(),
                config.gas_reports_ignore.clone(),
                config.gas_reports_include_tests,
            )
        });

        let mut gas_snapshots = BTreeMap::<String, BTreeMap<String, String>>::new();

        let mut outcome = TestOutcome::empty(self.allow_failure);

        let mut any_test_failed = false;
        for (contract_name, suite_result) in rx {
            let tests = &suite_result.test_results;

            // Clear the addresses and labels from previous test.
            decoder.clear_addresses();

            // We identify addresses if we're going to print *any* trace or gas report.
            let identify_addresses = verbosity >= 3 ||
                self.gas_report ||
                self.debug ||
                self.flamegraph ||
                self.flamechart;

            // Print suite header.
            if !silent {
                sh_println!()?;
                for warning in suite_result.warnings.iter() {
                    sh_warn!("{warning}")?;
                }
                if !tests.is_empty() {
                    let len = tests.len();
                    let tests = if len > 1 { "tests" } else { "test" };
                    sh_println!("Ran {len} {tests} for {contract_name}")?;
                }
            }

            // Process individual test results, printing logs and traces when necessary.
            for (name, result) in tests {
                if !silent {
                    sh_println!("{}", result.short_result(name))?;

                    // Display invariant metrics if invariant kind.
                    if let TestKind::Invariant { metrics, .. } = &result.kind {
                        if !metrics.is_empty() {
                            let _ = sh_println!("\n{}\n", format_invariant_metrics_table(metrics));
                        }
                    }

                    // We only display logs at level 2 and above
                    if verbosity >= 2 {
                        // We only decode logs from Hardhat and DS-style console events
                        let console_logs = decode_console_logs(&result.logs);
                        if !console_logs.is_empty() {
                            sh_println!("Logs:")?;
                            for log in console_logs {
                                sh_println!("  {log}")?;
                            }
                            sh_println!()?;
                        }
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
                    // - 5..: display all traces for all tests, including storage changes
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
                        decoded_traces.push(render_trace_arena_inner(arena, false, verbosity > 4));
                    }
                }

                if !silent && !decoded_traces.is_empty() {
                    sh_println!("Traces:")?;
                    for trace in &decoded_traces {
                        sh_println!("{trace}")?;
                    }
                }

                if let Some(gas_report) = &mut gas_report {
                    gas_report.analyze(result.traces.iter().map(|(_, a)| &a.arena), &decoder).await;

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

                // Collect and merge gas snapshots.
                for (group, new_snapshots) in result.gas_snapshots.iter() {
                    gas_snapshots.entry(group.clone()).or_default().extend(new_snapshots.clone());
                }
            }

            // Write gas snapshots to disk if any were collected.
            if !gas_snapshots.is_empty() {
                // By default `gas_snapshot_check` is set to `false` in the config.
                //
                // The user can either:
                // - Set `FORGE_SNAPSHOT_CHECK=true` in the environment.
                // - Pass `--gas-snapshot-check=true` as a CLI argument.
                // - Set `gas_snapshot_check = true` in the config.
                //
                // If the user passes `--gas-snapshot-check=<bool>` then it will override the config
                // and the environment variable, disabling the check if `false` is passed.
                //
                // Exiting early with code 1 if differences are found.
                if self.gas_snapshot_check.unwrap_or(config.gas_snapshot_check) {
                    let differences_found = gas_snapshots.clone().into_iter().fold(
                        false,
                        |mut found, (group, snapshots)| {
                            // If the snapshot file doesn't exist, we can't compare so we skip.
                            if !&config.snapshots.join(format!("{group}.json")).exists() {
                                return false;
                            }

                            let previous_snapshots: BTreeMap<String, String> =
                                fs::read_json_file(&config.snapshots.join(format!("{group}.json")))
                                    .expect("Failed to read snapshots from disk");

                            let diff: BTreeMap<_, _> = snapshots
                                .iter()
                                .filter_map(|(k, v)| {
                                    previous_snapshots.get(k).and_then(|previous_snapshot| {
                                        if previous_snapshot != v {
                                            Some((
                                                k.clone(),
                                                (previous_snapshot.clone(), v.clone()),
                                            ))
                                        } else {
                                            None
                                        }
                                    })
                                })
                                .collect();

                            if !diff.is_empty() {
                                let _ = sh_eprintln!(
                                    "{}",
                                    format!("\n[{group}] Failed to match snapshots:").red().bold()
                                );

                                for (key, (previous_snapshot, snapshot)) in &diff {
                                    let _ = sh_eprintln!(
                                        "{}",
                                        format!("- [{key}] {previous_snapshot} → {snapshot}").red()
                                    );
                                }

                                found = true;
                            }

                            found
                        },
                    );

                    if differences_found {
                        sh_eprintln!()?;
                        eyre::bail!("Snapshots differ from previous run");
                    }
                }

                // Create `snapshots` directory if it doesn't exist.
                fs::create_dir_all(&config.snapshots)?;

                // Write gas snapshots to disk per group.
                gas_snapshots.clone().into_iter().for_each(|(group, snapshots)| {
                    fs::read_json_file::<BTreeMap<String, String>>(
                        &config.snapshots.join(format!("{group}.json")),
                    )
                    .map(|previous_snapshots| {
                        let mut merged_snapshots = BTreeMap::new();

                        for (k, v) in &previous_snapshots {
                            merged_snapshots.insert(k.clone(), v.clone());
                        }

                        for (k, v) in &snapshots {
                            merged_snapshots.insert(k.clone(), v.clone());
                        }

                        fs::write_pretty_json_file(
                            &config.snapshots.join(format!("{group}.json")),
                            &merged_snapshots,
                        )
                        .expect("Failed to write gas snapshots to disk");
                    })
                    .unwrap_or_else(|_| {
                        fs::write_pretty_json_file(
                            &config.snapshots.join(format!("{group}.json")),
                            &snapshots,
                        )
                        .expect("Failed to write gas snapshots to disk");
                    });
                });
            }

            // Print suite summary.
            if !silent {
                sh_println!("{}", suite_result.summary())?;
            }

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
            sh_println!("{}", &finalized)?;
            outcome.gas_report = Some(finalized);
        }

        if !self.summary && !shell::is_json() {
            sh_println!("{}", outcome.summary(duration))?;
        }

        if self.summary && !outcome.results.is_empty() {
            let summary_report = TestSummaryReport::new(self.detailed, outcome.clone());
            sh_println!("{}", &summary_report)?;
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
        if filter.path_pattern.is_some() {
            if self.path.is_some() {
                panic!("Can not supply both --match-path and |path|");
            }
        } else {
            filter.path_pattern = self.path.clone();
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
            let config = self.load_config()?;
            Ok([config.src, config.test])
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
        if let Some(fuzz_timeout) = self.fuzz_timeout {
            fuzz_dict.insert("timeout".to_string(), fuzz_timeout.into());
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

        Ok(Map::from([(Config::selected_profile(), dict)]))
    }
}

/// Lists all matching tests
fn list(runner: MultiContractRunner, filter: &ProjectPathsAwareFilter) -> Result<TestOutcome> {
    let results = runner.list(filter);

    if shell::is_json() {
        sh_println!("{}", serde_json::to_string(&results)?)?;
    } else {
        for (file, contracts) in results.iter() {
            sh_println!("{file}")?;
            for (contract, tests) in contracts.iter() {
                sh_println!("  {contract}")?;
                sh_println!("    {}\n", tests.join("\n    "))?;
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
            if test_name.is_any_test() {
                if let Some(test_match) = test_name.split("(").next() {
                    filter.push_str(test_match);
                    if failures.peek().is_some() {
                        filter.push('|');
                    }
                }
            }
        }
        let _ = fs::write(&config.test_failures_file, filter);
    }
}

/// Generate test report in JUnit XML report format.
fn junit_xml_report(results: &BTreeMap<String, SuiteResult>, verbosity: u8) -> Report {
    let mut total_duration = Duration::default();
    let mut junit_report = Report::new("Test run");
    junit_report.set_timestamp(Utc::now());
    for (suite_name, suite_result) in results {
        let mut test_suite = TestSuite::new(suite_name);
        total_duration += suite_result.duration;
        test_suite.set_time(suite_result.duration);
        test_suite.set_system_out(suite_result.summary());
        for (test_name, test_result) in &suite_result.test_results {
            let mut test_status = match test_result.status {
                TestStatus::Success => TestCaseStatus::success(),
                TestStatus::Failure => TestCaseStatus::non_success(NonSuccessKind::Failure),
                TestStatus::Skipped => TestCaseStatus::skipped(),
            };
            if let Some(reason) = &test_result.reason {
                test_status.set_message(reason);
            }

            let mut test_case = TestCase::new(test_name, test_status);
            test_case.set_time(test_result.duration);

            let mut sys_out = String::new();
            let result_report = test_result.kind.report();
            write!(sys_out, "{test_result} {test_name} {result_report}").unwrap();
            if verbosity >= 2 && !test_result.logs.is_empty() {
                write!(sys_out, "\\nLogs:\\n").unwrap();
                let console_logs = decode_console_logs(&test_result.logs);
                for log in console_logs {
                    write!(sys_out, "  {log}\\n").unwrap();
                }
            }

            test_case.set_system_out(sys_out);
            test_suite.add_test_case(test_case);
        }
        junit_report.add_test_suite(test_suite);
    }
    junit_report.set_time(total_duration);
    junit_report
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
            assert_eq!(args.evm.env.chain, Some(expected));
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
        ]);
        let outcome = args.run().await.unwrap();
        let gas_report = outcome.gas_report.as_ref().unwrap();

        assert_eq!(gas_report.contracts.len(), 3, "{}", outcome.summary(Default::default()));
        let call_cnts = gas_report
            .contracts
            .values()
            .flat_map(|c| c.functions.values().flat_map(|f| f.values().map(|v| v.frames.len())))
            .collect::<Vec<_>>();
        // assert that all functions were called at least 100 times
        assert!(call_cnts.iter().all(|c| *c > 100));
    });
}
