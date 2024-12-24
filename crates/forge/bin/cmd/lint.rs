use clap::ValueEnum;
use clap::{Parser, ValueHint};
use eyre::Result;
use forge_lint::{Input, Linter};
use foundry_cli::utils::{FoundryPathExt, LoadConfig};
use foundry_compilers::{compilers::solc::SolcLanguage, solc::SOLC_EXTENSIONS};
use foundry_config::{filter::expand_globs, impl_figment_convert_basic};
use std::{
    io,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Json,
    Markdown,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Severity {
    High,
    Med,
    Low,
    Info,
    Gas,
}

/// CLI arguments for `forge fmt`.
#[derive(Clone, Debug, Parser)]
pub struct LintArgs {
    /// The project's root path.
    ///
    /// By default root of the Git repository, if in one,
    /// or the current working directory.
    #[arg(long, value_hint = ValueHint::DirPath, value_name = "PATH")]
    root: Option<PathBuf>,

    /// Include only the specified files.
    #[arg(long, value_hint = ValueHint::FilePath, value_name = "FILES", num_args(1..))]
    include: Option<Vec<PathBuf>>,

    /// Exclude the specified files.
    #[arg(long, value_hint = ValueHint::FilePath, value_name = "FILES", num_args(1..))]
    exclude: Option<Vec<PathBuf>>,

    /// Format of the output.
    ///
    /// Supported values: `json` or `markdown`.
    #[arg(long, value_name = "FORMAT", default_value = "json")]
    format: OutputFormat,

    /// Use only selected severities for output.
    ///
    /// Supported values: `high`, `med`, `low`, `info`, `gas`.
    #[arg(long, value_name = "SEVERITY", num_args(1..))]
    severity: Option<Vec<Severity>>,

    /// Show descriptions in the output.
    ///
    /// Disabled by default to avoid long console output.
    #[arg(long)]
    with_description: bool,
}

impl_figment_convert_basic!(LintArgs);

impl LintArgs {
    pub fn run(self) -> Result<()> {
        let config = self.try_load_config_emit_warnings()?;

        // Expand ignore globs and canonicalize from the get go
        let ignored = expand_globs(&config.root, config.fmt.ignore.iter())?
            .iter()
            .flat_map(foundry_common::fs::canonicalize_path)
            .collect::<Vec<_>>();

        let cwd = std::env::current_dir()?;

        // let input = match &self.paths[..] {
        //     [] => {
        //         // Retrieve the project paths, and filter out the ignored ones.
        //         let project_paths: Vec<PathBuf> = config
        //             .project_paths::<SolcLanguage>()
        //             .input_files_iter()
        //             .filter(|p| !(ignored.contains(p) || ignored.contains(&cwd.join(p))))
        //             .collect();
        //         Input::Paths(project_paths)
        //     }
        //     [one] if one == Path::new("-") => {
        //         let mut s = String::new();
        //         io::stdin().read_to_string(&mut s).expect("Failed to read from stdin");
        //         Input::Stdin(s)
        //     }
        //     paths => {
        //         let mut inputs = Vec::with_capacity(paths.len());
        //         for path in paths {
        //             if !ignored.is_empty()
        //                 && ((path.is_absolute() && ignored.contains(path))
        //                     || ignored.contains(&cwd.join(path)))
        //             {
        //                 continue;
        //             }

        //             if path.is_dir() {
        //                 inputs.extend(foundry_compilers::utils::source_files_iter(
        //                     path,
        //                     SOLC_EXTENSIONS,
        //                 ));
        //             } else if path.is_sol() {
        //                 inputs.push(path.to_path_buf());
        //             } else {
        //                 warn!("Cannot process path {}", path.display());
        //             }
        //         }
        //         Input::Paths(inputs)
        //     }
        // };

        // Linter::new(input).lint();

        Ok(())
    }
}
