use super::{
    install,
    test::{ProjectPathsAwareFilter, TestOutcome},
};
use alloy_primitives::U256;
use crate::cmd::mutate::summary::{
    MutantTestResult, MutationTestOutcome, MutationTestSuiteResult, MutationTestSummaryReporter,
};
use clap::Parser;
use eyre::{eyre, Result};
use forge::{
    inspectors::CheatsConfig, result::SuiteResult, revm::primitives::Env,
    MultiContractRunnerBuilder, TestOptions, TestOptionsBuilder,
};
use foundry_cli::{
    opts::CoreBuildArgs,
    utils::{self, LoadConfig},
};
use foundry_common::{
    compile::ProjectCompiler,
    evm::EvmArgs,
    shell::{self},
    term::{MutatorSpinnerReporter, ProgressReporter},
};
use foundry_compilers::{
    project_util::{copy_dir, TempProject},
    Project, ProjectCompileOutput,
};
use foundry_config::{
    figment,
    figment::{
        value::{Dict, Map},
        Metadata, Profile, Provider,
    },
    get_available_profiles, Config,
};
use foundry_evm::{backend::Backend, opts::EvmOpts};
use foundry_evm_mutator::{Mutant, MutatorConfigBuilder};
use futures::future::{join_all, try_join_all};
use itertools::Itertools;
use std::{
    collections::{BTreeMap, HashMap as StdHashMap},
    fs,
    path::PathBuf,
    sync::mpsc::channel,
    time::{Duration, Instant},
};

mod filter;
pub use filter::*;
mod summary;
pub use summary::*;

// Loads project's figment and merges the build cli arguments into it
foundry_config::merge_impl_figment_convert!(MutateTestArgs, opts, evm_opts);

/// CLI arguments for `forge mutate`.
#[derive(Debug, Clone, Parser)]
#[clap(next_help_heading = "Mutation Test options")]
pub struct MutateTestArgs {
    /// Output mutate results in JSON format.
    #[clap(long, short, help_heading = "Display options")]
    json: bool,

    #[clap(flatten)]
    filter: MutateFilterArgs,

    /// Exit with code 0 even if a test fails.
    #[clap(long, env = "FORGE_ALLOW_FAILURE")]
    allow_failure: bool,

    /// Stop running mutation tests after the first surviving mutation
    #[clap(long)]
    pub fail_fast: bool,

    /// List all matching functions instead of running them
    #[clap(long, short, help_heading = "Display options")]
    list: bool,

    /// Set seed used to generate randomness during your fuzz runs.
    #[clap(long)]
    pub fuzz_seed: Option<U256>,

    #[clap(long, env = "FOUNDRY_FUZZ_RUNS", value_name = "RUNS")]
    pub fuzz_runs: Option<u64>,

    #[clap(flatten)]
    evm_opts: EvmArgs,

    #[clap(flatten)]
    opts: CoreBuildArgs,

    /// Export generated mutants to a directory
    #[clap(long, default_value_t = false)]
    pub export: bool,

    /// Print mutation test summary table
    #[clap(long, help_heading = "Display options", default_value_t = false)]
    pub summary: bool,

    /// Print detailed mutation test summary table
    #[clap(long, help_heading = "Display options")]
    pub detailed: bool,
}

impl MutateTestArgs {
    /// Returns the flattened [`CoreBuildArgs`].
    pub fn build_args(&self) -> &CoreBuildArgs {
        &self.opts
    }

    pub async fn run(self) -> Result<()> {
        trace!(target: "forge::mutate", "executing mutation command");
        shell::set_shell(shell::Shell::from_args(self.opts.silent, self.json))?;
        println!(
            "{}",
            "[⠆] Starting Mutation Test. Go grab a cup of coffee ☕, it's going to take a while"
        );

        let mutation_test_outcome = self.execute_mutation_test().await?;
        println!();
        mutation_test_outcome.ensure_ok()?;

        Ok(())
    }

