//! terminal utils
use foundry_compilers::{
    artifacts::remappings::Remapping,
    report::{self, BasicStdoutReporter, Reporter},
};
use once_cell::sync::Lazy;
use semver::Version;
use std::{
    io,
    io::{prelude::*, IsTerminal},
    path::{Path, PathBuf},
    sync::mpsc::{self, TryRecvError},
    thread,
    time::Duration,
};
use yansi::Paint;

/// Some spinners
// https://github.com/gernest/wow/blob/master/spin/spinners.go
pub static SPINNERS: &[&[&str]] = &[
    &["⠃", "⠊", "⠒", "⠢", "⠆", "⠰", "⠔", "⠒", "⠑", "⠘"],
    &[" ", "⠁", "⠉", "⠙", "⠚", "⠖", "⠦", "⠤", "⠠"],
    &["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"],
    &["▹▹▹▹▹", "▸▹▹▹▹", "▹▸▹▹▹", "▹▹▸▹▹", "▹▹▹▸▹", "▹▹▹▹▸"],
    &[" ", "▘", "▀", "▜", "█", "▟", "▄", "▖"],
];

static TERM_SETTINGS: Lazy<TermSettings> = Lazy::new(TermSettings::from_env);

/// Helper type to determine the current tty
pub struct TermSettings {
    indicate_progress: bool,
}

impl TermSettings {
    /// Returns a new [`TermSettings`], configured from the current environment.
    pub fn from_env() -> Self {
        Self { indicate_progress: std::io::stdout().is_terminal() }
    }
}

#[allow(missing_docs)]
pub struct Spinner {
    indicator: &'static [&'static str],
    no_progress: bool,
    message: String,
    idx: usize,
}

#[allow(unused)]
#[allow(missing_docs)]
impl Spinner {
    pub fn new(msg: impl Into<String>) -> Self {
        Self::with_indicator(SPINNERS[0], msg)
    }

    pub fn with_indicator(indicator: &'static [&'static str], msg: impl Into<String>) -> Self {
        Self {
            indicator,
            no_progress: !TERM_SETTINGS.indicate_progress,
            message: msg.into(),
            idx: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.no_progress {
            return
        }

        let indicator = self.indicator[self.idx % self.indicator.len()].green();
        let indicator = Paint::new(format!("[{indicator}]")).bold();
        print!("\r\x33[2K\r{indicator} {}", self.message);
        io::stdout().flush().unwrap();

        self.idx = self.idx.wrapping_add(1);
    }

    pub fn message(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
    }
}

/// A spinner used as [`report::Reporter`]
///
/// This reporter will prefix messages with a spinning cursor
#[derive(Debug)]
#[must_use = "Terminates the spinner on drop"]
pub struct SpinnerReporter {
    /// The sender to the spinner thread.
    sender: mpsc::Sender<SpinnerMsg>,
}

impl SpinnerReporter {
    /// Spawns the [`Spinner`] on a new thread
    ///
    /// The spinner's message will be updated via the `reporter` events
    ///
    /// On drop the channel will disconnect and the thread will terminate
    pub fn spawn() -> Self {
        let (sender, rx) = mpsc::channel::<SpinnerMsg>();

        std::thread::Builder::new()
            .name("spinner".into())
            .spawn(move || {
                let mut spinner = Spinner::new("Compiling...");
                loop {
                    spinner.tick();
                    match rx.try_recv() {
                        Ok(SpinnerMsg::Msg(msg)) => {
                            spinner.message(msg);
                            // new line so past messages are not overwritten
                            println!();
                        }
                        Ok(SpinnerMsg::Shutdown(ack)) => {
                            // end with a newline
                            println!();
                            let _ = ack.send(());
                            break
                        }
                        Err(TryRecvError::Disconnected) => break,
                        Err(TryRecvError::Empty) => thread::sleep(Duration::from_millis(100)),
                    }
                }
            })
            .expect("failed to spawn thread");

        Self { sender }
    }

