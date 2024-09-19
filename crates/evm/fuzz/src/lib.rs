//! # foundry-evm-fuzz
//!
//! EVM fuzzing implementation using [`proptest`].

#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[macro_use]
extern crate tracing;

use alloy_dyn_abi::{DynSolValue, JsonAbiExt};
use alloy_primitives::{Address, Bytes, Log, U256};
use foundry_common::{calc, contracts::ContractsByAddress, evm::Breakpoints};
use foundry_evm_coverage::HitMaps;
use foundry_evm_traces::{CallTraceArena, SparsedTraceArena};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, sync::Arc};

pub use proptest::test_runner::{Config as FuzzConfig, Reason};

mod error;
pub use error::FuzzError;

pub mod invariant;
pub mod strategies;

mod inspector;
pub use inspector::Fuzzer;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CounterExample {
    /// Call used as a counter example for fuzz tests.
    Single(BaseCounterExample),
    /// Sequence of calls used as a counter example for invariant tests.
    Sequence(Vec<BaseCounterExample>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaseCounterExample {
    /// Address which makes the call
    pub sender: Option<Address>,
    /// Address to which to call to
    pub addr: Option<Address>,
    /// The data to provide
    pub calldata: Bytes,
    /// The number of wei sent
    pub value: Option<U256>,
    /// Contract name if it exists
    pub contract_name: Option<String>,
    /// Function signature if it exists
    pub signature: Option<String>,
    /// Args used to call the function
    pub args: Option<String>,
    /// Traces
    #[serde(skip)]
    pub traces: Option<SparsedTraceArena>,
}

impl BaseCounterExample {
    /// Creates counter example representing a step from invariant call sequence.
    pub fn from_invariant_call(
        sender: Address,
        addr: Address,
        bytes: &Bytes,
        value: U256,
        contracts: &ContractsByAddress,
        traces: Option<SparsedTraceArena>,
    ) -> Self {
        let value = if value == U256::ZERO { None } else { Some(value) };
        if let Some((name, abi)) = &contracts.get(&addr) {
            if let Some(func) = abi.functions().find(|f| f.selector() == bytes[..4]) {
                // skip the function selector when decoding
                if let Ok(args) = func.abi_decode_input(&bytes[4..], false) {
                    return Self {
                        sender: Some(sender),
                        addr: Some(addr),
                        calldata: bytes.clone(),
                        value,
                        contract_name: Some(name.clone()),
                        signature: Some(func.signature()),
                        args: Some(
                            foundry_common::fmt::format_tokens(&args).format(", ").to_string(),
                        ),
                        traces,
                    };
                }
            }
        }

        Self {
            sender: Some(sender),
            addr: Some(addr),
            calldata: bytes.clone(),
            value,
            contract_name: None,
            signature: None,
            args: None,
            traces,
        }
    }

    /// Creates counter example for a fuzz test failure.
    pub fn from_fuzz_call(
        bytes: Bytes,
        args: Vec<DynSolValue>,
        traces: Option<SparsedTraceArena>,
    ) -> Self {
        Self {
            sender: None,
            addr: None,
            calldata: bytes,
            value: None,
            contract_name: None,
            signature: None,
            args: Some(foundry_common::fmt::format_tokens(&args).format(", ").to_string()),
            traces,
        }
    }
}

impl fmt::Display for BaseCounterExample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(sender) = self.sender {
            write!(f, "sender={sender} addr=")?
        }

        if let Some(name) = &self.contract_name {
            write!(f, "[{name}]")?
        }

        if let Some(addr) = &self.addr {
            write!(f, "{addr} ")?
        }

        if let Some(sig) = &self.signature {
            write!(f, "calldata={sig}")?
        } else {
            write!(f, "calldata={}", &self.calldata)?
        }

        if let Some(args) = &self.args {
            write!(f, " args=[{args}]")?
        } else {
            write!(f, " args=[]")?
        }

        if let Some(value) = &self.value {
            write!(f, " value=[{value}]")
        } else {
            write!(f, "")
        }
    }
}

/// The outcome of a fuzz test
#[derive(Debug)]
pub struct FuzzTestResult {
    /// we keep this for the debugger
    pub first_case: FuzzCase,
    /// Gas usage (gas_used, call_stipend) per cases
    pub gas_by_case: Vec<(u64, u64)>,
    /// Whether the test case was successful. This means that the transaction executed
    /// properly, or that there was a revert and that the test was expected to fail
    /// (prefixed with `testFail`)
    pub success: bool,
    /// Whether the test case was skipped. `reason` will contain the skip reason, if any.
    pub skipped: bool,

    /// If there was a revert, this field will be populated. Note that the test can
    /// still be successful (i.e self.success == true) when it's expected to fail.
    pub reason: Option<String>,

    /// Minimal reproduction test case for failing fuzz tests
    pub counterexample: Option<CounterExample>,

    /// Any captured & parsed as strings logs along the test's execution which should
    /// be printed to the user.
    pub logs: Vec<Log>,

    /// Labeled addresses
    pub labeled_addresses: HashMap<Address, String>,