    /// Executes mutation test for the project
    ///
    /// This will trigger the build process and run tests that match the configured filter.
    /// On success mutants will get be generated for functions matching configured filter.
    /// On success tests matching configured filter will be executed for all mutants.
    ///
    /// Returns the mutation test results for all matching functions
    pub async fn execute_mutation_test(self) -> Result<MutationTestOutcome> {
        let (mut config, evm_opts) = self.load_config_and_evm_opts_emit_warnings()?;
        // Fetch project mutate and test filter
        let (mutate_filter, test_filter) = self.filter(&config);

        // Set up the project
        let mut project = config.project()?;

        // install missing dependencies
        if install::install_missing_dependencies(&mut config, self.build_args().silent) &&
            config.auto_detect_remappings
        {
            // need to re-configure here to also catch additional remappings
            config = self.load_config();
            project = config.project()?;
        }

        let (test_outcome, output) =
            self.ensure_valid_project(&project, &config, &evm_opts, test_filter.clone()).await?;

        // Ensure test outcome is ok, exit if any test is failing
        if test_outcome.failures().count() > 0 {
            test_outcome.ensure_ok()?;
        }

        let (mutants_output, mutants_len) =
            self.execute_mutation(&project, mutate_filter, output, &config)?;

        println!("[⠆] Testing Mutants...");
        let env = evm_opts.evm_env().await?;
        let test_backend = Backend::spawn(evm_opts.get_fork(&config, env.clone())).await;
        let progress_bar_reporter = ProgressReporter::spawn("Mutants".into(), mutants_len);

        let mut mutant_test_suite_results: BTreeMap<String, MutationTestSuiteResult> =
            BTreeMap::new();

        let mutation_project_root = project.root();
        let mutant_fuzz_runs = self.fuzz_runs.unwrap_or(0) as u32;
        let mutant_fuzz_seed = self.fuzz_seed.clone();
        for (contract_out_dir, contract_mutants) in mutants_output.into_iter() {
            let mut mutant_test_statuses: Vec<(Duration, MutantTestStatus)> =
                Vec::with_capacity(contract_mutants.len());
            let contract_mutants_start = Instant::now();

            // a file can have hundreds of mutants depending on design
            // we chunk there to prevent huge memory consumption
            // join_all which launches all the futures and polls
            let mut contract_mutants_iterator = contract_mutants.chunks(config.mutate.parallel);


            while let Some(mutant_chunks) = contract_mutants_iterator.next() {
                let mutant_data_iterator = mutant_chunks.iter().map(|mutant| {
                    (
                        mutation_project_root.clone(),
                        mutant.source.filename_as_str(),
                        mutant.as_source_string().expect("Failed to read a file"),
                    )
                });

                // we compile the projects here
                let mutant_project_and_compile_output: Vec<_> =
                    try_join_all(mutant_data_iterator.map(|(root, file_name, mutant_contents)| {
                        tokio::task::spawn_blocking(move || {
                            setup_and_compile_mutant(
                                root, 
                                file_name,
                                mutant_contents,
                                mutant_fuzz_runs.clone(),
                                mutant_fuzz_seed.clone()
                            )
                        })
                    }))
                    .await?
                    .into_iter()
                    .filter_map(|x| x.ok())
                    .collect();

                let test_output = join_all(mutant_project_and_compile_output.into_iter().map(
                    |(temp_project, mutant_compile_output, mutant_config)| {
                        test_mutant(
                            test_backend.clone(),
                            progress_bar_reporter.clone(),
                            test_filter.clone(),
                            temp_project,
                            mutant_config,
                            mutant_compile_output,
                            &evm_opts,
                            env.clone(),
                        )
                    },
                ))
                .await;

                mutant_test_statuses.extend(test_output);
            }

            let mutant_test_results: Vec<_> =
                std::iter::zip(contract_mutants, mutant_test_statuses)
                    .map(|(mutant, (duration, status))| {
                        MutantTestResult::new(duration, mutant, status)
                    })
                    .collect_vec();

            let has_survivors = mutant_test_results.iter().any(|result| result.survived());
            let contract_name = contract_out_dir
                .split(std::path::MAIN_SEPARATOR_STR)
                .nth(1)
                .ok_or(eyre!("Failed to parse contract name"))?;

            mutant_test_suite_results.insert(
                contract_name.to_string(),
                MutationTestSuiteResult::new(contract_mutants_start.elapsed(), mutant_test_results),
            );

            // Exit if fail fast is configured
            if self.fail_fast && has_survivors {
                break;
            }

            // Exit if number of maximum number of timeout tests is reached
            if mutant_test_suite_results.values().flat_map(|result| result.timeout()).count() >=
                config.mutate.maximum_timeout_test
            {
                break;
            }
        }

        // finish progress bar and clear it
        progress_bar_reporter.finish_and_clear();

        if self.json {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &mutant_test_suite_results
                        .values()
                        .flat_map(|x| x.mutation_test_results())
                        .collect::<Vec<_>>()
                )?
            );
            std::process::exit(0);
        }

        let mutation_test_outcome =
            MutationTestOutcome::new(self.allow_failure, mutant_test_suite_results);

        if self.summary {
            let mut reporter = MutationTestSummaryReporter::new(self.detailed);
            reporter.print_summary(&mutation_test_outcome);
        }

        println!("{}", mutation_test_outcome.summary());

        Ok(mutation_test_outcome)
    }

    /// Performs mutation on the project contracts
    pub fn execute_mutation(
        &self,
        project: &Project,
        mutate_filter: MutationProjectPathsAwareFilter,
        output: ProjectCompileOutput,
        config: &Config,
    ) -> Result<(StdHashMap<String, Vec<Mutant>>, usize)> {
        trace!(target: "forge::mutate", "running gambit");

        let mutator = MutatorConfigBuilder::new(
            project.solc.solc.clone(),
            config.optimizer,
            project.allowed_paths.paths().map(|x| x.to_owned()).collect_vec(),
            project.include_paths.paths().map(|x| x.to_owned()).collect(),
            config.remappings.clone(),
        )
        .build(config.src.clone(), output)?;

        if mutator.matching_function_count(&mutate_filter) == 0 {
            println!("\nNo functions match the provided pattern");
            println!("{}", mutate_filter.to_string());
            // Try to suggest a function when there's no match
            if let Some(ref function_pattern) = mutate_filter.args().function_pattern {
                let function_name = function_pattern.as_str();
                let candidates = mutator.get_function_names(&mutate_filter);
                if let Some(suggestion) = utils::did_you_mean(function_name, candidates).pop() {
                    println!("\nDid you mean `{suggestion}`?");
                }
                std::process::exit(0);
            }
        }

        if self.list {
            println!();

            let results = mutator.list(&mutate_filter);
            for (source, mutants) in results.iter() {
                for (name, functions) in mutants.iter() {
                    println!("{}:{}", source, name);
                    println!("\t{}", functions.join(" \n\t"));
                }
            }
            std::process::exit(0);
        }

        let spinner = MutatorSpinnerReporter::spawn("Generating Mutants...".into());

        let now = Instant::now();
        // init spinner
        let mutants_output =
            mutator.run_mutate(self.export, config.mutate.out.clone(), mutate_filter.clone())?;

        let elapsed = now.elapsed();

        drop(spinner);

        trace!(target: "forge::mutate", "finished running gambit");

        let mutants_len = mutants_output.iter().flat_map(|(_, v)| v).count();

        println!("Generated {} mutants in {:.2?}", mutants_len, elapsed);

        Ok((mutants_output, mutants_len))
    }

    /// Compiles and Tests the project to ensure no failing tests
    pub async fn ensure_valid_project(
        &self,
        project: &Project,
        config: &Config,
        evm_opts: &EvmOpts,
        filter: ProjectPathsAwareFilter,
    ) -> Result<(TestOutcome, ProjectCompileOutput)> {
        let compiler = ProjectCompiler::default();
        let output = compiler.compile(&project)?;
        // Create test options from general project settings
        // and compiler output
        let start = Instant::now();
        let test_reporter = MutatorSpinnerReporter::spawn("Running Project Tests...".into());
        let project_root = &project.paths.root;
        let toml = config.get_config_path();
        let profiles = get_available_profiles(toml)?;
        let env = evm_opts.evm_env().await?;

        let test_options: TestOptions = TestOptionsBuilder::default()
            .fuzz(config.mutate.fuzz)
            .invariant(config.invariant)
            .profiles(profiles)
            .build(&output, project_root)?;

        let runner_builder = MultiContractRunnerBuilder::default()
            .set_debug(false)
            .initial_balance(evm_opts.initial_balance)
            .evm_spec(config.evm_spec_id())
            .sender(evm_opts.sender)
            .with_fork(evm_opts.get_fork(&config, env.clone()))
            .with_cheats_config(CheatsConfig::new(&config, evm_opts.clone()))
            .with_test_options(test_options.clone());

        let mut runner = runner_builder.clone().build(
            project_root,
            output.clone(),
            env.clone(),
            evm_opts.clone(),
        )?;

        if runner.matching_test_function_count(&filter) == 0 {
            let filter_str = filter.to_string();
            if filter_str.is_empty() {
                println!(
                    "\nNo tests found in project! Forge looks for functions that starts with `test`."
                );
            } else {
                println!("\nNo tests match the provided pattern:");
                println!("{filter_str}");
                // Try to suggest a test when there's no match
                if let Some(ref test_pattern) = filter.args().test_pattern {
                    let test_name = test_pattern.as_str();
                    let candidates = runner.get_tests(&filter);
                    if let Some(suggestion) = utils::did_you_mean(test_name, candidates).pop() {
                        println!("\nDid you mean `{suggestion}`?");
                    }
                }
            }
        }

        let mut results = BTreeMap::new();
        // Set up test reporter channel
        let (tx, rx) = channel::<(String, SuiteResult)>();

        // Run tests
        let handle =
            tokio::task::spawn(async move { runner.test(&filter, tx, test_options).await });

        'outer: for (contract_name, suite_result) in rx {
            results.insert(contract_name.clone(), suite_result.clone());
            if suite_result.failures().count() > 0 {
                break 'outer
            }
        }
        let _results = handle.await?;

        // stop reporter
        drop(test_reporter);

        println!("Finished running project tests in {:2?}", start.elapsed());
        Ok((TestOutcome::new(results, false), output))
    }

    /// Returns the flattened [`MutateFilterArgs`] arguments merged with [`Config`].
    pub fn filter(
        &self,
        config: &Config,
    ) -> (MutationProjectPathsAwareFilter, ProjectPathsAwareFilter) {
        self.filter.merge_with_config(config)
    }
}

