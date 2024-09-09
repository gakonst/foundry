//! Implementations of [`Utilities`](spec::Group::Utilities) cheatcodes.

use crate::{Cheatcode, Cheatcodes, CheatsCtxt, Result, Vm::*};
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::U256;
use alloy_sol_types::SolValue;
use foundry_common::ens::namehash;
use foundry_evm_core::{backend::DatabaseExt, constants::DEFAULT_CREATE2_DEPLOYER};
use proptest::strategy::{Strategy, ValueTree};
use rand::Rng;
use std::collections::HashMap;

/// Contains locations of traces ignored via cheatcodes.
///
/// The way we identify location in traces is by (node_idx, item_idx) tuple where node_idx is an
/// index of a call trace node, and item_idx is a value between 0 and `node.ordering.len()` where i
/// represents point after ith item, and 0 represents the beginning of the node trace.
#[derive(Debug, Default, Clone)]
pub struct IgnoredTraces {
    /// Mapping from (start_node_idx, start_item_idx) to (end_node_idx, end_item_idx) representing
    /// ranges of trace nodes to ignore.
    pub ignored: HashMap<(usize, usize), (usize, usize)>,
    /// Keeps track of (start_node_idx, start_item_idx) of the last `vm.pauseTracing` call.
    pub last_pause_call: Option<(usize, usize)>,
}

impl Cheatcode for labelCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self { account, newLabel } = self;
        state.labels.insert(*account, newLabel.clone());
        Ok(Default::default())
    }
}

impl Cheatcode for getLabelCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self { account } = self;
        Ok(match state.labels.get(account) {
            Some(label) => label.abi_encode(),
            None => format!("unlabeled:{account}").abi_encode(),
        })
    }
}

impl Cheatcode for computeCreateAddressCall {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { nonce, deployer } = self;
        ensure!(*nonce <= U256::from(u64::MAX), "nonce must be less than 2^64 - 1");
        Ok(deployer.create(nonce.to()).abi_encode())
    }
}

impl Cheatcode for computeCreate2Address_0Call {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { salt, initCodeHash, deployer } = self;
        Ok(deployer.create2(salt, initCodeHash).abi_encode())
    }
}

impl Cheatcode for computeCreate2Address_1Call {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { salt, initCodeHash } = self;
        Ok(DEFAULT_CREATE2_DEPLOYER.create2(salt, initCodeHash).abi_encode())
    }
}

impl Cheatcode for ensNamehashCall {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { name } = self;
        Ok(namehash(name).abi_encode())
    }
}

impl Cheatcode for randomUint_0Call {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        arbitrary_uint(state, None, None)
    }
}

impl Cheatcode for randomUint_1Call {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self { min, max } = *self;
        arbitrary_uint(state, None, Some((min, max)))
    }
}

impl Cheatcode for randomAddressCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        arbitrary_address(state)
    }
}

impl Cheatcode for pauseTracingCall {
    fn apply_full<DB: DatabaseExt, E: crate::CheatcodesExecutor>(
        &self,
        ccx: &mut crate::CheatsCtxt<DB>,
        executor: &mut E,
    ) -> Result {
        let Some(tracer) = executor.tracing_inspector().and_then(|t| t.as_ref()) else {
            // No tracer -> nothing to pause
            return Ok(Default::default())
        };

        // If paused earlier, ignore the call
        if ccx.state.ignored_traces.last_pause_call.is_some() {
            return Ok(Default::default())
        }

        let cur_node = &tracer.traces().nodes().last().expect("no trace nodes");
        ccx.state.ignored_traces.last_pause_call = Some((cur_node.idx, cur_node.ordering.len()));

        Ok(Default::default())
    }
}

impl Cheatcode for resumeTracingCall {
    fn apply_full<DB: DatabaseExt, E: crate::CheatcodesExecutor>(
        &self,
        ccx: &mut crate::CheatsCtxt<DB>,
        executor: &mut E,
    ) -> Result {
        let Some(tracer) = executor.tracing_inspector().and_then(|t| t.as_ref()) else {
            // No tracer -> nothing to unpause
            return Ok(Default::default())
        };

        let Some(start) = ccx.state.ignored_traces.last_pause_call.take() else {
            // Nothing to unpause
            return Ok(Default::default())
        };

        let node = &tracer.traces().nodes().last().expect("no trace nodes");
        ccx.state.ignored_traces.ignored.insert(start, (node.idx, node.ordering.len()));

        Ok(Default::default())
    }
}

impl Cheatcode for setArbitraryStorageCall {
    fn apply_stateful<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { target } = self;
        ccx.state.arbitrary_storage().mark_arbitrary(target);

