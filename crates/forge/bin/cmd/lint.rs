use clap::{Parser, ValueHint};
use eyre::Result;
use forge_lint::{
    linter::{Linter, Severity},
    sol::SolidityLinter,
};
use foundry_cli::utils::LoadConfig;
use foundry_config::impl_figment_convert_basic;
use std::{collections::HashSet, path::PathBuf};

/// CLI arguments for `forge lint`.
#[derive(Clone, Debug, Parser)]
pub struct LintArgs {
    /// The project's root path.
    ///
    /// By default root of the Git repository, if in one,
    /// or the current working directory.
    #[arg(long, value_hint = ValueHint::DirPath, value_name = "PATH")]
    root: Option<PathBuf>,

    /// Include only the specified files when linting.
    #[arg(long, value_hint = ValueHint::FilePath, value_name = "FILES", num_args(1..))]
    include: Option<Vec<PathBuf>>,

    /// Exclude the specified files when linting.
    #[arg(long, value_hint = ValueHint::FilePath, value_name = "FILES", num_args(1..))]
    exclude: Option<Vec<PathBuf>>,

    /// Specifies which lints to run based on severity.
    ///
    /// Supported values: `high`, `med`, `low`, `info`, `gas`.
    #[arg(long, value_name = "SEVERITY", num_args(1..))]
    severity: Option<Vec<Severity>>,
}

impl_figment_convert_basic!(LintArgs);

impl LintArgs {
    pub fn run(self) -> Result<()> {
        let config = self.try_load_config_emit_warnings()?;
        let project = config.project()?;

        // Get all source files from the project
        let mut sources =
            project.paths.read_input_files()?.keys().cloned().collect::<Vec<PathBuf>>();

        // Add included paths to sources
        if let Some(include_paths) = &self.include {
            let included =
                include_paths.iter().filter(|path| path.exists()).cloned().collect::<Vec<_>>();
            sources.extend(included);
        }

        // Remove excluded files from sources
        if let Some(exclude_paths) = &self.exclude {
            let excluded = exclude_paths.iter().cloned().collect::<HashSet<_>>();
            sources.retain(|path| !excluded.contains(path));
        }

        if sources.is_empty() {
            sh_println!("Nothing to lint")?;
            std::process::exit(0);
        }

        if project.compiler.solc.is_some() {
            SolidityLinter::new().with_severity(self.severity).lint(&sources)?;
        } else {
            todo!("Linting not supported for this language");
        };

        Ok(())
    }
}