impl Provider for MutateTestArgs {
    fn metadata(&self) -> Metadata {
        Metadata::named("Mutation Test: Args ")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        let mut dict = Dict::default();

        let mut mutate_dict = Dict::default();
        // Override the fuzz and invariants run
        // We do not want fuzz and invariant tests to run once so the test
        // setup is faster.
        let mut fuzz_dict = Dict::default();
        if let Some(fuzz_seed) = self.fuzz_seed {
            fuzz_dict.insert("seed".to_string(), fuzz_seed.to_string().into());
        }
        if let Some(fuzz_runs) = self.fuzz_runs {
            fuzz_dict.insert("runs".to_string(), fuzz_runs.into());
        }
        mutate_dict.insert("fuzz".to_string(), fuzz_dict.into());
        // insert into config
        dict.insert("mutate".to_string(), mutate_dict.into());

        let mut invariant_dict = Dict::default();
        invariant_dict.insert("runs".to_string(), 0.into());
        dict.insert("invariant".to_string(), invariant_dict.into());

        Ok(Map::from([(Config::selected_profile(), dict)]))
    }
}

/// Creates a temp project from source project and compiles the project
pub fn setup_and_compile_mutant(
    mutation_project_root: PathBuf,
    mutant_file: String,
    mutant_contents: String,
    fuzz_runs: u32,
    fuzz_seed: Option<U256>
) -> Result<(TempProject, ProjectCompileOutput, Config)> {
    trace!(target: "forge::mutate", "setting up and compiling mutant");

    let start = Instant::now();

    // we do not support hardhat style testing
    let temp_project = TempProject::dapptools()?;
    let temp_project_root = temp_project.root();

    // copy project source code to temp dir
    copy_dir(mutation_project_root, temp_project_root)?;

    // load config for this temp project
    let mut config = Config::load_with_root(temp_project_root);
    // appends the root dir to the config folder variables
    // it's important
    config = config.canonic_at(temp_project_root);
    // override fuzz and invariant runs
    config.mutate.fuzz.runs = fuzz_runs;
    config.mutate.fuzz.seed = fuzz_seed;
    config.invariant.runs = 0;

    let mutant_file_path = temp_project_root.join(mutant_file);
    // Write Mutant contents to file in temp_directory
    fs::write(&mutant_file_path.as_path(), mutant_contents)?;

    debug!(
        duration = ?start.elapsed(),
        "compilation times",
    );

    let mut project = config.project()?;
    project.set_solc_jobs(4);

    let compile_output = project.compile().map_err(|_| eyre!("compilation failed"))?;

    trace!(target: "forge::mutate", "finishing setting up and compiling mutant");

    Ok((temp_project, compile_output, config))
}

