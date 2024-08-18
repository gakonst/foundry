use crate::cmd::test::{FilterArgs, ProjectPathsAwareFilter};
use clap::Parser;
use foundry_cli::utils::FoundryPathExt;
use foundry_common::{
    glob::GlobMatcher,
    traits::{FunctionFilter, TestFunctionExt},
    ContractFilter,
};
use foundry_compilers::{FileFilter, ProjectPathsConfig};
use foundry_config::Config;
use std::{fmt, path::Path};

/// The filter to use during mutation testing.
///
/// See also `FileFilter`.
#[derive(Clone, Parser)]
#[clap(next_help_heading = "Mutation Test filtering")]
pub struct MutateFilterArgs {
    /// Only run mutations on functions matching the specified regex pattern.
    #[clap(long = "match-function", visible_alias = "mf", value_name = "REGEX")]
    pub function_pattern: Option<regex::Regex>,

    /// Only run mutations on functions that do not match the specified regex pattern.
    #[clap(long = "no-match-function", visible_alias = "nmf", value_name = "REGEX")]
    pub function_pattern_inverse: Option<regex::Regex>,

    /// Only run mutations on functions in contracts matching the specified regex pattern.
    #[clap(long = "match-contract", visible_alias = "mc", value_name = "REGEX")]
    pub contract_pattern: Option<regex::Regex>,

    /// Only run mutations in contracts that do not match the specified regex pattern.
    #[clap(long = "no-match-contract", visible_alias = "nmc", value_name = "REGEX")]
    pub contract_pattern_inverse: Option<regex::Regex>,

    /// Only run mutations on source files matching the specified glob pattern.
    #[clap(long = "match-path", visible_alias = "mp", value_name = "GLOB")]
    pub path_pattern: Option<GlobMatcher>,

    /// Only run mutations on source files that do not match the specified glob pattern.
    #[clap(
        name = "no-match-path",
        long = "no-match-path",
        visible_alias = "nmp",
        value_name = "GLOB"
    )]
    pub path_pattern_inverse: Option<GlobMatcher>,

    /// Only run test functions matching the specified regex pattern.
    #[clap(long = "match-test")]
    pub test_pattern: Option<regex::Regex>,

    /// Only run test functions that do not match the specified regex pattern.
    #[clap(long = "no-match-test")]
    pub test_pattern_inverse: Option<regex::Regex>,

    /// Only run tests in contracts matching the specified regex pattern.
    #[clap(long = "match-test-contract")]
    pub test_contract_pattern: Option<regex::Regex>,

    /// Only run tests in contracts that do not match the specified regex pattern.
    #[clap(long = "no-match-test-contract")]
    pub test_contract_pattern_inverse: Option<regex::Regex>,

    /// Only run tests in source files matching the specified glob pattern.
    #[clap(long = "match-test-path", value_name = "GLOB")]
    pub test_path_pattern: Option<GlobMatcher>,

    /// Only run tests in source files that do not match the specified glob pattern.
    #[clap(long = "no-match-test-path", value_name = "GLOB")]
    pub test_path_pattern_inverse: Option<GlobMatcher>,
}

impl MutateFilterArgs {
    /// Merges the set filter globs with the config's values
    /// Returns mutate and test filters
    pub fn merge_with_config(
        &self,
        config: &Config,
    ) -> (MutationProjectPathsAwareFilter, ProjectPathsAwareFilter) {
        let mut filter = self.clone();
        if filter.function_pattern.is_none() {
            filter.function_pattern = config.mutate.function_pattern.clone().map(|p| p.into());
        }
        if filter.function_pattern_inverse.is_none() {
            filter.function_pattern_inverse =
                config.mutate.function_pattern_inverse.clone().map(|p| p.into());
        }
        if filter.contract_pattern.is_none() {
            filter.contract_pattern = config.mutate.contract_pattern.clone().map(|p| p.into());
        }
        if filter.contract_pattern_inverse.is_none() {
            filter.contract_pattern_inverse =
                config.mutate.contract_pattern_inverse.clone().map(|p| p.into());
        }
        if filter.path_pattern.is_none() {
            filter.path_pattern = config.mutate.path_pattern.clone().map(Into::into);
        }
        if filter.path_pattern_inverse.is_none() {
            filter.path_pattern_inverse =
                config.mutate.path_pattern_inverse.clone().map(Into::into);
        }

        // Parse test filter
        let test_filter: FilterArgs = FilterArgs {
            test_pattern: filter.test_pattern.clone(),
            test_pattern_inverse: filter.test_pattern_inverse.clone(),
            contract_pattern: filter.test_contract_pattern.clone(),
            contract_pattern_inverse: filter.test_contract_pattern_inverse.clone(),
            path_pattern: filter.test_path_pattern.clone(),
            path_pattern_inverse: filter.test_path_pattern_inverse.clone(),
        };
        let test_paths_aware_filter = test_filter.merge_with_mutate_config(&config);

        (
            MutationProjectPathsAwareFilter { args_filter: filter, paths: config.project_paths() },
            test_paths_aware_filter,
        )
    }
}

