use super::{
    build::BuildArgs, coverage::CoverageArgs, doc::DocArgs, fmt::FmtArgs,
    snapshot::GasSnapshotArgs, test::TestArgs,
};
use alloy_primitives::map::HashSet;
use clap::Parser;
use eyre::Result;
use foundry_cli::utils::{self, FoundryPathExt, LoadConfig};
use foundry_config::Config;
use parking_lot::Mutex;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::process::Command as TokioCommand;
use watchexec::{
    action::ActionHandler,
    command::{Command, Program},
    job::{CommandState, Job},
    paths::summarise_events_to_env,
    Watchexec,
};
use watchexec_events::{Event, Priority, ProcessEnd};
use watchexec_signals::Signal;
use yansi::{Color, Paint};

type SpawnHook = Arc<dyn Fn(&[Event], &mut TokioCommand) + Send + Sync + 'static>;

#[derive(Clone, Debug, Default, Parser)]
#[command(next_help_heading = "Watch options")]
pub struct WatchArgs {
    /// Watch the given files or directories for changes.
    ///
    /// If no paths are provided, the source and test directories of the project are watched.
    #[arg(long, short, num_args(0..), value_name = "PATH")]
    pub watch: Option<Vec<PathBuf>>,

    /// Do not restart the command while it's still running.
    #[arg(long)]
    pub no_restart: bool,

    /// Explicitly re-run all tests when a change is made.
    ///
    /// By default, only the tests of the last modified test file are executed.
    #[arg(long)]
    pub run_all: bool,

    /// File update debounce delay.
    ///
    /// During the delay, incoming change events are accumulated and
    /// only once the delay has passed, is an action taken. Note that
    /// this does not mean a command will be started: if --no-restart is
    /// given and a command is already running, the outcome of the
    /// action will be to do nothing.
    ///
    /// Defaults to 50ms. Parses as decimal seconds by default, but
    /// using an integer with the `ms` suffix may be more convenient.
    ///
    /// When using --poll mode, you'll want a larger duration, or risk
    /// overloading disk I/O.
    #[arg(long, value_name = "DELAY")]
    pub watch_delay: Option<String>,
}

impl WatchArgs {
    /// Creates a new [`watchexec::Config`].
    ///
    /// If paths were provided as arguments the these will be used as the watcher's pathset,
    /// otherwise the path the closure returns will be used.
    pub fn watchexec_config<PS: IntoIterator<Item = P>, P: Into<PathBuf>>(
        &self,
        default_paths: impl FnOnce() -> Result<PS>,
    ) -> Result<watchexec::Config> {
        self.watchexec_config_generic(default_paths, None)
    }

    /// Creates a new [`watchexec::Config`] with a custom command spawn hook.
    ///
    /// If paths were provided as arguments the these will be used as the watcher's pathset,
    /// otherwise the path the closure returns will be used.
    pub fn watchexec_config_with_override<PS: IntoIterator<Item = P>, P: Into<PathBuf>>(
        &self,
        default_paths: impl FnOnce() -> Result<PS>,
        spawn_hook: impl Fn(&[Event], &mut TokioCommand) + Send + Sync + 'static,
    ) -> Result<watchexec::Config> {
        self.watchexec_config_generic(default_paths, Some(Arc::new(spawn_hook)))
    }

    fn watchexec_config_generic<PS: IntoIterator<Item = P>, P: Into<PathBuf>>(
        &self,
        default_paths: impl FnOnce() -> Result<PS>,
        spawn_hook: Option<SpawnHook>,
    ) -> Result<watchexec::Config> {
        let mut paths = self.watch.as_deref().unwrap_or_default();
        let storage: Vec<_>;
        if paths.is_empty() {
            storage = default_paths()?.into_iter().map(Into::into).filter(|p| p.exists()).collect();
            paths = &storage;
        }
        self.watchexec_config_inner(paths, spawn_hook)
    }

