//! Implementations of [`Evm`](spec::Group::Evm) cheatcodes.

use crate::{
    inspector::{InnerEcx, RecordDebugStepInfo},
    BroadcastableTransaction, Cheatcode, Cheatcodes, CheatcodesExecutor, CheatsCtxt, Error, Result,
    Vm::*,
};
use alloy_consensus::TxEnvelope;
use alloy_genesis::{Genesis, GenesisAccount};
use alloy_primitives::{map::HashMap, Address, Bytes, B256, U256};
use alloy_rlp::Decodable;
use alloy_sol_types::SolValue;
use foundry_common::fs::{read_json_file, write_json_file};
use foundry_evm_core::{
    backend::{DatabaseExt, RevertStateSnapshotAction},
    constants::{CALLER, CHEATCODE_ADDRESS, HARDHAT_CONSOLE_ADDRESS, TEST_CONTRACT_ADDRESS},
};
use foundry_evm_traces::StackSnapshotType;
use rand::Rng;
use revm::primitives::{Account, Bytecode, SpecId, KECCAK_EMPTY};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Display,
    path::Path,
};

mod record_debug_step;
use record_debug_step::{convert_call_trace_to_debug_step, flatten_call_trace};
use serde::Serialize;

mod fork;
pub(crate) mod mapping;
pub(crate) mod mock;
pub(crate) mod prank;

/// Records storage slots reads and writes.
#[derive(Clone, Debug, Default)]
pub struct RecordAccess {
    /// Storage slots reads.
    pub reads: HashMap<Address, Vec<U256>>,
    /// Storage slots writes.
    pub writes: HashMap<Address, Vec<U256>>,
}

impl RecordAccess {
    /// Records a read access to a storage slot.
    pub fn record_read(&mut self, target: Address, slot: U256) {
        self.reads.entry(target).or_default().push(slot);
    }

    /// Records a write access to a storage slot.
    ///
    /// This also records a read internally as `SSTORE` does an implicit `SLOAD`.
    pub fn record_write(&mut self, target: Address, slot: U256) {
        self.record_read(target, slot);
        self.writes.entry(target).or_default().push(slot);
    }
}

/// Records the `snapshotGas*` cheatcodes.
#[derive(Clone, Debug)]
pub struct GasRecord {
    /// The group name of the gas snapshot.
    pub group: String,
    /// The name of the gas snapshot.
    pub name: String,
    /// The total gas used in the gas snapshot.
    pub gas_used: u64,
    /// Depth at which the gas snapshot was taken.
    pub depth: u64,
}

/// Records `deal` cheatcodes
#[derive(Clone, Debug)]
pub struct DealRecord {
    /// Target of the deal.
    pub address: Address,
    /// The balance of the address before deal was applied
    pub old_balance: U256,
    /// Balance after deal was applied
    pub new_balance: U256,
}

/// Storage slot diff info.
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct SlotStateDiff {
    /// Initial storage value.
    previous_value: B256,
    /// Current storage value.
    new_value: B256,
}

/// Balance diff info.
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct BalanceDiff {
    /// Initial storage value.
    previous_value: U256,
    /// Current storage value.
    new_value: U256,
}

/// Account state diff info.
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct AccountStateDiffs {
    /// Address label, if any set.
    label: Option<String>,
    /// Account balance changes.
    balance_diff: Option<BalanceDiff>,
    /// State changes, per slot.
    state_diff: BTreeMap<B256, SlotStateDiff>,
}

impl Display for AccountStateDiffs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> eyre::Result<(), std::fmt::Error> {
        // Print changed account.
        if let Some(label) = &self.label {
            writeln!(f, "label: {label}")?;
        }
        // Print balance diff if changed.
        if let Some(balance_diff) = &self.balance_diff {
            if balance_diff.previous_value != balance_diff.new_value {
                writeln!(
                    f,
                    "- balance diff: {} → {}",
                    balance_diff.previous_value, balance_diff.new_value
                )?;
            }
        }
        // Print state diff if any.
        if !&self.state_diff.is_empty() {
            writeln!(f, "- state diff:")?;
            for (slot, slot_changes) in &self.state_diff {
                writeln!(
                    f,
                    "@ {slot}: {} → {}",
                    slot_changes.previous_value, slot_changes.new_value
                )?;
            }
        }

        Ok(())
    }
}

impl Cheatcode for addrCall {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { privateKey } = self;
        let wallet = super::crypto::parse_wallet(privateKey)?;
        Ok(wallet.address().abi_encode())
    }
}