    /// Exemplary traces for a fuzz run of the test function
    ///
    /// **Note** We only store a single trace of a successful fuzz call, otherwise we would get
    /// `num(fuzz_cases)` traces, one for each run, which is neither helpful nor performant.
    pub traces: Option<SparsedTraceArena>,

    /// Additional traces used for gas report construction.
    /// Those traces should not be displayed.
    pub gas_report_traces: Vec<CallTraceArena>,

    /// Raw coverage info
    pub coverage: Option<HitMaps>,

    /// Breakpoints for debugger. Correspond to the same fuzz case as `traces`.
    pub breakpoints: Option<Breakpoints>,
}

impl FuzzTestResult {
    /// Returns the median gas of all test cases
    pub fn median_gas(&self, with_stipend: bool) -> u64 {
        let mut values = self.gas_values(with_stipend);
        values.sort_unstable();
        calc::median_sorted(&values)
    }

    /// Returns the average gas use of all test cases
    pub fn mean_gas(&self, with_stipend: bool) -> u64 {
        let mut values = self.gas_values(with_stipend);
        values.sort_unstable();
        calc::mean(&values)
    }

    fn gas_values(&self, with_stipend: bool) -> Vec<u64> {
        self.gas_by_case
            .iter()
            .map(|gas| if with_stipend { gas.0 } else { gas.0.saturating_sub(gas.1) })
            .collect()
    }
}

/// Data of a single fuzz test case
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FuzzCase {
    /// The calldata used for this fuzz test
    pub calldata: Bytes,
    /// Consumed gas
    pub gas: u64,
    /// The initial gas stipend for the transaction
    pub stipend: u64,
}

/// Container type for all successful test cases
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FuzzedCases {
    cases: Vec<FuzzCase>,
}

impl FuzzedCases {
    #[inline]
    pub fn new(mut cases: Vec<FuzzCase>) -> Self {
        cases.sort_by_key(|c| c.gas);
        Self { cases }
    }

    #[inline]
    pub fn cases(&self) -> &[FuzzCase] {
        &self.cases
    }

    #[inline]
    pub fn into_cases(self) -> Vec<FuzzCase> {
        self.cases
    }

    /// Get the last [FuzzCase]
    #[inline]
    pub fn last(&self) -> Option<&FuzzCase> {
        self.cases.last()
    }

    /// Returns the median gas of all test cases
    #[inline]
    pub fn median_gas(&self, with_stipend: bool) -> u64 {
        let mut values = self.gas_values(with_stipend);
        values.sort_unstable();
        calc::median_sorted(&values)
    }

    /// Returns the average gas use of all test cases
    #[inline]
    pub fn mean_gas(&self, with_stipend: bool) -> u64 {
        let mut values = self.gas_values(with_stipend);
        values.sort_unstable();
        calc::mean(&values)
    }

    #[inline]
    fn gas_values(&self, with_stipend: bool) -> Vec<u64> {
        self.cases
            .iter()
            .map(|c| if with_stipend { c.gas } else { c.gas.saturating_sub(c.stipend) })
            .collect()
    }

    /// Returns the case with the highest gas usage
    #[inline]
    pub fn highest(&self) -> Option<&FuzzCase> {
        self.cases.last()
    }

    /// Returns the case with the lowest gas usage
    #[inline]
    pub fn lowest(&self) -> Option<&FuzzCase> {
        self.cases.first()
    }

    /// Returns the highest amount of gas spent on a fuzz case
    #[inline]
    pub fn highest_gas(&self, with_stipend: bool) -> u64 {
        self.highest()
            .map(|c| if with_stipend { c.gas } else { c.gas - c.stipend })
            .unwrap_or_default()
    }

    /// Returns the lowest amount of gas spent on a fuzz case
    #[inline]
    pub fn lowest_gas(&self) -> u64 {
        self.lowest().map(|c| c.gas).unwrap_or_default()
    }
}

/// Fixtures to be used for fuzz tests.
///
/// The key represents name of the fuzzed parameter, value holds possible fuzzed values.
/// For example, for a fixture function declared as
/// `function fixture_sender() external returns (address[] memory senders)`
/// the fuzz fixtures will contain `sender` key with `senders` array as value
#[derive(Clone, Default, Debug)]
pub struct FuzzFixtures {
    inner: Arc<HashMap<String, DynSolValue>>,
}

impl FuzzFixtures {
    pub fn new(fixtures: HashMap<String, DynSolValue>) -> Self {
        Self { inner: Arc::new(fixtures) }
    }

    /// Returns configured fixtures for `param_name` fuzzed parameter.
    pub fn param_fixtures(&self, param_name: &str) -> Option<&[DynSolValue]> {
        if let Some(param_fixtures) = self.inner.get(&normalize_fixture(param_name)) {
            param_fixtures.as_fixed_array().or_else(|| param_fixtures.as_array())
        } else {
            None
        }
    }
}

/// Extracts fixture name from a function name.
/// For example: fixtures defined in `fixture_Owner` function will be applied for `owner` parameter.
pub fn fixture_name(function_name: String) -> String {
    normalize_fixture(function_name.strip_prefix("fixture").unwrap())
}

/// Normalize fixture parameter name, for example `_Owner` to `owner`.
fn normalize_fixture(param_name: &str) -> String {
    param_name.trim_matches('_').to_ascii_lowercase()
}
