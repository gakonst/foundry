use alloy_json_abi::JsonAbi;
use alloy_primitives::U256;
use alloy_provider::{network::AnyNetwork, Provider};
use eyre::{ContextCompat, Result};
use foundry_common::{
    provider::{ProviderBuilder, RetryProvider},
    shell,
};
use foundry_config::{Chain, Config};
use serde::de::DeserializeOwned;
use std::{
    ffi::OsStr,
    future::Future,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tracing_subscriber::prelude::*;

mod cmd;
pub use cmd::*;

mod suggestions;
pub use suggestions::*;

mod abi;
pub use abi::*;

// reexport all `foundry_config::utils`
#[doc(hidden)]
pub use foundry_config::utils::*;

/// Deterministic fuzzer seed used for gas snapshots and coverage reports.
///
/// The keccak256 hash of "foundry rulez"
pub const STATIC_FUZZ_SEED: [u8; 32] = [
    0x01, 0x00, 0xfa, 0x69, 0xa5, 0xf1, 0x71, 0x0a, 0x95, 0xcd, 0xef, 0x94, 0x88, 0x9b, 0x02, 0x84,
    0x5d, 0x64, 0x0b, 0x19, 0xad, 0xf0, 0xe3, 0x57, 0xb8, 0xd4, 0xbe, 0x7d, 0x49, 0xee, 0x70, 0xe6,
];

/// Useful extensions to [`std::path::Path`].
pub trait FoundryPathExt {
    /// Returns true if the [`Path`] ends with `.t.sol`
    fn is_sol_test(&self) -> bool;

    /// Returns true if the  [`Path`] has a `sol` extension
    fn is_sol(&self) -> bool;

    /// Returns true if the  [`Path`] has a `yul` extension
    fn is_yul(&self) -> bool;
}

impl<T: AsRef<Path>> FoundryPathExt for T {
    fn is_sol_test(&self) -> bool {
        self.as_ref()
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.ends_with(".t.sol"))
            .unwrap_or_default()
    }

    fn is_sol(&self) -> bool {
        self.as_ref().extension() == Some(std::ffi::OsStr::new("sol"))
    }

    fn is_yul(&self) -> bool {
        self.as_ref().extension() == Some(std::ffi::OsStr::new("yul"))
    }
}

/// Initializes a tracing Subscriber for logging
pub fn subscriber() {
    let registry = tracing_subscriber::Registry::default()
        .with(tracing_subscriber::EnvFilter::from_default_env());
    #[cfg(feature = "tracy")]
    let registry = registry.with(tracing_tracy::TracyLayer::default());
    registry.with(tracing_subscriber::fmt::layer()).init()
}

pub fn abi_to_solidity(abi: &JsonAbi, name: &str) -> Result<String> {
    let s = abi.to_sol(name, None);
    let s = forge_fmt::format(&s)?;
    Ok(s)
}

/// Returns a [RetryProvider] instantiated using [Config]'s
/// RPC
pub fn get_provider(config: &Config) -> Result<RetryProvider> {
    get_provider_builder(config)?.build()
}

/// Returns a [ProviderBuilder] instantiated using [Config] values.
///
/// Defaults to `http://localhost:8545` and `Mainnet`.
pub fn get_provider_builder(config: &Config) -> Result<ProviderBuilder> {
    let url = config.get_rpc_url_or_localhost_http()?;
    let mut builder = ProviderBuilder::new(url.as_ref());

    if let Ok(chain) = config.chain.unwrap_or_default().try_into() {
        builder = builder.chain(chain);
    }

    if let Some(jwt) = config.get_rpc_jwt_secret()? {
        builder = builder.jwt(jwt.as_ref());
    }

    if let Some(rpc_timeout) = config.eth_rpc_timeout {
        builder = builder.timeout(Duration::from_secs(rpc_timeout));
    }

    if let Some(rpc_headers) = config.eth_rpc_headers.clone() {
        builder = builder.headers(rpc_headers);
    }

    Ok(builder)
}