impl Cheatcode for getNonce_0Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { account } = self;
        get_nonce(ccx, account)
    }
}

impl Cheatcode for getNonce_1Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { wallet } = self;
        get_nonce(ccx, &wallet.addr)
    }
}

impl Cheatcode for loadCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { target, slot } = *self;
        ensure_not_precompile!(&target, ccx);
        ccx.ecx.load_account(target)?;
        let mut val = ccx.ecx.sload(target, slot.into())?;

        if val.is_cold && val.data.is_zero() {
            if ccx.state.has_arbitrary_storage(&target) {
                // If storage slot is untouched and load from a target with arbitrary storage,
                // then set random value for current slot.
                let rand_value = ccx.state.rng().gen();
                ccx.state.arbitrary_storage.as_mut().unwrap().save(
                    ccx.ecx,
                    target,
                    slot.into(),
                    rand_value,
                );
                val.data = rand_value;
            } else if ccx.state.is_arbitrary_storage_copy(&target) {
                // If storage slot is untouched and load from a target that copies storage from
                // a source address with arbitrary storage, then copy existing arbitrary value.
                // If no arbitrary value generated yet, then the random one is saved and set.
                let rand_value = ccx.state.rng().gen();
                val.data = ccx.state.arbitrary_storage.as_mut().unwrap().copy(
                    ccx.ecx,
                    target,
                    slot.into(),
                    rand_value,
                );
            }
        }

        Ok(val.abi_encode())
    }
}

impl Cheatcode for loadAllocsCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { pathToAllocsJson } = self;

        let path = Path::new(pathToAllocsJson);
        ensure!(path.exists(), "allocs file does not exist: {pathToAllocsJson}");

        // Let's first assume we're reading a file with only the allocs.
        let allocs: BTreeMap<Address, GenesisAccount> = match read_json_file(path) {
            Ok(allocs) => allocs,
            Err(_) => {
                // Let's try and read from a genesis file, and extract allocs.
                let genesis = read_json_file::<Genesis>(path)?;
                genesis.alloc
            }
        };

        // Then, load the allocs into the database.
        ccx.ecx
            .db
            .load_allocs(&allocs, &mut ccx.ecx.journaled_state)
            .map(|()| Vec::default())
            .map_err(|e| fmt_err!("failed to load allocs: {e}"))
    }
}

impl Cheatcode for cloneAccountCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { source, target } = self;

        let account = ccx.ecx.journaled_state.load_account(*source, &mut ccx.ecx.db)?;
        ccx.ecx.db.clone_account(
            &genesis_account(account.data),
            target,
            &mut ccx.ecx.journaled_state,
        )?;
        // Cloned account should persist in forked envs.
        ccx.ecx.db.add_persistent_account(*target);
        Ok(Default::default())
    }
}

impl Cheatcode for dumpStateCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { pathToStateJson } = self;
        let path = Path::new(pathToStateJson);

        // Do not include system account or empty accounts in the dump.
        let skip = |key: &Address, val: &Account| {
            key == &CHEATCODE_ADDRESS ||
                key == &CALLER ||
                key == &HARDHAT_CONSOLE_ADDRESS ||
                key == &TEST_CONTRACT_ADDRESS ||
                key == &ccx.caller ||
                key == &ccx.state.config.evm_opts.sender ||
                val.is_empty()
        };

        let alloc = ccx
            .ecx
            .journaled_state
            .state()
            .iter_mut()
            .filter(|(key, val)| !skip(key, val))
            .map(|(key, val)| (key, genesis_account(val)))
            .collect::<BTreeMap<_, _>>();

        write_json_file(path, &alloc)?;
        Ok(Default::default())
    }
}

impl Cheatcode for recordCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        state.accesses = Some(Default::default());
        Ok(Default::default())
    }
}

impl Cheatcode for accessesCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self { target } = *self;
        let result = state
            .accesses
            .as_mut()
            .map(|accesses| {
                (
                    &accesses.reads.entry(target).or_default()[..],
                    &accesses.writes.entry(target).or_default()[..],
                )
            })
            .unwrap_or_default();
        Ok(result.abi_encode_params())
    }
}

impl Cheatcode for recordLogsCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        state.recorded_logs = Some(Default::default());
        Ok(Default::default())
    }
}

impl Cheatcode for getRecordedLogsCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        Ok(state.recorded_logs.replace(Default::default()).unwrap_or_default().abi_encode())
    }
}

impl Cheatcode for pauseGasMeteringCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        state.gas_metering.paused = true;
        Ok(Default::default())
    }
}

impl Cheatcode for resumeGasMeteringCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        state.gas_metering.resume();
        Ok(Default::default())
    }
}