        Ok(Default::default())
    }
}

impl Cheatcode for copyStorageCall {
    fn apply_stateful<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { from, to } = self;

        ensure!(
            !ccx.state.has_arbitrary_storage(to),
            "target address cannot have arbitrary storage"
        );

        if let Ok(from_account) = ccx.load_account(*from) {
            let from_storage = from_account.storage.clone();
            if let Ok(mut to_account) = ccx.load_account(*to) {
                to_account.storage = from_storage;
                if let Some(ref mut arbitrary_storage) = &mut ccx.state.arbitrary_storage {
                    arbitrary_storage.mark_copy(from, to);
                }
            }
        }

        Ok(Default::default())
    }
}

impl Cheatcode for arbitraryUint_0Call {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        arbitrary_uint(state, None, None)
    }
}

impl Cheatcode for arbitraryUint_1Call {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self { min, max } = *self;
        arbitrary_uint(state, None, Some((min, max)))
    }
}

impl Cheatcode for arbitraryUint_2Call {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self { bits } = *self;
        arbitrary_uint(state, Some(bits), None)
    }
}

impl Cheatcode for arbitraryInt_0Call {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        arbitrary_int(state, None)
    }
}

impl Cheatcode for arbitraryInt_1Call {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self { bits } = *self;
        arbitrary_int(state, Some(bits))
    }
}

impl Cheatcode for arbitraryAddressCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        arbitrary_address(state)
    }
}

impl Cheatcode for arbitraryBoolCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        Ok(DynSolValue::type_strategy(&DynSolType::Bool)
            .new_tree(state.test_runner())
            .unwrap()
            .current()
            .abi_encode())
    }
}

impl Cheatcode for arbitraryBytesCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self { len } = *self;
        let mut val = DynSolValue::type_strategy(&DynSolType::Bytes)
            .new_tree(state.test_runner())
            .unwrap()
            .current()
            .as_bytes()
            .unwrap_or_default()
            .to_vec();
        let required_len = len.to::<usize>();
        let cur_len = val.len();
        if cur_len > required_len {
            // Slice to required length if random bytes length is lower than required length.
            Ok(val[..required_len].abi_encode())
        } else {
            // Fill with zeroes if random bytes length is lower than required length.
            val.extend(vec![0; required_len - cur_len]);
            Ok(val.abi_encode())
        }
    }
}

/// Helper to generate an arbitrary `uint` value (with given bits or bounded if specified)
/// from type strategy.
fn arbitrary_uint(
    state: &mut Cheatcodes,
    bits: Option<U256>,
    bounds: Option<(U256, U256)>,
) -> Result {
    if let Some(bits) = bits {
        // Generate arbitrary with specified bits.
        let no_bits = bits.to::<usize>();
        ensure!(no_bits <= 256, "no of bits cannot exceed 256");
        return Ok(DynSolValue::type_strategy(&DynSolType::Uint(no_bits))
            .new_tree(state.test_runner())
            .unwrap()
            .current()
            .abi_encode())
    }

    if let Some((min, max)) = bounds {
        ensure!(min <= max, "min must be less than or equal to max");
        // Generate arbitrary between range min..=max
        let exclusive_modulo = max - min;
        let mut random_number: U256 = state.test_runner().rng().gen();
        if exclusive_modulo != U256::MAX {
            let inclusive_modulo = exclusive_modulo + U256::from(1);
            random_number %= inclusive_modulo;
        }
        random_number += min;
        return Ok(random_number.abi_encode())
    }

    // Generate arbitrary `uint256` value.
    Ok(DynSolValue::type_strategy(&DynSolType::Uint(256))
        .new_tree(state.test_runner())
        .unwrap()
        .current()
        .abi_encode())
}

/// Helper to generate an arbitrary `address` value from type strategy.
fn arbitrary_address(state: &mut Cheatcodes) -> Result {
    Ok(DynSolValue::type_strategy(&DynSolType::Address)
        .new_tree(state.test_runner())
        .unwrap()
        .current()
        .abi_encode())
}

/// Helper to generate an arbitrary `int` value (with given bits if specified) from type strategy.
fn arbitrary_int(state: &mut Cheatcodes, bits: Option<U256>) -> Result {
    let no_bits = bits.unwrap_or(U256::from(256)).to::<usize>();
    ensure!(no_bits <= 256, "no of bits cannot exceed 256");
    Ok(DynSolValue::type_strategy(&DynSolType::Int(no_bits))
        .new_tree(state.test_runner())
        .unwrap()
        .current()
        .abi_encode())
}