pub async fn get_chain<P>(chain: Option<Chain>, provider: P) -> Result<Chain>
where
    P: Provider<AnyNetwork>,
{
    match chain {
        Some(chain) => Ok(chain),
        None => Ok(Chain::from_id(provider.get_chain_id().await?)),
    }
}

/// Parses an ether value from a string.
///
/// The amount can be tagged with a unit, e.g. "1ether".
///
/// If the string represents an untagged amount (e.g. "100") then
/// it is interpreted as wei.
pub fn parse_ether_value(value: &str) -> Result<U256> {
    Ok(if value.starts_with("0x") {
        U256::from_str_radix(value, 16)?
    } else {
        alloy_dyn_abi::DynSolType::coerce_str(&alloy_dyn_abi::DynSolType::Uint(256), value)?
            .as_uint()
            .wrap_err("Could not parse ether value from string")?
            .0
    })
}

/// Parses a `T` from a string using [`serde_json::from_str`].
pub fn parse_json<T: DeserializeOwned>(value: &str) -> serde_json::Result<T> {
    serde_json::from_str(value)
}

/// Parses a `Duration` from a &str
pub fn parse_delay(delay: &str) -> Result<Duration> {
    let delay = if delay.ends_with("ms") {
        let d: u64 = delay.trim_end_matches("ms").parse()?;
        Duration::from_millis(d)
    } else {
        let d: f64 = delay.parse()?;
        let delay = (d * 1000.0).round();
        if delay.is_infinite() || delay.is_nan() || delay.is_sign_negative() {
            eyre::bail!("delay must be finite and non-negative");
        }

        Duration::from_millis(delay as u64)
    };
    Ok(delay)
}

/// Returns the current time as a [`Duration`] since the Unix epoch.
pub fn now() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("time went backwards")
}

/// Runs the `future` in a new [`tokio::runtime::Runtime`]
pub fn block_on<F: Future>(future: F) -> F::Output {
    let rt = tokio::runtime::Runtime::new().expect("could not start tokio rt");
    rt.block_on(future)
}

/// Loads a dotenv file, from the cwd and the project root, ignoring potential failure.
///
/// We could use `warn!` here, but that would imply that the dotenv file can't configure
/// the logging behavior of Foundry.
///
/// Similarly, we could just use `eprintln!`, but colors are off limits otherwise dotenv is implied
/// to not be able to configure the colors. It would also mess up the JSON output.
pub fn load_dotenv() {
    let load = |p: &Path| {
        dotenvy::from_path(p.join(".env")).ok();
    };

    // we only want the .env file of the cwd and project root
    // `find_project_root` calls `current_dir` internally so both paths are either both `Ok` or
    // both `Err`
    if let (Ok(cwd), Ok(prj_root)) = (std::env::current_dir(), find_project_root(None)) {
        load(&prj_root);
        if cwd != prj_root {
            // prj root and cwd can be identical
            load(&cwd);
        }
    };
}

/// Sets the default [`yansi`] color output condition.
pub fn enable_paint() {
    let enable = yansi::Condition::os_support() && yansi::Condition::tty_and_color_live();
    yansi::whenever(yansi::Condition::cached(enable));
}

/// Useful extensions to [`std::process::Command`].
pub trait CommandUtils {
    /// Returns the command's output if execution is successful, otherwise, throws an error.
    fn exec(&mut self) -> Result<Output>;

    /// Returns the command's stdout if execution is successful, otherwise, throws an error.
    fn get_stdout_lossy(&mut self) -> Result<String>;
}