impl Cheatcode for resetGasMeteringCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        state.gas_metering.reset();
        Ok(Default::default())
    }
}

impl Cheatcode for lastCallGasCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        let Some(last_call_gas) = &state.gas_metering.last_call_gas else {
            bail!("no external call was made yet");
        };
        Ok(last_call_gas.abi_encode())
    }
}

impl Cheatcode for chainIdCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { newChainId } = self;
        ensure!(*newChainId <= U256::from(u64::MAX), "chain ID must be less than 2^64 - 1");
        ccx.ecx.env.cfg.chain_id = newChainId.to();
        Ok(Default::default())
    }
}

impl Cheatcode for coinbaseCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { newCoinbase } = self;
        ccx.ecx.env.block.coinbase = *newCoinbase;
        Ok(Default::default())
    }
}

impl Cheatcode for difficultyCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { newDifficulty } = self;
        ensure!(
            ccx.ecx.spec_id() < SpecId::MERGE,
            "`difficulty` is not supported after the Paris hard fork, use `prevrandao` instead; \
             see EIP-4399: https://eips.ethereum.org/EIPS/eip-4399"
        );
        ccx.ecx.env.block.difficulty = *newDifficulty;
        Ok(Default::default())
    }
}

impl Cheatcode for feeCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { newBasefee } = self;
        ccx.ecx.env.block.basefee = *newBasefee;
        Ok(Default::default())
    }
}

impl Cheatcode for prevrandao_0Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { newPrevrandao } = self;
        ensure!(
            ccx.ecx.spec_id() >= SpecId::MERGE,
            "`prevrandao` is not supported before the Paris hard fork, use `difficulty` instead; \
             see EIP-4399: https://eips.ethereum.org/EIPS/eip-4399"
        );
        ccx.ecx.env.block.prevrandao = Some(*newPrevrandao);
        Ok(Default::default())
    }
}

impl Cheatcode for prevrandao_1Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { newPrevrandao } = self;
        ensure!(
            ccx.ecx.spec_id() >= SpecId::MERGE,
            "`prevrandao` is not supported before the Paris hard fork, use `difficulty` instead; \
             see EIP-4399: https://eips.ethereum.org/EIPS/eip-4399"
        );
        ccx.ecx.env.block.prevrandao = Some((*newPrevrandao).into());
        Ok(Default::default())
    }
}

impl Cheatcode for blobhashesCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { hashes } = self;
        ensure!(
            ccx.ecx.spec_id() >= SpecId::CANCUN,
            "`blobhashes` is not supported before the Cancun hard fork; \
             see EIP-4844: https://eips.ethereum.org/EIPS/eip-4844"
        );
        ccx.ecx.env.tx.blob_hashes.clone_from(hashes);
        Ok(Default::default())
    }
}

impl Cheatcode for getBlobhashesCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self {} = self;
        ensure!(
            ccx.ecx.spec_id() >= SpecId::CANCUN,
            "`getBlobhashes` is not supported before the Cancun hard fork; \
             see EIP-4844: https://eips.ethereum.org/EIPS/eip-4844"
        );
        Ok(ccx.ecx.env.tx.blob_hashes.clone().abi_encode())
    }
}

impl Cheatcode for rollCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { newHeight } = self;
        ccx.ecx.env.block.number = *newHeight;
        Ok(Default::default())
    }
}

impl Cheatcode for getBlockNumberCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self {} = self;
        Ok(ccx.ecx.env.block.number.abi_encode())
    }
}

impl Cheatcode for txGasPriceCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { newGasPrice } = self;
        ccx.ecx.env.tx.gas_price = *newGasPrice;
        Ok(Default::default())
    }
}

impl Cheatcode for warpCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { newTimestamp } = self;
        ccx.ecx.env.block.timestamp = *newTimestamp;
        Ok(Default::default())
    }
}

impl Cheatcode for getBlockTimestampCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self {} = self;
        Ok(ccx.ecx.env.block.timestamp.abi_encode())
    }
}

impl Cheatcode for blobBaseFeeCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { newBlobBaseFee } = self;
        ensure!(
            ccx.ecx.spec_id() >= SpecId::CANCUN,
            "`blobBaseFee` is not supported before the Cancun hard fork; \
             see EIP-4844: https://eips.ethereum.org/EIPS/eip-4844"
        );
        ccx.ecx.env.block.set_blob_excess_gas_and_price(
            (*newBlobBaseFee).to(),
            ccx.ecx.spec_id() >= SpecId::PRAGUE,
        );
        Ok(Default::default())
    }
}