    fn send_msg(&self, msg: impl Into<String>) {
        let _ = self.sender.send(SpinnerMsg::Msg(msg.into()));
    }
}

enum SpinnerMsg {
    Msg(String),
    Shutdown(mpsc::Sender<()>),
}

impl Drop for SpinnerReporter {
    fn drop(&mut self) {
        let (tx, rx) = mpsc::channel();
        if self.sender.send(SpinnerMsg::Shutdown(tx)).is_ok() {
            let _ = rx.recv();
        }
    }
}

impl Reporter for SpinnerReporter {
    fn on_compiler_spawn(&self, compiler_name: &str, version: &Version, dirty_files: &[PathBuf]) {
        self.send_msg(format!(
            "Compiling {} files with {} {}.{}.{}",
            dirty_files.len(),
            compiler_name,
            version.major,
            version.minor,
            version.patch
        ));
    }

    fn on_compiler_success(&self, compiler_name: &str, version: &Version, duration: &Duration) {
        self.send_msg(format!(
            "{} {}.{}.{} finished in {duration:.2?}",
            compiler_name, version.major, version.minor, version.patch
        ));
    }

    fn on_solc_installation_start(&self, version: &Version) {
        self.send_msg(format!("Installing Solc version {version}"));
    }

    fn on_solc_installation_success(&self, version: &Version) {
        self.send_msg(format!("Successfully installed Solc {version}"));
    }

    fn on_solc_installation_error(&self, version: &Version, error: &str) {
        self.send_msg(format!("Failed to install Solc {version}: {error}").red().to_string());
    }

    fn on_unresolved_imports(&self, imports: &[(&Path, &Path)], remappings: &[Remapping]) {
        self.send_msg(report::format_unresolved_imports(imports, remappings));
    }
}

/// If the output medium is terminal, this calls `f` within the [`SpinnerReporter`] that displays a
/// spinning cursor to display solc progress.
///
/// If no terminal is available this falls back to common `println!` in [`BasicStdoutReporter`].
pub fn with_spinner_reporter<T>(f: impl FnOnce() -> T) -> T {
    let reporter = if TERM_SETTINGS.indicate_progress {
        report::Report::new(SpinnerReporter::spawn())
    } else {
        report::Report::new(BasicStdoutReporter::default())
    };
    report::with_scoped(&reporter, f)
}

/// A spinner used for reporting in Mutation tests
///
/// This reporter will prefix messages with a spinning cursor
#[derive(Debug)]
pub struct MutatorSpinnerReporter {
    /// The sender to the spinner thread.
    sender: mpsc::Sender<SpinnerMsg>
}

impl Drop for MutatorSpinnerReporter {
    fn drop(&mut self) {
        let (tx, rx) = mpsc::channel();
        if self.sender.send(SpinnerMsg::Shutdown(tx)).is_ok() {
            let _ = rx.recv();
        }
    }
}

impl MutatorSpinnerReporter {
    /// Spawns the [`Spinner`] on a new thread
    ///
    /// The spinner's message will be updated via the `reporter` events
    ///
    /// On drop the channel will disconnect and the thread will terminate
    pub fn spawn(message: String) -> Self {
        let (sender, rx) = mpsc::channel::<SpinnerMsg>();

        std::thread::Builder::new()
            .name("mutator".into())
            .spawn(move || {
                let mut spinner = Spinner::new(message);
                loop {
                    spinner.tick();
                    match rx.try_recv() {
                        Ok(SpinnerMsg::Msg(msg)) => {
                            spinner.message(msg);
                            // new line so past messages are not overwritten
                            println!();
                        }
                        Ok(SpinnerMsg::Shutdown(ack)) => {
                            // end with a newline
                            println!();
                            let _ = ack.send(());
                            break
                        }
                        Err(TryRecvError::Disconnected) => break,
                        Err(TryRecvError::Empty) => thread::sleep(Duration::from_millis(100)),
                    }
                }
            })
            .expect("failed to spawn thread");

        MutatorSpinnerReporter { sender }
    }

    
    // fn send_msg(&self, msg: impl Into<String>) {
    //     let _ = self.sender.send(SpinnerMsg::Msg(msg.into()));
    // }
}


