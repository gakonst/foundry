use super::state::EvmFuzzState;
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{Address, B256, I256, U256};
use proptest::prelude::*;
use std::collections::HashMap;

/// The max length of arrays we fuzz for is 256.
const MAX_ARRAY_LEN: usize = 256;

/// Struct to hold range configuration
#[derive(Default, Clone)]
pub struct FuzzConfig {
    ranges: HashMap<String, (U256, U256)>,
}

impl FuzzConfig {
    /// Initiates a  new range configuration
    pub fn new() -> Self {
        Self { ranges: HashMap::new() }
    }

    /// Adds a range
    pub fn with_range(mut self, param_name: &str, min: U256, max: U256) -> Self {
        self.ranges.insert(param_name.to_string(), (min, max));
        self
    }
}

/// Given a parameter type, returns a strategy for generating values for that type.
///
/// See [`fuzz_param_with_fixtures`] for more information.
pub fn fuzz_param(param: &DynSolType, config: &FuzzConfig) -> BoxedStrategy<DynSolValue> {
    fuzz_param_inner(param, config, None)
}

/// Given a parameter type and configured fixtures for param name, returns a strategy for generating
/// values for that type.
///
/// Fixtures can be currently generated for uint, int, address, bytes and
/// string types and are defined for parameter name.
/// For example, fixtures for parameter `owner` of type `address` can be defined in a function with
/// a `function fixture_owner() public returns (address[] memory)` signature.
///
/// Fixtures are matched on parameter name, hence fixtures defined in
/// `fixture_owner` function can be used in a fuzzed test function with a signature like
/// `function testFuzz_ownerAddress(address owner, uint amount)`.
///
/// Raises an error if all the fixture types are not of the same type as the input parameter.
///
/// Works with ABI Encoder v2 tuples.
pub fn fuzz_param_with_fixtures(
    param: &DynSolType,
    fixtures: Option<&[DynSolValue]>,
    name: &str,
) -> BoxedStrategy<DynSolValue> {
    fuzz_param_inner(param, &FuzzConfig::new(), fixtures.map(|f| (f, name)))
}

fn fuzz_param_inner(
    param: &DynSolType,
    config: &FuzzConfig,
    mut fuzz_fixtures: Option<(&[DynSolValue], &str)>,
) -> BoxedStrategy<DynSolValue> {
    let param_name = fuzz_fixtures.as_ref().map(|(_, name)| *name);

    if let Some((fixtures, name)) = fuzz_fixtures {
        if !fixtures.iter().all(|f| f.matches(param)) {
            error!("fixtures for {name:?} do not match type {param}");
            fuzz_fixtures = None;
        }
    }
    let fuzz_fixtures = fuzz_fixtures.map(|(f, _)| f);

    let value = || {
        let default_strategy = DynSolValue::type_strategy(param);
        if let Some(fixtures) = fuzz_fixtures {
            proptest::prop_oneof![
                50 => {
                    let fixtures = fixtures.to_vec();
                    any::<prop::sample::Index>()
                        .prop_map(move |index| index.get(&fixtures).clone())
                },
                50 => default_strategy,
            ]
            .boxed()
        } else {
            default_strategy.boxed()
        }
    };

    match *param {
        DynSolType::Address => value(),
        DynSolType::Int(n @ 8..=256) => super::IntStrategy::new(n, fuzz_fixtures)
            .prop_map(move |x| DynSolValue::Int(x, n))
            .boxed(),
        DynSolType::Uint(n @ 8..=256) => {
            let bounds = param_name.and_then(|name| config.ranges.get(name));
            match bounds {
                Some((min, max)) => {
                    super::UintStrategy::new(n, fuzz_fixtures, Some(*min), Some(*max))
                        .prop_map(move |x| DynSolValue::Uint(x, n))
                        .boxed()
                }
                None => super::UintStrategy::new(n, fuzz_fixtures, None, None)
                    .prop_map(move |x| DynSolValue::Uint(x, n))
                    .boxed(),
            }
        }
        DynSolType::Function | DynSolType::Bool => DynSolValue::type_strategy(param).boxed(),
        DynSolType::Bytes => value(),
        DynSolType::FixedBytes(_size @ 1..=32) => value(),
        DynSolType::String => value()
            .prop_map(move |value| {
                DynSolValue::String(
                    value.as_str().unwrap().trim().trim_end_matches('\0').to_string(),
                )
            })
            .boxed(),
        DynSolType::Tuple(ref params) => params
            .iter()
            .map(|param| fuzz_param_inner(param, &FuzzConfig::new(), None))
            .collect::<Vec<_>>()
            .prop_map(DynSolValue::Tuple)
            .boxed(),
        DynSolType::FixedArray(ref param, size) => {
            proptest::collection::vec(fuzz_param_inner(param, &FuzzConfig::new(), None), size)
                .prop_map(DynSolValue::FixedArray)
                .boxed()
        }
        DynSolType::Array(ref param) => proptest::collection::vec(
            fuzz_param_inner(param, &FuzzConfig::new(), None),
            0..MAX_ARRAY_LEN,
        )
        .prop_map(DynSolValue::Array)
        .boxed(),
        _ => panic!("unsupported fuzz param type: {param}"),
    }
}