impl Cheatcode for getBlobBaseFeeCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self {} = self;
        Ok(ccx.ecx.env.block.get_blob_excess_gas().unwrap_or(0).abi_encode())
    }
}

impl Cheatcode for dealCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { account: address, newBalance: new_balance } = *self;
        let account = journaled_account(ccx.ecx, address)?;
        let old_balance = std::mem::replace(&mut account.info.balance, new_balance);
        let record = DealRecord { address, old_balance, new_balance };
        ccx.state.eth_deals.push(record);
        Ok(Default::default())
    }
}

impl Cheatcode for etchCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { target, newRuntimeBytecode } = self;
        ensure_not_precompile!(target, ccx);
        ccx.ecx.load_account(*target)?;
        let bytecode = Bytecode::new_raw(Bytes::copy_from_slice(newRuntimeBytecode));
        ccx.ecx.journaled_state.set_code(*target, bytecode);
        Ok(Default::default())
    }
}

impl Cheatcode for resetNonceCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { account } = self;
        let account = journaled_account(ccx.ecx, *account)?;
        // Per EIP-161, EOA nonces start at 0, but contract nonces
        // start at 1. Comparing by code_hash instead of code
        // to avoid hitting the case where account's code is None.
        let empty = account.info.code_hash == KECCAK_EMPTY;
        let nonce = if empty { 0 } else { 1 };
        account.info.nonce = nonce;
        debug!(target: "cheatcodes", nonce, "reset");
        Ok(Default::default())
    }
}

impl Cheatcode for setNonceCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { account, newNonce } = *self;
        let account = journaled_account(ccx.ecx, account)?;
        // nonce must increment only
        let current = account.info.nonce;
        ensure!(
            newNonce >= current,
            "new nonce ({newNonce}) must be strictly equal to or higher than the \
             account's current nonce ({current})"
        );
        account.info.nonce = newNonce;
        Ok(Default::default())
    }
}

impl Cheatcode for setNonceUnsafeCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { account, newNonce } = *self;
        let account = journaled_account(ccx.ecx, account)?;
        account.info.nonce = newNonce;
        Ok(Default::default())
    }
}

impl Cheatcode for storeCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { target, slot, value } = *self;
        ensure_not_precompile!(&target, ccx);
        // ensure the account is touched
        let _ = journaled_account(ccx.ecx, target)?;
        ccx.ecx.sstore(target, slot.into(), value.into())?;
        Ok(Default::default())
    }
}

impl Cheatcode for coolCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { target } = self;
        if let Some(account) = ccx.ecx.journaled_state.state.get_mut(target) {
            account.unmark_touch();
            account.storage.clear();
        }
        Ok(Default::default())
    }
}

impl Cheatcode for readCallersCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self {} = self;
        read_callers(ccx.state, &ccx.ecx.env.tx.caller)
    }
}

impl Cheatcode for snapshotValue_0Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { name, value } = self;
        inner_value_snapshot(ccx, None, Some(name.clone()), value.to_string())
    }
}

impl Cheatcode for snapshotValue_1Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { group, name, value } = self;
        inner_value_snapshot(ccx, Some(group.clone()), Some(name.clone()), value.to_string())
    }
}

impl Cheatcode for snapshotGasLastCall_0Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { name } = self;
        let Some(last_call_gas) = &ccx.state.gas_metering.last_call_gas else {
            bail!("no external call was made yet");
        };
        inner_last_gas_snapshot(ccx, None, Some(name.clone()), last_call_gas.gasTotalUsed)
    }
}

impl Cheatcode for snapshotGasLastCall_1Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { name, group } = self;
        let Some(last_call_gas) = &ccx.state.gas_metering.last_call_gas else {
            bail!("no external call was made yet");
        };
        inner_last_gas_snapshot(
            ccx,
            Some(group.clone()),
            Some(name.clone()),
            last_call_gas.gasTotalUsed,
        )
    }
}

impl Cheatcode for startSnapshotGas_0Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { name } = self;
        inner_start_gas_snapshot(ccx, None, Some(name.clone()))
    }
}

impl Cheatcode for startSnapshotGas_1Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { group, name } = self;
        inner_start_gas_snapshot(ccx, Some(group.clone()), Some(name.clone()))
    }
}

impl Cheatcode for stopSnapshotGas_0Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self {} = self;
        inner_stop_gas_snapshot(ccx, None, None)
    }
}

impl Cheatcode for stopSnapshotGas_1Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { name } = self;
        inner_stop_gas_snapshot(ccx, None, Some(name.clone()))
    }
}