    fn watchexec_config_inner(
        &self,
        paths: &[PathBuf],
        spawn_hook: Option<SpawnHook>,
    ) -> Result<watchexec::Config> {
        let config = watchexec::Config::default();

        config.on_error(|err| {
            let _ = sh_eprintln!("[[{err:?}]]");
        });

        if let Some(delay) = &self.watch_delay {
            config.throttle(utils::parse_delay(delay)?);
        }

        config.pathset(paths.iter().map(|p| p.as_path()));

        let n_path_args = self.watch.as_deref().unwrap_or_default().len();
        let base_command = Arc::new(watch_command(cmd_args(n_path_args)));

        let id = watchexec::Id::default();
        let quit_again = Arc::new(AtomicU8::new(0));
        let stop_timeout = Duration::from_secs(5);
        let no_restart = self.no_restart;
        let stop_signal = Signal::Terminate;
        config.on_action(move |mut action| {
            let base_command = base_command.clone();
            let job = action.get_or_create_job(id, move || base_command.clone());

            let events = action.events.clone();
            let spawn_hook = spawn_hook.clone();
            job.set_spawn_hook(move |command, _| {
                // https://github.com/watchexec/watchexec/blob/72f069a8477c679e45f845219276b0bfe22fed79/crates/cli/src/emits.rs#L9
                let env = summarise_events_to_env(events.iter());
                for (k, v) in env {
                    command.command_mut().env(format!("WATCHEXEC_{k}_PATH"), v);
                }

                if let Some(spawn_hook) = &spawn_hook {
                    spawn_hook(&events, command.command_mut());
                }
            });

            let clear_screen = || {
                let _ = clearscreen::clear();
            };

            let quit = |mut action: ActionHandler| {
                match quit_again.fetch_add(1, Ordering::Relaxed) {
                    0 => {
                        let _ = sh_eprintln!(
                            "[Waiting {stop_timeout:?} for processes to exit before stopping... \
                             Ctrl-C again to exit faster]"
                        );
                        action.quit_gracefully(stop_signal, stop_timeout);
                    }
                    1 => action.quit_gracefully(Signal::ForceStop, Duration::ZERO),
                    _ => action.quit(),
                }

                action
            };

            let signals = action.signals().collect::<Vec<_>>();

            if signals.contains(&Signal::Terminate) || signals.contains(&Signal::Interrupt) {
                return quit(action);
            }

            // Only filesystem events below here (or empty synthetic events).
            if action.paths().next().is_none() && !action.events.iter().any(|e| e.is_empty()) {
                debug!("no filesystem or synthetic events, skip without doing more");
                return action;
            }

            job.run({
                let job = job.clone();
                move |context| {
                    if context.current.is_running() && no_restart {
                        return;
                    }
                    job.restart_with_signal(stop_signal, stop_timeout);
                    job.run({
                        let job = job.clone();
                        move |context| {
                            clear_screen();
                            setup_process(job, &context.command)
                        }
                    });
                }
            });

            action
        });

        Ok(config)
    }
}

fn setup_process(job: Job, _command: &Command) {
    tokio::spawn(async move {
        job.to_wait().await;
        job.run(move |context| end_of_process(context.current));
    });
}

fn end_of_process(state: &CommandState) {
    let CommandState::Finished { status, started, finished } = state else {
        return;
    };

    let duration = *finished - *started;
    let timings = true;
    let timing = if timings { format!(", lasted {duration:?}") } else { String::new() };
    let (msg, fg) = match status {
        ProcessEnd::ExitError(code) => (format!("Command exited with {code}{timing}"), Color::Red),
        ProcessEnd::ExitSignal(sig) => {
            (format!("Command killed by {sig:?}{timing}"), Color::Magenta)
        }
        ProcessEnd::ExitStop(sig) => (format!("Command stopped by {sig:?}{timing}"), Color::Blue),
        ProcessEnd::Continued => (format!("Command continued{timing}"), Color::Cyan),
        ProcessEnd::Exception(ex) => {
            (format!("Command ended by exception {ex:#x}{timing}"), Color::Yellow)
        }
        ProcessEnd::Success => (format!("Command was successful{timing}"), Color::Green),
    };

    let quiet = false;
    if !quiet {
        let _ = sh_eprintln!("{}", format!("[{msg}]").paint(fg.foreground()));
    }
}

/// Runs the given [`watchexec::Config`].
pub async fn run(config: watchexec::Config) -> Result<()> {
    let wx = Watchexec::with_config(config)?;
    wx.send_event(Event::default(), Priority::Urgent).await?;
    wx.main().await??;
    Ok(())
}

/// Executes a [`Watchexec`] that listens for changes in the project's src dir and reruns `forge
/// build`
pub async fn watch_build(args: BuildArgs) -> Result<()> {
    let config = args.watchexec_config()?;
    run(config).await
}

/// Executes a [`Watchexec`] that listens for changes in the project's src dir and reruns `forge
/// snapshot`
pub async fn watch_gas_snapshot(args: GasSnapshotArgs) -> Result<()> {
    let config = args.watchexec_config()?;
    run(config).await
}