/// Given a parameter type, returns a strategy for generating values for that type, given some EVM
/// fuzz state.
///
/// Works with ABI Encoder v2 tuples.
pub fn fuzz_param_from_state(
    param: &DynSolType,
    state: &EvmFuzzState,
) -> BoxedStrategy<DynSolValue> {
    // Value strategy that uses the state.
    let value = || {
        let state = state.clone();
        let param = param.clone();
        // Generate a bias and use it to pick samples or non-persistent values (50 / 50).
        // Use `Index` instead of `Selector` when selecting a value to avoid iterating over the
        // entire dictionary.
        any::<(bool, prop::sample::Index)>().prop_map(move |(bias, index)| {
            let state = state.dictionary_read();
            let values = if bias { state.samples(&param) } else { None }
                .unwrap_or_else(|| state.values())
                .as_slice();
            values[index.index(values.len())]
        })
    };

    // Convert the value based on the parameter type
    match *param {
        DynSolType::Address => {
            let deployed_libs = state.deployed_libs.clone();
            value()
                .prop_filter_map("filter address fuzzed from state", move |value| {
                    let fuzzed_addr = Address::from_word(value);
                    // Do not use addresses of deployed libraries as fuzz input.
                    // See <https://github.com/foundry-rs/foundry/issues/8639>.
                    if !deployed_libs.contains(&fuzzed_addr) {
                        Some(DynSolValue::Address(fuzzed_addr))
                    } else {
                        None
                    }
                })
                .boxed()
        }
        DynSolType::Function => value()
            .prop_map(move |value| {
                DynSolValue::Function(alloy_primitives::Function::from_word(value))
            })
            .boxed(),
        DynSolType::FixedBytes(size @ 1..=32) => value()
            .prop_map(move |mut v| {
                v[size..].fill(0);
                DynSolValue::FixedBytes(B256::from(v), size)
            })
            .boxed(),
        DynSolType::Bool => DynSolValue::type_strategy(param).boxed(),
        DynSolType::String => DynSolValue::type_strategy(param)
            .prop_map(move |value| {
                DynSolValue::String(
                    value.as_str().unwrap().trim().trim_end_matches('\0').to_string(),
                )
            })
            .boxed(),
        DynSolType::Bytes => {
            value().prop_map(move |value| DynSolValue::Bytes(value.0.into())).boxed()
        }
        DynSolType::Int(n @ 8..=256) => match n / 8 {
            32 => value()
                .prop_map(move |value| DynSolValue::Int(I256::from_raw(value.into()), 256))
                .boxed(),
            1..=31 => value()
                .prop_map(move |value| {
                    // Generate a uintN in the correct range, then shift it to the range of intN
                    // by subtracting 2^(N-1)
                    let uint = U256::from_be_bytes(value.0) % U256::from(1).wrapping_shl(n);
                    let max_int_plus1 = U256::from(1).wrapping_shl(n - 1);
                    let num = I256::from_raw(uint.wrapping_sub(max_int_plus1));
                    DynSolValue::Int(num, n)
                })
                .boxed(),
            _ => unreachable!(),
        },
        DynSolType::Uint(n @ 8..=256) => match n / 8 {
            32 => value()
                .prop_map(move |value| DynSolValue::Uint(U256::from_be_bytes(value.0), 256))
                .boxed(),
            1..=31 => value()
                .prop_map(move |value| {
                    let uint = U256::from_be_bytes(value.0) % U256::from(1).wrapping_shl(n);
                    DynSolValue::Uint(uint, n)
                })
                .boxed(),
            _ => unreachable!(),
        },
        DynSolType::Tuple(ref params) => params
            .iter()
            .map(|p| fuzz_param_from_state(p, state))
            .collect::<Vec<_>>()
            .prop_map(DynSolValue::Tuple)
            .boxed(),
        DynSolType::FixedArray(ref param, size) => {
            proptest::collection::vec(fuzz_param_from_state(param, state), size)
                .prop_map(DynSolValue::FixedArray)
                .boxed()
        }
        DynSolType::Array(ref param) => {
            proptest::collection::vec(fuzz_param_from_state(param, state), 0..MAX_ARRAY_LEN)
                .prop_map(DynSolValue::Array)
                .boxed()
        }
        _ => panic!("unsupported fuzz param type: {param}"),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        strategies::{fuzz_calldata, fuzz_calldata_from_state, EvmFuzzState},
        FuzzFixtures,
    };
    use alloy_dyn_abi::{DynSolType, DynSolValue};
    use alloy_primitives::U256;
    use foundry_common::abi::get_func;
    use foundry_config::FuzzDictionaryConfig;
    use proptest::{prelude::Strategy, test_runner::TestRunner};
    use revm::db::{CacheDB, EmptyDB};

    use super::{fuzz_param_inner, FuzzConfig};

    #[test]
    fn can_fuzz_array() {
        let f = "testArray(uint64[2] calldata values)";
        let func = get_func(f).unwrap();
        let db = CacheDB::new(EmptyDB::default());
        let state = EvmFuzzState::new(&db, FuzzDictionaryConfig::default(), &[]);
        let strategy = proptest::prop_oneof![
            60 => fuzz_calldata(func.clone(), &FuzzFixtures::default()),
            40 => fuzz_calldata_from_state(func, &state),
        ];
        let cfg = proptest::test_runner::Config { failure_persistence: None, ..Default::default() };
        let mut runner = proptest::test_runner::TestRunner::new(cfg);
        let _ = runner.run(&strategy, |_| Ok(()));
    }

    #[test]
    fn test_uint_param_with_range() {
        let mut config = FuzzConfig::new();
        let min = U256::from(100u64);
        let max = U256::from(1000u64);
        config = config.with_range("amount", min, max);

        let param = DynSolType::Uint(256);
        let strategy = fuzz_param_inner(&param, &config, Some((&[], "amount")));

        let mut runner = TestRunner::default();
        for _ in 0..1000 {
            let value = strategy.new_tree(&mut runner).unwrap().current();
            if let DynSolValue::Uint(value, _) = value {
                assert!(
                    value >= min && value <= max,
                    "Generated value {value} outside configured range [{min}, {max}]"
                );
            } else {
                panic!("Expected Uint value");
            }
        }
    }

    #[test]
    fn test_uint_param_without_range() {
        let config = FuzzConfig::new();
        let param = DynSolType::Uint(8);
        let strategy = fuzz_param_inner(&param, &config, None);

        let mut runner = TestRunner::default();
        for _ in 0..1000 {
            let value = strategy.new_tree(&mut runner).unwrap().current();
            if let DynSolValue::Uint(value, bits) = value {
                assert!(value <= U256::from(u8::MAX), "Generated value {value} exceeds uint8 max");
                assert_eq!(bits, 8, "Incorrect bit size");
            } else {
                panic!("Expected Uint value");
            }
        }
    }

    #[test]
    fn test_uint_param_with_fixtures() {
        let config = FuzzConfig::new();
        let fixtures = vec![
            DynSolValue::Uint(U256::from(500u64), 256),
            DynSolValue::Uint(U256::from(600u64), 256),
        ];

        let param = DynSolType::Uint(256);
        let strategy = fuzz_param_inner(&param, &config, Some((&fixtures, "test")));

        let mut runner = TestRunner::default();
        let mut found_fixture = false;

        for _ in 0..1000 {
            let value = strategy.new_tree(&mut runner).unwrap().current();
            if let DynSolValue::Uint(value, _) = value {
                if value == U256::from(500u64) || value == U256::from(600u64) {
                    found_fixture = true;
                    break;
                }
            }
        }
        assert!(found_fixture, "Never generated fixture value");
    }

    #[test]
    fn test_uint_param_with_range_and_fixtures() {
        let mut config = FuzzConfig::new();
        let min = U256::from(100u64);
        let max = U256::from(1000u64);
        config = config.with_range("test", min, max);

        let fixtures = vec![
            DynSolValue::Uint(U256::from(50u64), 256),
            DynSolValue::Uint(U256::from(500u64), 256),
            DynSolValue::Uint(U256::from(1500u64), 256),
        ];

        let param = DynSolType::Uint(256);
        let strategy = fuzz_param_inner(&param, &config, Some((&fixtures, "test")));

        let mut runner = TestRunner::default();
        for _ in 0..1000 {
            let value = strategy.new_tree(&mut runner).unwrap().current();
            if let DynSolValue::Uint(value, _) = value {
                assert!(
                    value >= min && value <= max,
                    "Generated value {value} outside configured range [{min}, {max}]"
                );
            }
        }
    }

    #[test]
    fn test_param_range_matching() {
        let mut config = FuzzConfig::new();
        config = config.with_range("amount", U256::from(100u64), U256::from(1000u64)).with_range(
            "other",
            U256::from(2000u64),
            U256::from(3000u64),
        );

        let param = DynSolType::Uint(256);
        let mut runner = TestRunner::default();

        let strategy1 = fuzz_param_inner(&param, &config, Some((&[], "amount")));
        for _ in 0..100 {
            let value = strategy1.new_tree(&mut runner).unwrap().current();
            match value {
                DynSolValue::Uint(value, bits) => {
                    assert_eq!(bits, 256, "Incorrect bit size");
                    assert!(
                        value >= U256::from(100u64) && value <= U256::from(1000u64),
                        "Generated value {value} outside 'amount' range [100, 1000]"
                    );
                }
                _ => panic!("Expected Uint value"),
            }
        }

        let strategy2 = fuzz_param_inner(&param, &config, Some((&[], "nonexistent")));
        for _ in 0..100 {
            let value = strategy2.new_tree(&mut runner).unwrap().current();
            match value {
                DynSolValue::Uint(value, bits) => {
                    assert_eq!(bits, 256, "Incorrect bit size");
                    assert!(
                        value <= (U256::from(1) << 256) - U256::from(1),
                        "Generated value {value} exceeds maximum uint256 value"
                    );
                }
                _ => panic!("Expected Uint value"),
            }
        }
    }
}