impl CommandUtils for Command {
    #[track_caller]
    fn exec(&mut self) -> Result<Output> {
        trace!(command=?self, "executing");

        let output = self.output()?;

        trace!(code=?output.status.code(), ?output);

        if output.status.success() {
            Ok(output)
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stdout = stdout.trim();
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr = stderr.trim();
            let msg = if stdout.is_empty() {
                stderr.to_string()
            } else if stderr.is_empty() {
                stdout.to_string()
            } else {
                format!("stdout:\n{stdout}\n\nstderr:\n{stderr}")
            };

            let mut name = self.get_program().to_string_lossy();
            if let Some(arg) = self.get_args().next() {
                let arg = arg.to_string_lossy();
                if !arg.starts_with('-') {
                    let name = name.to_mut();
                    name.push(' ');
                    name.push_str(&arg);
                }
            }

            let mut err = match output.status.code() {
                Some(code) => format!("{name} exited with code {code}"),
                None => format!("{name} terminated by a signal"),
            };
            if !msg.is_empty() {
                err.push(':');
                err.push(if msg.lines().count() == 0 { ' ' } else { '\n' });
                err.push_str(&msg);
            }
            Err(eyre::eyre!(err))
        }
    }

    #[track_caller]
    fn get_stdout_lossy(&mut self) -> Result<String> {
        let output = self.exec()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().into())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Git<'a> {
    pub root: &'a Path,
    pub quiet: bool,
    pub shallow: bool,
}

impl<'a> Git<'a> {
    #[inline]
    pub fn new(root: &'a Path) -> Self {
        Self { root, quiet: shell::is_quiet(), shallow: false }
    }

    #[inline]
    pub fn from_config(config: &'a Config) -> Self {
        Self::new(config.root.as_path())
    }

    pub fn root_of(relative_to: &Path) -> Result<PathBuf> {
        let output = Self::cmd_no_root()
            .current_dir(relative_to)
            .args(["rev-parse", "--show-toplevel"])
            .get_stdout_lossy()?;
        Ok(PathBuf::from(output))
    }

    pub fn clone_with_branch(
        shallow: bool,
        from: impl AsRef<OsStr>,
        branch: impl AsRef<OsStr>,
        to: Option<impl AsRef<OsStr>>,
    ) -> Result<()> {
        Self::cmd_no_root()
            .stderr(Stdio::inherit())
            .args(["clone", "--recurse-submodules"])
            .args(shallow.then_some("--depth=1"))
            .args(shallow.then_some("--shallow-submodules"))
            .arg("-b")
            .arg(branch)
            .arg(from)
            .args(to)
            .exec()
            .map(drop)
    }

    pub fn clone(
        shallow: bool,
        from: impl AsRef<OsStr>,
        to: Option<impl AsRef<OsStr>>,
    ) -> Result<()> {
        Self::cmd_no_root()
            .stderr(Stdio::inherit())
            .args(["clone", "--recurse-submodules"])
            .args(shallow.then_some("--depth=1"))
            .args(shallow.then_some("--shallow-submodules"))
            .arg(from)
            .args(to)
            .exec()
            .map(drop)
    }

    pub fn fetch(
        self,
        shallow: bool,
        remote: impl AsRef<OsStr>,
        branch: Option<impl AsRef<OsStr>>,
    ) -> Result<()> {
        self.cmd()
            .stderr(Stdio::inherit())
            .arg("fetch")
            .args(shallow.then_some("--no-tags"))
            .args(shallow.then_some("--depth=1"))
            .arg(remote)
            .args(branch)
            .exec()
            .map(drop)
    }

