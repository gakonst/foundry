//! Commonly used traits.

use alloy_json_abi::Function;
use alloy_primitives::Bytes;
use alloy_sol_types::SolError;
use std::{fmt, path::Path};

/// Test filter.
pub trait TestFilter: Send + Sync {
    /// Returns whether the test should be included.
    fn matches_test(&self, test_name: &str) -> bool;

    /// Returns whether the contract should be included.
    fn matches_contract(&self, contract_name: &str) -> bool;

    /// Returns a contract with the given path should be included.
    fn matches_path(&self, path: &Path) -> bool;
}

/// Extension trait for `Function`.
pub trait TestFunctionExt {
    /// Returns the kind of test function.
    fn test_function_kind(&self) -> TestFunctionKind {
        TestFunctionKind::classify(self.tfe_as_str(), self.tfe_has_inputs())
    }

    /// Returns `true` if this function is a `setUp` function.
    fn is_setup(&self) -> bool {
        self.test_function_kind().is_setup()
    }

    /// Returns `true` if this function is a unit, fuzz, or invariant test.
    fn is_any_test(&self) -> bool {
        self.test_function_kind().is_any_test()
    }

    /// Returns `true` if this function is a test that should fail.
    fn is_any_test_fail(&self) -> bool {
        self.test_function_kind().is_any_test_fail()
    }

    /// Returns `true` if this function is a unit test.
    fn is_unit_test(&self) -> bool {
        matches!(self.test_function_kind(), TestFunctionKind::UnitTest { .. })
    }

    /// Returns `true` if this function is a `beforeTestSetup` function.
    fn is_before_test_setup(&self) -> bool {
        self.tfe_as_str().eq_ignore_ascii_case("beforetestsetup")
    }

    /// Returns `true` if this function is a fuzz test.
    fn is_fuzz_test(&self) -> bool {
        self.test_function_kind().is_fuzz_test()
    }

    /// Returns `true` if this function is an invariant test.
    fn is_invariant_test(&self) -> bool {
        self.test_function_kind().is_invariant_test()
    }

    /// Returns `true` if this function is an `afterInvariant` function.
    fn is_after_invariant(&self) -> bool {
        self.test_function_kind().is_after_invariant()
    }

    /// Returns `true` if this function is a `fixture` function.
    fn is_fixture(&self) -> bool {
        self.test_function_kind().is_fixture()
    }

    #[doc(hidden)]
    fn tfe_as_str(&self) -> &str;
    #[doc(hidden)]
    fn tfe_has_inputs(&self) -> bool;
}

impl TestFunctionExt for Function {
    fn tfe_as_str(&self) -> &str {
        self.name.as_str()
    }

    fn tfe_has_inputs(&self) -> bool {
        !self.inputs.is_empty()
    }
}

impl TestFunctionExt for String {
    fn tfe_as_str(&self) -> &str {
        self
    }

    fn tfe_has_inputs(&self) -> bool {
        false
    }
}

impl TestFunctionExt for str {
    fn tfe_as_str(&self) -> &str {
        self
    }

    fn tfe_has_inputs(&self) -> bool {
        false
    }
}

/// Test function kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TestFunctionKind {
    /// `setUp`.
    Setup,
    /// `test*`. `should_fail` is `true` for `testFail*`.
    UnitTest { should_fail: bool },
    /// `test*`, with arguments. `should_fail` is `true` for `testFail*`.
    FuzzTest { should_fail: bool },
    /// `invariant*` or `statefulFuzz*`.
    InvariantTest,
    /// `afterInvariant`.
    AfterInvariant,
    /// `fixture*`.
    Fixture,
    /// Unknown kind.
    Unknown,
}

impl TestFunctionKind {
    /// Classify a function.
    #[inline]
    pub fn classify(name: &str, has_inputs: bool) -> Self {
        match () {
            _ if name.starts_with("test") => {
                let should_fail = name.starts_with("testFail");
                if has_inputs {
                    Self::FuzzTest { should_fail }
                } else {
                    Self::UnitTest { should_fail }
                }
            }
            _ if name.starts_with("invariant") || name.starts_with("statefulFuzz") => {
                Self::InvariantTest
            }
            _ if name.eq_ignore_ascii_case("setup") => Self::Setup,
            _ if name.eq_ignore_ascii_case("afterinvariant") => Self::AfterInvariant,
            _ if name.starts_with("fixture") => Self::Fixture,
            _ => Self::Unknown,
        }
    }

    /// Returns the name of the function kind.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Setup => "setUp",
            Self::UnitTest { should_fail: false } => "test",
            Self::UnitTest { should_fail: true } => "testFail",
            Self::FuzzTest { should_fail: false } => "fuzz",
            Self::FuzzTest { should_fail: true } => "fuzz fail",
            Self::InvariantTest => "invariant",
            Self::AfterInvariant => "afterInvariant",
            Self::Fixture => "fixture",
            Self::Unknown => "unknown",
        }
    }

    /// Returns `true` if this function is a `setUp` function.
    #[inline]
    pub const fn is_setup(&self) -> bool {
        matches!(self, Self::Setup)
    }

    /// Returns `true` if this function is a unit, fuzz, or invariant test.
    #[inline]
    pub const fn is_any_test(&self) -> bool {
        matches!(self, Self::UnitTest { .. } | Self::FuzzTest { .. } | Self::InvariantTest)
    }

    /// Returns `true` if this function is a test that should fail.
    #[inline]
    pub const fn is_any_test_fail(&self) -> bool {
        matches!(self, Self::UnitTest { should_fail: true } | Self::FuzzTest { should_fail: true })
    }

    /// Returns `true` if this function is a unit test.
    #[inline]
    pub fn is_unit_test(&self) -> bool {
        matches!(self, Self::UnitTest { .. })
    }

    /// Returns `true` if this function is a fuzz test.
    #[inline]
    pub const fn is_fuzz_test(&self) -> bool {
        matches!(self, Self::FuzzTest { .. })
    }

    /// Returns `true` if this function is an invariant test.
    #[inline]
    pub const fn is_invariant_test(&self) -> bool {
        matches!(self, Self::InvariantTest)
    }

    /// Returns `true` if this function is an `afterInvariant` function.
    #[inline]
    pub const fn is_after_invariant(&self) -> bool {
        matches!(self, Self::AfterInvariant)
    }

    /// Returns `true` if this function is a `fixture` function.
    #[inline]
    pub const fn is_fixture(&self) -> bool {
        matches!(self, Self::Fixture)
    }

    /// Returns `true` if this function kind is known.
    #[inline]
    pub const fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown)
    }

    /// Returns `true` if this function kind is unknown.
    #[inline]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl fmt::Display for TestFunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name().fmt(f)
    }
}

/// An extension trait for `std::error::Error` for ABI encoding.
pub trait ErrorExt: std::error::Error {
    /// ABI-encodes the error using `Revert(string)`.
    fn abi_encode_revert(&self) -> Bytes;
}

impl<T: std::error::Error> ErrorExt for T {
    fn abi_encode_revert(&self) -> Bytes {
        alloy_sol_types::Revert::from(self.to_string()).abi_encode().into()
    }
}

/// Extension trait for matching functions
pub trait FunctionFilter {
    /// Returns whether the function should be included
    fn matches_function(&self, function_name: impl AsRef<str>) -> bool;
}

/// Extension trait for matching functions
pub trait ContractFilter {
    /// Returns whether the contract should be included
    fn matches_contract(&self, contract_name: impl AsRef<str>) -> bool;
    /// Returns a contract with the given path should be included
    fn matches_path(&self, path: &Path) -> bool;
}