impl Cheatcode for stopSnapshotGas_2Call {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { group, name } = self;
        inner_stop_gas_snapshot(ccx, Some(group.clone()), Some(name.clone()))
    }
}

// Deprecated in favor of `snapshotStateCall`
impl Cheatcode for snapshotCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self {} = self;
        inner_snapshot_state(ccx)
    }
}

impl Cheatcode for snapshotStateCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self {} = self;
        inner_snapshot_state(ccx)
    }
}

// Deprecated in favor of `revertToStateCall`
impl Cheatcode for revertToCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { snapshotId } = self;
        inner_revert_to_state(ccx, *snapshotId)
    }
}

impl Cheatcode for revertToStateCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { snapshotId } = self;
        inner_revert_to_state(ccx, *snapshotId)
    }
}

// Deprecated in favor of `revertToStateAndDeleteCall`
impl Cheatcode for revertToAndDeleteCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { snapshotId } = self;
        inner_revert_to_state_and_delete(ccx, *snapshotId)
    }
}

impl Cheatcode for revertToStateAndDeleteCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { snapshotId } = self;
        inner_revert_to_state_and_delete(ccx, *snapshotId)
    }
}

// Deprecated in favor of `deleteStateSnapshotCall`
impl Cheatcode for deleteSnapshotCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { snapshotId } = self;
        inner_delete_state_snapshot(ccx, *snapshotId)
    }
}

impl Cheatcode for deleteStateSnapshotCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { snapshotId } = self;
        inner_delete_state_snapshot(ccx, *snapshotId)
    }
}

// Deprecated in favor of `deleteStateSnapshotsCall`
impl Cheatcode for deleteSnapshotsCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self {} = self;
        inner_delete_state_snapshots(ccx)
    }
}

impl Cheatcode for deleteStateSnapshotsCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self {} = self;
        inner_delete_state_snapshots(ccx)
    }
}

impl Cheatcode for startStateDiffRecordingCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        state.recorded_account_diffs_stack = Some(Default::default());
        Ok(Default::default())
    }
}

impl Cheatcode for stopAndReturnStateDiffCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        get_state_diff(state)
    }
}

impl Cheatcode for getStateDiffCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let mut diffs = String::new();
        let state_diffs = get_recorded_state_diffs(state);
        for (address, state_diffs) in state_diffs {
            diffs.push_str(&format!("{address}\n"));
            diffs.push_str(&format!("{state_diffs}\n"));
        }
        Ok(diffs.abi_encode())
    }
}

impl Cheatcode for getStateDiffJsonCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let state_diffs = get_recorded_state_diffs(state);
        Ok(serde_json::to_string(&state_diffs)?.abi_encode())
    }
}

impl Cheatcode for broadcastRawTransactionCall {
    fn apply_full(&self, ccx: &mut CheatsCtxt, executor: &mut dyn CheatcodesExecutor) -> Result {
        let tx = TxEnvelope::decode(&mut self.data.as_ref())
            .map_err(|err| fmt_err!("failed to decode RLP-encoded transaction: {err}"))?;

        ccx.ecx.db.transact_from_tx(
            &tx.clone().into(),
            (*ccx.ecx.env).clone(),
            &mut ccx.ecx.journaled_state,
            &mut *executor.get_inspector(ccx.state),
        )?;

        if ccx.state.broadcast.is_some() {
            ccx.state.broadcastable_transactions.push_back(BroadcastableTransaction {
                rpc: ccx.db.active_fork_url(),
                transaction: tx.try_into()?,
            });
        }

        Ok(Default::default())
    }
}

impl Cheatcode for setBlockhashCall {
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        let Self { blockNumber, blockHash } = *self;
        ensure!(
            blockNumber <= ccx.ecx.env.block.number,
            "block number must be less than or equal to the current block number"
        );

        ccx.ecx.db.set_blockhash(blockNumber, blockHash);

        Ok(Default::default())
    }
}

impl Cheatcode for startDebugTraceRecordingCall {
    fn apply_full(&self, ccx: &mut CheatsCtxt, executor: &mut dyn CheatcodesExecutor) -> Result {
        let Some(tracer) = executor.tracing_inspector().and_then(|t| t.as_mut()) else {
            return Err(Error::from("no tracer initiated, consider adding -vvv flag"))
        };

        let mut info = RecordDebugStepInfo {
            // will be updated later
            start_node_idx: 0,
            // keep the original config to revert back later
            original_tracer_config: *tracer.config(),
        };

        // turn on tracer configuration for recording
        tracer.update_config(|config| {
            config
                .set_steps(true)
                .set_memory_snapshots(true)
                .set_stack_snapshots(StackSnapshotType::Full)
        });

        // track where the recording starts
        if let Some(last_node) = tracer.traces().nodes().last() {
            info.start_node_idx = last_node.idx;
        }

        ccx.state.record_debug_steps_info = Some(info);
        Ok(Default::default())
    }
}