impl fmt::Debug for MutateFilterArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MutateFilterArgs")
            .field("match-function", &self.function_pattern.as_ref().map(|r| r.as_str()))
            .field("no-match-function", &self.function_pattern_inverse.as_ref().map(|r| r.as_str()))
            .field("match-contract", &self.contract_pattern.as_ref().map(|r| r.as_str()))
            .field("no-match-contract", &self.contract_pattern_inverse.as_ref().map(|r| r.as_str()))
            .field("match-path", &self.path_pattern.as_ref().map(|g| g.as_str()))
            .field("no-match-path", &self.path_pattern_inverse.as_ref().map(|g| g.as_str()))
            .field("match-test", &self.test_pattern.as_ref().map(|r| r.as_str()))
            .field("no-match-test", &self.test_path_pattern_inverse.as_ref().map(|r| r.as_str()))
            .field("match-test-contract", &self.test_contract_pattern.as_ref().map(|r| r.as_str()))
            .field(
                "no-match-test-contract",
                &self.test_contract_pattern_inverse.as_ref().map(|r| r.as_str()),
            )
            .field("match-test-path", &self.test_path_pattern.as_ref().map(|g| g.as_str()))
            .field(
                "no-match-test-path",
                &self.test_path_pattern_inverse.as_ref().map(|g| g.as_str()),
            )
            .finish_non_exhaustive()
    }
}

impl FileFilter for MutateFilterArgs {
    /// Returns true if the file regex pattern match the `file`
    ///
    /// If no file regex is set this returns true if the file ends with `.t.sol`, see
    /// [FoundryPathExr::is_sol_test()]
    fn is_match(&self, file: &Path) -> bool {
        if let Some(file) = file.as_os_str().to_str() {
            if let Some(ref glob) = self.path_pattern {
                return glob.is_match(file);
            }
            if let Some(ref glob) = self.path_pattern_inverse {
                return !glob.is_match(file);
            }
        }
        file.is_sol_test()
    }
}

impl ContractFilter for MutateFilterArgs {
    fn matches_contract(&self, contract_name: impl AsRef<str>) -> bool {
        let mut ok = true;
        let contract_name = contract_name.as_ref();
        if let Some(re) = &self.contract_pattern {
            ok &= re.is_match(contract_name);
        }
        if let Some(re) = &self.contract_pattern_inverse {
            ok &= !re.is_match(contract_name);
        }
        ok
    }

    fn matches_path(&self, path: &Path) -> bool {
        let Some(path) = path.to_str() else {
            return false;
        };

        let mut ok = true;
        if let Some(ref glob) = self.path_pattern {
            ok &= glob.is_match(path);
        }
        if let Some(ref glob) = self.path_pattern_inverse {
            ok &= !glob.is_match(path);
        }
        ok
    }
}

impl FunctionFilter for MutateFilterArgs {
    fn matches_function(&self, function_name: impl AsRef<str>) -> bool {
        let mut ok = true;
        let function_name = function_name.as_ref();

        if let Some(re) = &self.function_pattern {
            ok &= re.is_match(function_name);
        }

        if let Some(re) = &self.function_pattern_inverse {
            ok &= !re.is_match(function_name);
        }

        ok &= !function_name.is_test();

        ok
    }
}

impl fmt::Display for MutateFilterArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut patterns = Vec::new();
        if let Some(ref p) = self.function_pattern {
            patterns.push(format!("\tmatch-function: `{}`", p.as_str()));
        }
        if let Some(ref p) = self.function_pattern_inverse {
            patterns.push(format!("\tno-match-function: `{}`", p.as_str()));
        }
        if let Some(ref p) = self.contract_pattern {
            patterns.push(format!("\tmatch-contract: `{}`", p.as_str()));
        }
        if let Some(ref p) = self.contract_pattern_inverse {
            patterns.push(format!("\tno-match-contract: `{}`", p.as_str()));
        }
        if let Some(ref p) = self.path_pattern {
            patterns.push(format!("\tmatch-path: `{}`", p.as_str()));
        }
        if let Some(ref p) = self.path_pattern_inverse {
            patterns.push(format!("\tno-match-path: `{}`", p.as_str()));
        }
        write!(f, "{}", patterns.join("\n"))
    }
}

/// A filter that combines all command line arguments and the paths of the current projects
#[derive(Debug, Clone)]
pub struct MutationProjectPathsAwareFilter {
    args_filter: MutateFilterArgs,
    paths: ProjectPathsConfig,
}

impl MutationProjectPathsAwareFilter {
    /// Returns the CLI arguments
    pub fn args(&self) -> &MutateFilterArgs {
        &self.args_filter
    }

    /// Returns the CLI arguments mutably
    pub fn args_mut(&mut self) -> &mut MutateFilterArgs {
        &mut self.args_filter
    }
}

impl FileFilter for MutationProjectPathsAwareFilter {
    /// Returns true if the file regex pattern match the `file`
    ///
    /// If no file regex is set this returns true if the file ends with `.t.sol`, see
    /// [FoundryPathExr::is_sol_test()]
    fn is_match(&self, file: &Path) -> bool {
        self.args_filter.is_match(file)
    }
}

impl ContractFilter for MutationProjectPathsAwareFilter {
    fn matches_contract(&self, contract_name: impl AsRef<str>) -> bool {
        self.args_filter.matches_contract(contract_name)
    }

    fn matches_path(&self, path: &Path) -> bool {
        // we don't want to test files that belong to a library
        self.args_filter.matches_path(path) &&
            !self.paths.has_library_ancestor(path) &&
            !Path::new(path).starts_with(&self.paths.tests)
    }
}

impl FunctionFilter for MutationProjectPathsAwareFilter {
    fn matches_function(&self, function_name: impl AsRef<str>) -> bool {
        self.args_filter.matches_function(function_name)
    }
}

impl fmt::Display for MutationProjectPathsAwareFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.args_filter.fmt(f)
    }
}
