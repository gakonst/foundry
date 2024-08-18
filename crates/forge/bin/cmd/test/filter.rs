use clap::Parser;
use foundry_common::TestFilter;
use foundry_compilers::{FileFilter, ProjectPathsConfig};
use foundry_config::{filter::GlobMatcher, Config};
use std::{fmt, path::Path};

/// The filter to use during testing.
///
/// See also `FileFilter`.
#[derive(Clone, Parser)]
#[command(next_help_heading = "Test filtering")]
pub struct FilterArgs {
    /// Only run test functions matching the specified regex pattern.
    #[arg(long = "match-test", visible_alias = "mt", value_name = "REGEX")]
    pub test_pattern: Option<regex::Regex>,

    /// Only run test functions that do not match the specified regex pattern.
    #[arg(long = "no-match-test", visible_alias = "nmt", value_name = "REGEX")]
    pub test_pattern_inverse: Option<regex::Regex>,

    /// Only run tests in contracts matching the specified regex pattern.
    #[arg(long = "match-contract", visible_alias = "mc", value_name = "REGEX")]
    pub contract_pattern: Option<regex::Regex>,

    /// Only run tests in contracts that do not match the specified regex pattern.
    #[arg(long = "no-match-contract", visible_alias = "nmc", value_name = "REGEX")]
    pub contract_pattern_inverse: Option<regex::Regex>,

    /// Only run tests in source files matching the specified glob pattern.
    #[arg(long = "match-path", visible_alias = "mp", value_name = "GLOB")]
    pub path_pattern: Option<GlobMatcher>,

    /// Only run tests in source files that do not match the specified glob pattern.
    #[arg(
        id = "no-match-path",
        long = "no-match-path",
        visible_alias = "nmp",
        value_name = "GLOB"
    )]
    pub path_pattern_inverse: Option<GlobMatcher>,

    /// Only show coverage for files that do not match the specified regex pattern.
    #[arg(long = "no-match-coverage", visible_alias = "nmco", value_name = "REGEX")]
    pub coverage_pattern_inverse: Option<regex::Regex>,
}

impl FilterArgs {
    /// Returns true if the filter is empty.
    pub fn is_empty(&self) -> bool {
        self.test_pattern.is_none() &&
            self.test_pattern_inverse.is_none() &&
            self.contract_pattern.is_none() &&
            self.contract_pattern_inverse.is_none() &&
            self.path_pattern.is_none() &&
            self.path_pattern_inverse.is_none()
    }

    /// Merges the set filter globs with the config's values
    pub fn merge_with_config(mut self, config: &Config) -> ProjectPathsAwareFilter {
        if self.test_pattern.is_none() {
            self.test_pattern = config.test_pattern.clone().map(Into::into);
        }
        if self.test_pattern_inverse.is_none() {
            self.test_pattern_inverse = config.test_pattern_inverse.clone().map(Into::into);
        }
        if self.contract_pattern.is_none() {
            self.contract_pattern = config.contract_pattern.clone().map(Into::into);
        }
        if self.contract_pattern_inverse.is_none() {
            self.contract_pattern_inverse = config.contract_pattern_inverse.clone().map(Into::into);
        }
        if self.path_pattern.is_none() {
            self.path_pattern = config.path_pattern.clone().map(Into::into);
        }
        if self.path_pattern_inverse.is_none() {
            self.path_pattern_inverse = config.path_pattern_inverse.clone().map(Into::into);
        }
        if self.coverage_pattern_inverse.is_none() {
            self.coverage_pattern_inverse = config.coverage_pattern_inverse.clone().map(Into::into);
        }
        ProjectPathsAwareFilter { args_filter: self, paths: config.project_paths() }
    }

    /// Merges the set filter globs with mutate test config values
    pub fn merge_with_mutate_config(&self, config: &Config) -> ProjectPathsAwareFilter {
        let mut filter = self.clone();

        if filter.test_pattern.is_none() {
            filter.test_pattern = config.mutate.test_pattern.clone().map(|p| p.into());
        }

        if filter.test_pattern_inverse.is_none() {
            filter.test_pattern_inverse = config.mutate.test_contract_pattern_inverse.clone().map(|p| p.into());
        }

        if filter.contract_pattern.is_none() {
            filter.contract_pattern = config.mutate.test_contract_pattern.clone().map(|p| p.into());
        }

        if filter.contract_pattern_inverse.is_none() {
            filter.contract_pattern = config.mutate.test_contract_pattern_inverse.clone().map(|p| p.into());
        }

        if filter.path_pattern.is_none() {
            filter.path_pattern = config.mutate.test_path_pattern.clone().map(|p| p.into());
        }

        if filter.path_pattern_inverse.is_none() {
            filter.path_pattern = config.mutate.test_path_pattern_inverse.clone().map(|p| p.into());
        }
        
        ProjectPathsAwareFilter { args_filter: self.clone(), paths: config.project_paths() }
    }
}