/// Runs mutation test for a mutation temp project.
/// returns on the first failed test suite
pub async fn test_mutant(
    db: Backend,
    progress_bar: ProgressReporter,
    filter: ProjectPathsAwareFilter,
    temp_project: TempProject,
    config: Config,
    output: ProjectCompileOutput,
    evm_opts: &EvmOpts,
    env: Env,
) -> (Duration, MutantTestStatus) {
    trace!(target: "forge::mutate", "testing mutant");

    let start = Instant::now();

    let project_root = temp_project.root();
    let toml = config.get_config_path();
    let profiles = get_available_profiles(toml).expect("Failed to get profiles");

    let test_options: TestOptions = TestOptionsBuilder::default()
        .fuzz(config.mutate.fuzz)
        .invariant(config.invariant)
        .profiles(profiles)
        .build(&output, project_root)
        .expect("Failed to setup test options");

    let runner_builder = MultiContractRunnerBuilder::default()
        .set_debug(false)
        .initial_balance(evm_opts.initial_balance)
        .evm_spec(config.evm_spec_id())
        .sender(evm_opts.sender)
        .with_fork(evm_opts.get_fork(&config, env.clone()))
        .with_cheats_config(CheatsConfig::new(&config, evm_opts.clone()))
        .with_test_options(test_options.clone());

    let mut runner = runner_builder
        .build(project_root, output.clone(), env, evm_opts.clone())
        .expect("Failed to build test runner");

    // We use a thread and recv_timeout here because Gambit generates
    // valid solidity grammar mutants that leads to very long running
    // execution in REVM due to large amount of gas available.
    //
    // An example of a mutant generated is below, the test will take forever to end
    // because it's an infinite loop
    //
    // unchecked {
    //    UnaryOperatorMutation(`++` |==> `~`) of: `for (uint256 i = 0; i < owners.length; ++i)
    //    for (uint256 i = 0; i < owners.length; ~i) {
    //       balances[i] = balanceOf[owners[i]][ids[i]];
    //    }
    // }
    //
    //
    // mpsc channel for reporting test results
    let (tx, rx) = channel::<(String, SuiteResult)>();
    let _ = tokio::task::spawn_blocking(move || {
        // We create ThreadPool here because it's possible to have
        // a long running test that attaches itself to the global rayon ThreadPool
        // This would prevent other rayon tasks from executing leading to a deadlock
        // Creating a pool means we can isolate this and prevent it from affecting
        // other tasks.
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .expect("Failed to setup thread pool ");

        pool.install(|| runner.test_with_backend(db, &filter, tx, test_options));
    });

    let mut status: MutantTestStatus = MutantTestStatus::Survived;
    loop {
        match rx.recv_timeout(config.mutate.test_timeout) {
            Ok((_, suite_result)) => {
                if suite_result.failures().count() > 0 {
                    status = MutantTestStatus::Killed;
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                trace!(target: "forge:mutate", "test timeout");
                status = MutantTestStatus::Timeout;
                break;
            }
            _ => {
                break;
            }
        }
    }

    progress_bar.increment();

    trace!(target: "forge::mutate", "received mutant test results");

    (start.elapsed(), status)
}