impl Cheatcode for stopAndReturnDebugTraceRecordingCall {
    fn apply_full(&self, ccx: &mut CheatsCtxt, executor: &mut dyn CheatcodesExecutor) -> Result {
        let Some(tracer) = executor.tracing_inspector().and_then(|t| t.as_mut()) else {
            return Err(Error::from("no tracer initiated, consider adding -vvv flag"))
        };

        let Some(record_info) = ccx.state.record_debug_steps_info else {
            return Err(Error::from("nothing recorded"))
        };

        // Use the trace nodes to flatten the call trace
        let root = tracer.traces();
        let steps = flatten_call_trace(0, root, record_info.start_node_idx);

        let debug_steps: Vec<DebugStep> =
            steps.iter().map(|&step| convert_call_trace_to_debug_step(step)).collect();
        // Free up memory by clearing the steps if they are not recorded outside of cheatcode usage.
        if !record_info.original_tracer_config.record_steps {
            tracer.traces_mut().nodes_mut().iter_mut().for_each(|node| {
                node.trace.steps = Vec::new();
                node.logs = Vec::new();
                node.ordering = Vec::new();
            });
        }

        // Revert the tracer config to the one before recording
        tracer.update_config(|_config| record_info.original_tracer_config);

        // Clean up the recording info
        ccx.state.record_debug_steps_info = None;

        Ok(debug_steps.abi_encode())
    }
}

pub(super) fn get_nonce(ccx: &mut CheatsCtxt, address: &Address) -> Result {
    let account = ccx.ecx.journaled_state.load_account(*address, &mut ccx.ecx.db)?;
    Ok(account.info.nonce.abi_encode())
}

fn inner_snapshot_state(ccx: &mut CheatsCtxt) -> Result {
    Ok(ccx.ecx.db.snapshot_state(&ccx.ecx.journaled_state, &ccx.ecx.env).abi_encode())
}

fn inner_revert_to_state(ccx: &mut CheatsCtxt, snapshot_id: U256) -> Result {
    let result = if let Some(journaled_state) = ccx.ecx.db.revert_state(
        snapshot_id,
        &ccx.ecx.journaled_state,
        &mut ccx.ecx.env,
        RevertStateSnapshotAction::RevertKeep,
    ) {
        // we reset the evm's journaled_state to the state of the snapshot previous state
        ccx.ecx.journaled_state = journaled_state;
        true
    } else {
        false
    };
    Ok(result.abi_encode())
}

fn inner_revert_to_state_and_delete(ccx: &mut CheatsCtxt, snapshot_id: U256) -> Result {
    let result = if let Some(journaled_state) = ccx.ecx.db.revert_state(
        snapshot_id,
        &ccx.ecx.journaled_state,
        &mut ccx.ecx.env,
        RevertStateSnapshotAction::RevertRemove,
    ) {
        // we reset the evm's journaled_state to the state of the snapshot previous state
        ccx.ecx.journaled_state = journaled_state;
        true
    } else {
        false
    };
    Ok(result.abi_encode())
}

fn inner_delete_state_snapshot(ccx: &mut CheatsCtxt, snapshot_id: U256) -> Result {
    let result = ccx.ecx.db.delete_state_snapshot(snapshot_id);
    Ok(result.abi_encode())
}

fn inner_delete_state_snapshots(ccx: &mut CheatsCtxt) -> Result {
    ccx.ecx.db.delete_state_snapshots();
    Ok(Default::default())
}

fn inner_value_snapshot(
    ccx: &mut CheatsCtxt,
    group: Option<String>,
    name: Option<String>,
    value: String,
) -> Result {
    let (group, name) = derive_snapshot_name(ccx, group, name);

    ccx.state.gas_snapshots.entry(group).or_default().insert(name, value);

    Ok(Default::default())
}

fn inner_last_gas_snapshot(
    ccx: &mut CheatsCtxt,
    group: Option<String>,
    name: Option<String>,
    value: u64,
) -> Result {
    // If isolation mode is not enabled, silently skip the snapshotting.
    if !ccx.state.config.evm_opts.isolate {
        return Ok(value.abi_encode());
    }

    let (group, name) = derive_snapshot_name(ccx, group, name);

    ccx.state.gas_snapshots.entry(group).or_default().insert(name, value.to_string());

    Ok(value.abi_encode())
}