impl fmt::Debug for FilterArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilterArgs")
            .field("match-test", &self.test_pattern.as_ref().map(|r| r.as_str()))
            .field("no-match-test", &self.test_pattern_inverse.as_ref().map(|r| r.as_str()))
            .field("match-contract", &self.contract_pattern.as_ref().map(|r| r.as_str()))
            .field("no-match-contract", &self.contract_pattern_inverse.as_ref().map(|r| r.as_str()))
            .field("match-path", &self.path_pattern.as_ref().map(|g| g.as_str()))
            .field("no-match-path", &self.path_pattern_inverse.as_ref().map(|g| g.as_str()))
            .field("no-match-coverage", &self.coverage_pattern_inverse.as_ref().map(|g| g.as_str()))
            .finish_non_exhaustive()
    }
}

impl FileFilter for FilterArgs {
    /// Returns true if the file regex pattern match the `file`
    ///
    /// If no file regex is set this returns true by default
    fn is_match(&self, file: &Path) -> bool {
        self.matches_path(file)
    }
}

impl TestFilter for FilterArgs {
    fn matches_test(&self, test_name: &str) -> bool {
        let mut ok = true;
        if let Some(re) = &self.test_pattern {
            ok = ok && re.is_match(test_name);
        }
        if let Some(re) = &self.test_pattern_inverse {
            ok = ok && !re.is_match(test_name);
        }
        ok
    }

    fn matches_contract(&self, contract_name: &str) -> bool {
        let mut ok = true;
        if let Some(re) = &self.contract_pattern {
            ok = ok && re.is_match(contract_name);
        }
        if let Some(re) = &self.contract_pattern_inverse {
            ok = ok && !re.is_match(contract_name);
        }
        ok
    }

    fn matches_path(&self, path: &Path) -> bool {
        let mut ok = true;
        if let Some(re) = &self.path_pattern {
            ok = ok && re.is_match(path);
        }
        if let Some(re) = &self.path_pattern_inverse {
            ok = ok && !re.is_match(path);
        }
        ok
    }
}

impl fmt::Display for FilterArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(p) = &self.test_pattern {
            writeln!(f, "\tmatch-test: `{}`", p.as_str())?;
        }
        if let Some(p) = &self.test_pattern_inverse {
            writeln!(f, "\tno-match-test: `{}`", p.as_str())?;
        }
        if let Some(p) = &self.contract_pattern {
            writeln!(f, "\tmatch-contract: `{}`", p.as_str())?;
        }
        if let Some(p) = &self.contract_pattern_inverse {
            writeln!(f, "\tno-match-contract: `{}`", p.as_str())?;
        }
        if let Some(p) = &self.path_pattern {
            writeln!(f, "\tmatch-path: `{}`", p.as_str())?;
        }
        if let Some(p) = &self.path_pattern_inverse {
            writeln!(f, "\tno-match-path: `{}`", p.as_str())?;
        }
        if let Some(p) = &self.coverage_pattern_inverse {
            writeln!(f, "\tno-match-coverage: `{}`", p.as_str())?;
        }
        Ok(())
    }
}

/// A filter that combines all command line arguments and the paths of the current projects
#[derive(Clone, Debug)]
pub struct ProjectPathsAwareFilter {
    args_filter: FilterArgs,
    paths: ProjectPathsConfig,
}

impl ProjectPathsAwareFilter {
    /// Returns true if the filter is empty.
    pub fn is_empty(&self) -> bool {
        self.args_filter.is_empty()
    }

    /// Returns the CLI arguments.
    pub fn args(&self) -> &FilterArgs {
        &self.args_filter
    }

    /// Returns the CLI arguments mutably.
    pub fn args_mut(&mut self) -> &mut FilterArgs {
        &mut self.args_filter
    }

    /// Returns the project paths.
    pub fn paths(&self) -> &ProjectPathsConfig {
        &self.paths
    }
}

impl FileFilter for ProjectPathsAwareFilter {
    /// Returns true if the file regex pattern match the `file`
    ///
    /// If no file regex is set this returns true by default
    fn is_match(&self, mut file: &Path) -> bool {
        file = file.strip_prefix(&self.paths.root).unwrap_or(file);
        self.args_filter.is_match(file)
    }
}

impl TestFilter for ProjectPathsAwareFilter {
    fn matches_test(&self, test_name: &str) -> bool {
        self.args_filter.matches_test(test_name)
    }

    fn matches_contract(&self, contract_name: &str) -> bool {
        self.args_filter.matches_contract(contract_name)
    }

    fn matches_path(&self, mut path: &Path) -> bool {
        // we don't want to test files that belong to a library
        path = path.strip_prefix(&self.paths.root).unwrap_or(path);
        self.args_filter.matches_path(path) && !self.paths.has_library_ancestor(path)
    }
}

impl fmt::Display for ProjectPathsAwareFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.args_filter.fmt(f)
    }
}
