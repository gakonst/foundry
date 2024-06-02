use crate::cmd::{
    bind::BindArgs, build::BuildArgs, cache::CacheArgs, clone::CloneArgs, config, coverage,
    create::CreateArgs, debug::DebugArgs, decode_error, doc::DocArgs, flatten, fmt::FmtArgs,
    geiger, generate, init::InitArgs, inspect, install::InstallArgs, remappings::RemappingArgs,
    remove::RemoveArgs, selectors::SelectorsSubcommands, snapshot, test, tree, update,
};
use clap::{Parser, Subcommand, ValueHint};
use forge_script::ScriptArgs;
use forge_verify::{bytecode::VerifyBytecodeArgs, VerifyArgs, VerifyCheckArgs};
use std::path::PathBuf;

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("VERGEN_GIT_SHA"),
    " ",
    env!("VERGEN_BUILD_TIMESTAMP"),
    ")"
);

/// Build, test, fuzz, debug and deploy Solidity contracts.
#[derive(Parser)]
#[command(
    name = "forge",
    version = VERSION_MESSAGE,
    after_help = "Find more information in the book: http://book.getfoundry.sh/reference/forge/forge.html",
    next_display_order = None,
)]
pub struct Forge {
    #[command(subcommand)]
    pub cmd: ForgeSubcommand,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ForgeSubcommand {
    /// Run the project's tests.
    #[command(visible_alias = "t")]
    Test(test::TestArgs),

    /// Run a smart contract as a script, building transactions that can be sent onchain.
    Script(ScriptArgs),

    /// Generate coverage reports.
    Coverage(coverage::CoverageArgs),

    /// Generate Rust bindings for smart contracts.
    #[command(alias = "bi")]
    Bind(BindArgs),

    /// Build the project's smart contracts.
    #[command(visible_aliases = ["b", "compile"])]
    Build(BuildArgs),

    /// Clone a contract from Etherscan.
    Clone(CloneArgs),

    /// Debugs a single smart contract as a script.
    #[command(visible_alias = "d")]
    Debug(DebugArgs),

    /// Update one or multiple dependencies.
    ///
    /// If no arguments are provided, then all dependencies are updated.
    #[command(visible_alias = "u")]
    Update(update::UpdateArgs),

    /// Install one or multiple dependencies.
    ///
    /// If no arguments are provided, then existing dependencies will be installed.
    #[command(visible_alias = "i")]
    Install(InstallArgs),

    /// Remove one or multiple dependencies.
    #[command(visible_alias = "rm")]
    Remove(RemoveArgs),

    /// Get the automatically inferred remappings for the project.
    #[command(visible_alias = "re")]
    Remappings(RemappingArgs),

    /// Verify smart contracts on Etherscan.
    #[command(visible_alias = "v")]
    VerifyContract(VerifyArgs),

    /// Check verification status on Etherscan.
    #[command(visible_alias = "vc")]
    VerifyCheck(VerifyCheckArgs),

    /// Deploy a smart contract.
    #[command(visible_alias = "c")]
    Create(CreateArgs),

    /// Create a new Forge project.
    Init(InitArgs),

    /// Generate shell completions script.
    #[command(visible_alias = "com")]
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// Generate Fig autocompletion spec.
    #[command(visible_alias = "fig")]
    GenerateFigSpec,

    /// Remove the build artifacts and cache directories.
    #[command(visible_alias = "cl")]
    Clean {
        /// The project's root path.
        ///
        /// By default root of the Git repository, if in one,
        /// or the current working directory.
        #[arg(long, value_hint = ValueHint::DirPath, value_name = "PATH")]
        root: Option<PathBuf>,
    },

    /// Manage the Foundry cache.
    Cache(CacheArgs),

    /// Create a snapshot of each test's gas usage.
    #[command(visible_alias = "s")]
    Snapshot(snapshot::SnapshotArgs),

    /// Display the current config.
    #[command(visible_alias = "co")]
    Config(config::ConfigArgs),

    /// Flatten a source file and all of its imports into one file.
    #[command(visible_alias = "f")]
    Flatten(flatten::FlattenArgs),

    /// Format Solidity source files.
    Fmt(FmtArgs),

    /// Get specialized information about a smart contract.
    #[command(visible_alias = "in")]
    Inspect(inspect::InspectArgs),

    /// Display a tree visualization of the project's dependency graph.
    #[command(visible_alias = "tr")]
    Tree(tree::TreeArgs),

    /// Detects usage of unsafe cheat codes in a project and its dependencies.
    Geiger(geiger::GeigerArgs),

    /// Generate documentation for the project.
    Doc(DocArgs),

    /// Function selector utilities
    #[command(visible_alias = "se")]
    Selectors {
        #[command(subcommand)]
        command: SelectorsSubcommands,
    },

    /// Generate scaffold files.
    Generate(generate::GenerateArgs),

    /// Verify the deployed bytecode against its source.
    #[clap(visible_alias = "vb")]
    VerifyBytecode(VerifyBytecodeArgs),

    /// Attempt to decode raw bytes from a revert message
    DecodeError(decode_error::DecodeError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Forge::command().debug_assert();
    }
}