enum ProgressMsg {
    Increment,
    Shutdown(mpsc::Sender<()>),
}

/// A progress bar used for reporting progess
///
/// Spins up a dedicated thread.
#[derive(Debug, Clone)]
pub struct ProgressReporter {
    /// The sender to the spinner thread.
    sender: mpsc::Sender<ProgressMsg>
}

impl ProgressReporter {
    /// Spawns a progress bar on a new thread
    ///
    ///
    /// On drop the channel will disconnect and the thread will terminate
    pub fn spawn(label: String, size: usize) -> Self {
        let (sender, rx) = mpsc::channel::<ProgressMsg>();

        std::thread::Builder::new()
            .name("mutator".into())
            .spawn(move || {
                let pb = indicatif::ProgressBar::new(size as u64);
                let mut template =
                    "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ".to_string();
                template += &label;
                template += " ({eta})";
                pb.set_style(
                    indicatif::ProgressStyle::with_template(&template)
                        .unwrap()
                        .with_key("eta", eta_key)
                        .progress_chars("#>-"),
                );
                pb.set_position(0);

                let mut current_index = 0;
                loop {
                    pb.tick();
                    match rx.try_recv() {
                        Ok(ProgressMsg::Increment) => {
                            current_index += 1;
                            // new line so past messages are not overwritten
                            pb.set_position(current_index);
                        }
                        Ok(ProgressMsg::Shutdown(ack)) => {
                            pb.finish_and_clear();
                            let _ = ack.send(());
                            break
                        }
                        Err(TryRecvError::Disconnected) => break,
                        Err(TryRecvError::Empty) => thread::sleep(Duration::from_millis(100)),
                    }
                }
            })
            .expect("failed to spawn thread");

        Self { sender }
    }

    /// Increment the progress bar    
    pub fn increment(&self) {
        let _ = self.sender.send(ProgressMsg::Increment);
    }

    /// Finish and clear progress bar
    pub fn finish_and_clear(&self) {
        let (tx, rx) = mpsc::channel();
        if self.sender.send(ProgressMsg::Shutdown(tx)).is_ok() {
            let _ = rx.recv();
        }
    }
}

fn eta_key(state: &indicatif::ProgressState, f: &mut dyn std::fmt::Write) {
    write!(f, "{:.1}s", state.eta().as_secs_f64()).unwrap()
}

#[macro_export]
/// Displays warnings on the cli
macro_rules! cli_warn {
    ($($arg:tt)*) => {
        eprintln!(
            "{}{} {}",
            yansi::Painted::new("warning").yellow().bold(),
            yansi::Painted::new(":").bold(),
            format_args!($($arg)*)
        )
    }
}

pub use cli_warn;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn can_spin() {
        let mut s = Spinner::new("Compiling".to_string());
        let ticks = 50;
        for _ in 0..ticks {
            std::thread::sleep(std::time::Duration::from_millis(100));
            s.tick();
        }
    }

    #[test]
    fn can_format_properly() {
        let r = SpinnerReporter::spawn();
        let remappings: Vec<Remapping> = vec![
            "library/=library/src/".parse().unwrap(),
            "weird-erc20/=lib/weird-erc20/src/".parse().unwrap(),
            "ds-test/=lib/ds-test/src/".parse().unwrap(),
            "openzeppelin-contracts/=lib/openzeppelin-contracts/contracts/".parse().unwrap(),
        ];
        let unresolved = vec![(Path::new("./src/Import.sol"), Path::new("src/File.col"))];
        r.on_unresolved_imports(&unresolved, &remappings);
        // formats:
        // [⠒] Unable to resolve imports:
        //       "./src/Import.sol" in "src/File.col"
        // with remappings:
        //       library/=library/src/
        //       weird-erc20/=lib/weird-erc20/src/
        //       ds-test/=lib/ds-test/src/
        //       openzeppelin-contracts/=lib/openzeppelin-contracts/contracts/
    }
}