fn inner_start_gas_snapshot(
    ccx: &mut CheatsCtxt,
    group: Option<String>,
    name: Option<String>,
) -> Result {
    // If isolation mode is not enabled, silently skip the snapshotting.
    if !ccx.state.config.evm_opts.isolate {
        return Ok(Default::default());
    }

    // Revert if there is an active gas snapshot as we can only have one active snapshot at a time.
    if ccx.state.gas_metering.active_gas_snapshot.is_some() {
        let (group, name) = ccx.state.gas_metering.active_gas_snapshot.as_ref().unwrap().clone();
        bail!("gas snapshot was already started with group: {group} and name: {name}");
    }

    let (group, name) = derive_snapshot_name(ccx, group, name);

    ccx.state.gas_metering.gas_records.push(GasRecord {
        group: group.clone(),
        name: name.clone(),
        gas_used: 0,
        depth: ccx.ecx.journaled_state.depth(),
    });

    ccx.state.gas_metering.active_gas_snapshot = Some((group, name));

    ccx.state.gas_metering.start();

    Ok(Default::default())
}

fn inner_stop_gas_snapshot(
    ccx: &mut CheatsCtxt,
    group: Option<String>,
    name: Option<String>,
) -> Result {
    // If isolation mode is not enabled, silently skip the snapshotting.
    if !ccx.state.config.evm_opts.isolate {
        return Ok(0.abi_encode());
    }

    // If group and name are not provided, use the last snapshot group and name.
    let (group, name) = group.zip(name).unwrap_or_else(|| {
        let (group, name) = ccx.state.gas_metering.active_gas_snapshot.as_ref().unwrap().clone();
        (group, name)
    });

    if let Some(record) = ccx
        .state
        .gas_metering
        .gas_records
        .iter_mut()
        .find(|record| record.group == group && record.name == name)
    {
        // Calculate the gas used since the snapshot was started.
        // We subtract 171 from the gas used to account for gas used by the snapshot itself.
        let value = record.gas_used.saturating_sub(171);

        ccx.state
            .gas_snapshots
            .entry(group.clone())
            .or_default()
            .insert(name.clone(), value.to_string());

        // Stop the gas metering.
        ccx.state.gas_metering.stop();

        // Remove the gas record.
        ccx.state
            .gas_metering
            .gas_records
            .retain(|record| record.group != group && record.name != name);

        // Clear last snapshot cache if we have an exact match.
        if let Some((snapshot_group, snapshot_name)) = &ccx.state.gas_metering.active_gas_snapshot {
            if snapshot_group == &group && snapshot_name == &name {
                ccx.state.gas_metering.active_gas_snapshot = None;
            }
        }

        Ok(value.abi_encode())
    } else {
        bail!("no gas snapshot was started with the name: {name} in group: {group}");
    }
}

// Derives the snapshot group and name from the provided group and name or the running contract.
fn derive_snapshot_name(
    ccx: &CheatsCtxt,
    group: Option<String>,
    name: Option<String>,
) -> (String, String) {
    let group = group.unwrap_or_else(|| {
        ccx.state.config.running_artifact.clone().expect("expected running contract").name
    });
    let name = name.unwrap_or_else(|| "default".to_string());
    (group, name)
}

/// Reads the current caller information and returns the current [CallerMode], `msg.sender` and
/// `tx.origin`.
///
/// Depending on the current caller mode, one of the following results will be returned:
/// - If there is an active prank:
///     - caller_mode will be equal to:
///         - [CallerMode::Prank] if the prank has been set with `vm.prank(..)`.
///         - [CallerMode::RecurrentPrank] if the prank has been set with `vm.startPrank(..)`.
///     - `msg.sender` will be equal to the address set for the prank.
///     - `tx.origin` will be equal to the default sender address unless an alternative one has been
///       set when configuring the prank.
///
/// - If there is an active broadcast:
///     - caller_mode will be equal to:
///         - [CallerMode::Broadcast] if the broadcast has been set with `vm.broadcast(..)`.
///         - [CallerMode::RecurrentBroadcast] if the broadcast has been set with
///           `vm.startBroadcast(..)`.
///     - `msg.sender` and `tx.origin` will be equal to the address provided when setting the
///       broadcast.
///
/// - If no caller modification is active:
///     - caller_mode will be equal to [CallerMode::None],
///     - `msg.sender` and `tx.origin` will be equal to the default sender address.
fn read_callers(state: &Cheatcodes, default_sender: &Address) -> Result {
    let Cheatcodes { prank, broadcast, .. } = state;

    let mut mode = CallerMode::None;
    let mut new_caller = default_sender;
    let mut new_origin = default_sender;
    if let Some(prank) = prank {
        mode = if prank.single_call { CallerMode::Prank } else { CallerMode::RecurrentPrank };
        new_caller = &prank.new_caller;
        if let Some(new) = &prank.new_origin {
            new_origin = new;
        }
    } else if let Some(broadcast) = broadcast {
        mode = if broadcast.single_call {
            CallerMode::Broadcast
        } else {
            CallerMode::RecurrentBroadcast
        };
        new_caller = &broadcast.new_origin;
        new_origin = &broadcast.new_origin;
    }

    Ok((mode, new_caller, new_origin).abi_encode_params())
}

