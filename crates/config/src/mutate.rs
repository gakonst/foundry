//! Configuration specific to the `forge mutate` command

use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};
use crate::{RegexWrapper, from_opt_glob, FuzzConfig};
use std::time::Duration;

/// Contains the mutation test config
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutateConfig {
    /// path to where mutation artifacts should be written to
    #[serde(default = "default_out_path")]
    pub out: PathBuf,

    /// Flag to write out mutants
    #[serde(default = "default_export")]
    pub export: bool,

    /// Only run mutations for functions matching the specified regex pattern.
    #[serde(rename = "match_function")]
    pub function_pattern: Option<RegexWrapper>,

    /// Only run mutations on functions that do not match the specified regex pattern.
    #[serde(rename = "no_match_function")]
    pub function_pattern_inverse: Option<RegexWrapper>,

    /// Only run mutations on functions in contracts matching the specified regex pattern.
    #[serde(rename = "match_contract")]
    pub contract_pattern: Option<RegexWrapper>,

    /// Only run mutations in contracts that do not match the specified regex pattern.
    #[serde(rename = "no_match_contract")]
    pub contract_pattern_inverse: Option<RegexWrapper>,

    /// Only run mutations on source files matching the specified glob pattern.
    #[serde(rename = "match_path", with = "from_opt_glob")]
    pub path_pattern: Option<globset::Glob>,

    /// Only run mutations on source files that do not match the specified glob pattern.
    #[serde(rename = "no_match_path", with = "from_opt_glob")]
    pub path_pattern_inverse: Option<globset::Glob>,

    /// Only run test functions matching the specified regex pattern.
    #[serde(rename = "match_test")]
    pub test_pattern: Option<RegexWrapper>,

    /// Only run test functions that do not match the specified regex pattern.
    #[serde(rename = "no_match_test")]
    pub test_pattern_inverse: Option<RegexWrapper>,

    /// Only run tests in contracts matching the specified regex pattern.
    #[serde(rename = "match_test_contract")]
    pub test_contract_pattern: Option<RegexWrapper>,

    /// Only run tests in contracts that do not match the specified regex pattern.
    #[serde(rename = "no_match_test_contract")]
    pub test_contract_pattern_inverse: Option<RegexWrapper>,

    /// Only run tests in source files matching the specified glob pattern.
    #[serde(rename = "match_test_path", with = "from_opt_glob")]
    pub test_path_pattern: Option<globset::Glob>,

    /// Only run tests in source files that do not match the specified glob pattern.
    #[serde(rename = "no_match_test_path", with = "from_opt_glob")]
    pub test_path_pattern_inverse: Option<globset::Glob>,

    /// Timeout for tests it helps exit long running test
    #[serde(default = "default_test_timeout")]
    pub test_timeout: Duration,

    /// Fuzz configuration
    pub fuzz: FuzzConfig,

    /// Number of mutants to execute concurrently
    ///
    /// This should be configured conservatively because of "Too Many Files Open Error" as
    /// we use join_all to run tasks in concurrently
    #[serde(default = "default_parallel")]
    pub parallel: usize,

    /// Max Timeout
    ///
    /// Maximum number of tests allowed to timeout, this is required because a test run
    /// can be long depending on the mutation. This leads to memory consumption per each
    /// mutant test that runs for a long time. 
    /// We configure this value here to put a bound on the possible memory leak for this
    /// 
    /// This is required because we can't cancel a thread
    /// 
    /// 16 * 32 MB (EVM default memory limit) = 512 MB
    #[serde(default = "default_maximum_timeout_test")]
    pub maximum_timeout_test: usize,
}


impl Default for MutateConfig {
    fn default() -> Self {
        Self {
            out: default_out_path(),
            export: default_export(),
            function_pattern: None,
            function_pattern_inverse: None,
            contract_pattern: None,
            contract_pattern_inverse: None,
            path_pattern: None,
            path_pattern_inverse: None,
            test_pattern: None,
            test_pattern_inverse: None,
            test_contract_pattern: None,
            test_contract_pattern_inverse: None,
            test_path_pattern: None,
            test_path_pattern_inverse: None,
            test_timeout: default_test_timeout(),
            fuzz: FuzzConfig {
                runs: 1,
                ..Default::default()
            },
            parallel: default_parallel(),
            maximum_timeout_test: default_maximum_timeout_test(),
        }
    }
}

fn default_out_path() -> PathBuf {
    PathBuf::from_str(".gambit").unwrap()
}

fn default_export() -> bool {
    false
}

fn default_parallel() -> usize {
    16
}

fn default_maximum_timeout_test() -> usize {
    16
}

fn default_test_timeout() -> Duration {
    Duration::from_millis(100)
}