    #[inline]
    pub fn root(self, root: &Path) -> Git<'_> {
        Git { root, ..self }
    }

    #[inline]
    pub fn quiet(self, quiet: bool) -> Self {
        Self { quiet, ..self }
    }

    /// True to perform shallow clones
    #[inline]
    pub fn shallow(self, shallow: bool) -> Self {
        Self { shallow, ..self }
    }

    pub fn checkout(self, recursive: bool, tag: impl AsRef<OsStr>) -> Result<()> {
        self.cmd()
            .arg("checkout")
            .args(recursive.then_some("--recurse-submodules"))
            .arg(tag)
            .exec()
            .map(drop)
    }

    pub fn init(self) -> Result<()> {
        self.cmd().arg("init").exec().map(drop)
    }

    #[allow(clippy::should_implement_trait)] // this is not std::ops::Add clippy
    pub fn add<I, S>(self, paths: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.cmd().arg("add").args(paths).exec().map(drop)
    }

    pub fn reset(self, hard: bool, tree: impl AsRef<OsStr>) -> Result<()> {
        self.cmd().arg("reset").args(hard.then_some("--hard")).arg(tree).exec().map(drop)
    }

    pub fn commit_tree(
        self,
        tree: impl AsRef<OsStr>,
        msg: Option<impl AsRef<OsStr>>,
    ) -> Result<String> {
        self.cmd()
            .arg("commit-tree")
            .arg(tree)
            .args(msg.as_ref().is_some().then_some("-m"))
            .args(msg)
            .get_stdout_lossy()
    }

    pub fn rm<I, S>(self, force: bool, paths: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.cmd().arg("rm").args(force.then_some("--force")).args(paths).exec().map(drop)
    }

    pub fn commit(self, msg: &str) -> Result<()> {
        let output = self
            .cmd()
            .args(["commit", "-m", msg])
            .args(cfg!(any(test, debug_assertions)).then_some("--no-gpg-sign"))
            .output()?;
        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            // ignore "nothing to commit" error
            let msg = "nothing to commit, working tree clean";
            if !(stdout.contains(msg) || stderr.contains(msg)) {
                return Err(eyre::eyre!(
                    "failed to commit (code={:?}, stdout={:?}, stderr={:?})",
                    output.status.code(),
                    stdout.trim(),
                    stderr.trim()
                ));
            }
        }
        Ok(())
    }

    pub fn is_in_repo(self) -> std::io::Result<bool> {
        self.cmd().args(["rev-parse", "--is-inside-work-tree"]).status().map(|s| s.success())
    }

    pub fn is_repo_root(self) -> Result<bool> {
        self.cmd().args(["rev-parse", "--show-cdup"]).exec().map(|out| out.stdout.is_empty())
    }

    pub fn is_clean(self) -> Result<bool> {
        self.cmd().args(["status", "--porcelain"]).exec().map(|out| out.stdout.is_empty())
    }

    pub fn has_branch(self, branch: impl AsRef<OsStr>, at: &Path) -> Result<bool> {
        self.cmd_at(at)
            .args(["branch", "--list", "--no-color"])
            .arg(branch)
            .get_stdout_lossy()
            .map(|stdout| !stdout.is_empty())
    }

    pub fn ensure_clean(self) -> Result<()> {
        if self.is_clean()? {
            Ok(())
        } else {
            Err(eyre::eyre!(
                "\
The target directory is a part of or on its own an already initialized git repository,
and it requires clean working and staging areas, including no untracked files.

Check the current git repository's status with `git status`.
Then, you can track files with `git add ...` and then commit them with `git commit`,
ignore them in the `.gitignore` file, or run this command again with the `--no-commit` flag."
            ))
        }
    }

    pub fn commit_hash(self, short: bool, revision: &str) -> Result<String> {
        self.cmd()
            .arg("rev-parse")
            .args(short.then_some("--short"))
            .arg(revision)
            .get_stdout_lossy()
    }

    pub fn tag(self) -> Result<String> {
        self.cmd().arg("tag").get_stdout_lossy()
    }

    pub fn has_missing_dependencies<I, S>(self, paths: I) -> Result<bool>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.cmd()
            .args(["submodule", "status"])
            .args(paths)
            .get_stdout_lossy()
            .map(|stdout| stdout.lines().any(|line| line.starts_with('-')))
    }

    /// Returns true if the given path has no submodules by checking `git submodule status`
    pub fn has_submodules<I, S>(self, paths: I) -> Result<bool>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.cmd()
            .args(["submodule", "status"])
            .args(paths)
            .get_stdout_lossy()
            .map(|stdout| stdout.trim().lines().next().is_some())
    }

    pub fn submodule_add(
        self,
        force: bool,
        url: impl AsRef<OsStr>,
        path: impl AsRef<OsStr>,
    ) -> Result<()> {
        self.cmd()
            .stderr(self.stderr())
            .args(["submodule", "add"])
            .args(self.shallow.then_some("--depth=1"))
            .args(force.then_some("--force"))
            .arg(url)
            .arg(path)
            .exec()
            .map(drop)
    }

    pub fn submodule_update<I, S>(
        self,
        force: bool,
        remote: bool,
        no_fetch: bool,
        recursive: bool,
        paths: I,
    ) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.cmd()
            .stderr(self.stderr())
            .args(["submodule", "update", "--progress", "--init"])
            .args(self.shallow.then_some("--depth=1"))
            .args(force.then_some("--force"))
            .args(remote.then_some("--remote"))
            .args(no_fetch.then_some("--no-fetch"))
            .args(recursive.then_some("--recursive"))
            .args(paths)
            .exec()
            .map(drop)
    }

    pub fn submodule_foreach(self, recursive: bool, cmd: impl AsRef<OsStr>) -> Result<()> {
        self.cmd()
            .stderr(self.stderr())
            .args(["submodule", "foreach"])
            .args(recursive.then_some("--recursive"))
            .arg(cmd)
            .exec()
            .map(drop)
    }

    pub fn submodule_init(self) -> Result<()> {
        self.cmd().stderr(self.stderr()).args(["submodule", "init"]).exec().map(drop)
    }

    pub fn cmd(self) -> Command {
        let mut cmd = Self::cmd_no_root();
        cmd.current_dir(self.root);
        cmd
    }

    pub fn cmd_at(self, path: &Path) -> Command {
        let mut cmd = Self::cmd_no_root();
        cmd.current_dir(path);
        cmd
    }

    pub fn cmd_no_root() -> Command {
        let mut cmd = Command::new("git");
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        cmd
    }

    // don't set this in cmd() because it's not wanted for all commands
    fn stderr(self) -> Stdio {
        if self.quiet {
            Stdio::piped()
        } else {
            Stdio::inherit()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundry_common::fs;
    use std::{env, fs::File, io::Write};
    use tempfile::tempdir;

    #[test]
    fn foundry_path_ext_works() {
        let p = Path::new("contracts/MyTest.t.sol");
        assert!(p.is_sol_test());
        assert!(p.is_sol());
        let p = Path::new("contracts/Greeter.sol");
        assert!(!p.is_sol_test());
    }

    // loads .env from cwd and project dir, See [`find_project_root()`]
    #[test]
    fn can_load_dotenv() {
        let temp = tempdir().unwrap();
        Git::new(temp.path()).init().unwrap();
        let cwd_env = temp.path().join(".env");
        fs::create_file(temp.path().join("foundry.toml")).unwrap();
        let nested = temp.path().join("nested");
        fs::create_dir(&nested).unwrap();

        let mut cwd_file = File::create(cwd_env).unwrap();
        let mut prj_file = File::create(nested.join(".env")).unwrap();

        cwd_file.write_all("TESTCWDKEY=cwd_val".as_bytes()).unwrap();
        cwd_file.sync_all().unwrap();

        prj_file.write_all("TESTPRJKEY=prj_val".as_bytes()).unwrap();
        prj_file.sync_all().unwrap();

        let cwd = env::current_dir().unwrap();
        env::set_current_dir(nested).unwrap();
        load_dotenv();
        env::set_current_dir(cwd).unwrap();

        assert_eq!(env::var("TESTCWDKEY").unwrap(), "cwd_val");
        assert_eq!(env::var("TESTPRJKEY").unwrap(), "prj_val");
    }
}