/// Ensures the `Account` is loaded and touched.
pub(super) fn journaled_account<'a>(
    ecx: InnerEcx<'a, '_, '_>,
    addr: Address,
) -> Result<&'a mut Account> {
    ecx.load_account(addr)?;
    ecx.journaled_state.touch(&addr);
    Ok(ecx.journaled_state.state.get_mut(&addr).expect("account is loaded"))
}

/// Consumes recorded account accesses and returns them as an abi encoded
/// array of [AccountAccess]. If there are no accounts were
/// recorded as accessed, an abi encoded empty array is returned.
///
/// In the case where `stopAndReturnStateDiff` is called at a lower
/// depth than `startStateDiffRecording`, multiple `Vec<RecordedAccountAccesses>`
/// will be flattened, preserving the order of the accesses.
fn get_state_diff(state: &mut Cheatcodes) -> Result {
    let res = state
        .recorded_account_diffs_stack
        .replace(Default::default())
        .unwrap_or_default()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    Ok(res.abi_encode())
}

/// Helper function that creates a `GenesisAccount` from a regular `Account`.
fn genesis_account(account: &Account) -> GenesisAccount {
    GenesisAccount {
        nonce: Some(account.info.nonce),
        balance: account.info.balance,
        code: account.info.code.as_ref().map(|o| o.original_bytes()),
        storage: Some(
            account
                .storage
                .iter()
                .map(|(k, v)| (B256::from(*k), B256::from(v.present_value())))
                .collect(),
        ),
        private_key: None,
    }
}

/// Helper function to returns state diffs recorded for each changed account.
fn get_recorded_state_diffs(state: &mut Cheatcodes) -> BTreeMap<Address, AccountStateDiffs> {
    let mut state_diffs: BTreeMap<Address, AccountStateDiffs> = BTreeMap::default();
    if let Some(records) = &state.recorded_account_diffs_stack {
        records
            .iter()
            .flatten()
            .filter(|account_access| {
                !account_access.storageAccesses.is_empty() ||
                    account_access.oldBalance != account_access.newBalance
            })
            .for_each(|account_access| {
                // Record account balance diffs.
                if account_access.oldBalance != account_access.newBalance {
                    let account_diff =
                        state_diffs.entry(account_access.account).or_insert_with(|| {
                            AccountStateDiffs {
                                label: state.labels.get(&account_access.account).cloned(),
                                ..Default::default()
                            }
                        });
                    // Update balance diff. Do not overwrite the initial balance if already set.
                    if let Some(diff) = &mut account_diff.balance_diff {
                        diff.new_value = account_access.newBalance;
                    } else {
                        account_diff.balance_diff = Some(BalanceDiff {
                            previous_value: account_access.oldBalance,
                            new_value: account_access.newBalance,
                        });
                    }
                }

                // Record account state diffs.
                for storage_access in &account_access.storageAccesses {
                    if storage_access.isWrite && !storage_access.reverted {
                        let account_diff = state_diffs
                            .entry(storage_access.account)
                            .or_insert_with(|| AccountStateDiffs {
                                label: state.labels.get(&storage_access.account).cloned(),
                                ..Default::default()
                            });
                        // Update state diff. Do not overwrite the initial value if already set.
                        match account_diff.state_diff.entry(storage_access.slot) {
                            Entry::Vacant(slot_state_diff) => {
                                slot_state_diff.insert(SlotStateDiff {
                                    previous_value: storage_access.previousValue,
                                    new_value: storage_access.newValue,
                                });
                            }
                            Entry::Occupied(mut slot_state_diff) => {
                                slot_state_diff.get_mut().new_value = storage_access.newValue;
                            }
                        }
                    }
                }
            });
    }
    state_diffs
}