/// Executes a [`Watchexec`] that listens for changes in the project's src dir and reruns `forge
/// test`
pub async fn watch_test(args: TestArgs) -> Result<()> {
    let config: Config = args.build.load_config()?;
    let filter = args.filter(&config);
    // Marker to check whether to override the command.
    let no_reconfigure = filter.args().test_pattern.is_some() ||
        filter.args().path_pattern.is_some() ||
        filter.args().contract_pattern.is_some() ||
        args.watch.run_all;

    let last_test_files = Mutex::new(HashSet::<String>::default());
    let project_root = config.root.to_string_lossy().into_owned();
    let config = args.watch.watchexec_config_with_override(
        || Ok([&config.test, &config.src]),
        move |events, command| {
            let mut changed_sol_test_files: HashSet<_> = events
                .iter()
                .flat_map(|e| e.paths())
                .filter(|(path, _)| path.is_sol_test())
                .filter_map(|(path, _)| path.to_str())
                .map(str::to_string)
                .collect();

            if changed_sol_test_files.len() > 1 {
                // Run all tests if multiple files were changed at once, for example when running
                // `forge fmt`.
                return;
            }

            if changed_sol_test_files.is_empty() {
                // Reuse the old test files if a non-test file was changed.
                let last = last_test_files.lock();
                if last.is_empty() {
                    return;
                }
                changed_sol_test_files = last.clone();
            }

            // append `--match-path` glob
            let mut file = changed_sol_test_files.iter().next().expect("test file present").clone();

            // remove the project root dir from the detected file
            if let Some(f) = file.strip_prefix(&project_root) {
                file = f.trim_start_matches('/').to_string();
            }

            trace!(?file, "reconfigure test command");

            // Before appending `--match-path`, check if it already exists
            if !no_reconfigure {
                command.arg("--match-path").arg(file);
            }
        },
    )?;
    run(config).await
}

pub async fn watch_coverage(args: CoverageArgs) -> Result<()> {
    let config = args.watch().watchexec_config(|| {
        let config = args.load_config()?;
        Ok([config.test, config.src])
    })?;
    run(config).await
}

pub async fn watch_fmt(args: FmtArgs) -> Result<()> {
    let config = args.watch.watchexec_config(|| {
        let config = args.load_config()?;
        Ok([config.src, config.test, config.script])
    })?;
    run(config).await
}

/// Executes a [`Watchexec`] that listens for changes in the project's sources directory
pub async fn watch_doc(args: DocArgs) -> Result<()> {
    let config = args.watch.watchexec_config(|| {
        let config = args.config()?;
        Ok([config.src])
    })?;
    run(config).await
}

/// Converts a list of arguments to a `watchexec::Command`.
///
/// The first index in `args` is the path to the executable.
///
/// # Panics
///
/// Panics if `args` is empty.
fn watch_command(mut args: Vec<String>) -> Command {
    debug_assert!(!args.is_empty());
    let prog = args.remove(0);
    Command { program: Program::Exec { prog: prog.into(), args }, options: Default::default() }
}

/// Returns the env args without the `--watch` flag from the args for the Watchexec command
fn cmd_args(num: usize) -> Vec<String> {
    clean_cmd_args(num, std::env::args().collect())
}

#[instrument(level = "debug", ret)]
fn clean_cmd_args(num: usize, mut cmd_args: Vec<String>) -> Vec<String> {
    if let Some(pos) = cmd_args.iter().position(|arg| arg == "--watch" || arg == "-w") {
        cmd_args.drain(pos..=(pos + num));
    }

    // There's another edge case where short flags are combined into one which is supported by clap,
    // like `-vw` for verbosity and watch
    // this removes any `w` from concatenated short flags
    if let Some(pos) = cmd_args.iter().position(|arg| {
        fn contains_w_in_short(arg: &str) -> Option<bool> {
            let mut iter = arg.chars().peekable();
            if *iter.peek()? != '-' {
                return None
            }
            iter.next();
            if *iter.peek()? == '-' {
                return None
            }
            Some(iter.any(|c| c == 'w'))
        }
        contains_w_in_short(arg).unwrap_or(false)
    }) {
        let clean_arg = cmd_args[pos].replace('w', "");
        if clean_arg == "-" {
            cmd_args.remove(pos);
        } else {
            cmd_args[pos] = clean_arg;
        }
    }

    cmd_args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cmd_args() {
        let args = vec!["-vw".to_string()];
        let cleaned = clean_cmd_args(0, args);
        assert_eq!(cleaned, vec!["-v".to_string()]);
    }
}
