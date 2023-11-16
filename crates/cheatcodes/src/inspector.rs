//! Cheatcode EVM [Inspector].

use crate::{
    evm::{
        mapping::{self, MappingSlots},
        mock::{MockCallDataContext, MockCallReturnData},
        prank::Prank,
        DealRecord, RecordAccess,
    },
    script::Broadcast,
    test::expect::{
        self, ExpectedCallData, ExpectedCallTracker, ExpectedCallType, ExpectedEmit, ExpectedRevert,
    },
    CheatsConfig, CheatsCtxt, Error, Result, Vm,
};
use alloy_primitives::{Address, Bytes, B256, U256};
use alloy_sol_types::{SolInterface, SolValue};
use ethers_core::types::{
    transaction::eip2718::TypedTransaction, NameOrAddress, TransactionRequest,
};
use ethers_signers::LocalWallet;
use foundry_common::{evm::Breakpoints, RpcUrl};
use foundry_evm_core::{
    backend::{DatabaseError, DatabaseExt, RevertDiagnostic},
    constants::{CHEATCODE_ADDRESS, DEFAULT_CREATE2_DEPLOYER, HARDHAT_CONSOLE_ADDRESS, MAGIC_SKIP},
    utils::get_create_address,
};
use foundry_utils::types::ToEthers;
use itertools::Itertools;
use revm::{
    interpreter::{opcode, CallInputs, CreateInputs, Gas, InstructionResult, Interpreter},
    primitives::{BlockEnv, CreateScheme, TransactTo},
    EVMData, Inspector,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    fs::File,
    io::BufReader,
    ops::Range,
    path::PathBuf,
    sync::Arc,
};

macro_rules! try_or_continue {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(_) => return InstructionResult::Continue,
        }
    };
}

/// Contains additional, test specific resources that should be kept for the duration of the test
#[derive(Debug, Default)]
pub struct Context {
    /// Buffered readers for files opened for reading (path => BufReader mapping)
    pub opened_read_files: HashMap<PathBuf, BufReader<File>>,
}

/// Every time we clone `Context`, we want it to be empty
impl Clone for Context {
    fn clone(&self) -> Self {
        Default::default()
    }
}

impl Context {
    /// Clears the context.
    #[inline]
    pub fn clear(&mut self) {
        self.opened_read_files.clear();
    }
}

/// Helps collecting transactions from different forks.
#[derive(Debug, Clone, Default)]
pub struct BroadcastableTransaction {
    /// The optional RPC URL.
    pub rpc: Option<RpcUrl>,
    /// The transaction to broadcast.
    pub transaction: TypedTransaction,
}

/// List of transactions that can be broadcasted.
pub type BroadcastableTransactions = VecDeque<BroadcastableTransaction>;

/// An EVM inspector that handles calls to various cheatcodes, each with their own behavior.
///
/// Cheatcodes can be called by contracts during execution to modify the VM environment, such as
/// mocking addresses, signatures and altering call reverts.
///
/// Executing cheatcodes can be very powerful. Most cheatcodes are limited to evm internals, but
/// there are also cheatcodes like `ffi` which can execute arbitrary commands or `writeFile` and
/// `readFile` which can manipulate files of the filesystem. Therefore, several restrictions are
/// implemented for these cheatcodes:
/// - `ffi`, and file cheatcodes are _always_ opt-in (via foundry config) and never enabled by
///   default: all respective cheatcode handlers implement the appropriate checks
/// - File cheatcodes require explicit permissions which paths are allowed for which operation, see
///   `Config.fs_permission`
/// - Only permitted accounts are allowed to execute cheatcodes in forking mode, this ensures no
///   contract deployed on the live network is able to execute cheatcodes by simply calling the
///   cheatcode address: by default, the caller, test contract and newly deployed contracts are
///   allowed to execute cheatcodes
#[derive(Clone, Debug, Default)]
pub struct Cheatcodes {
    /// The block environment
    ///
    /// Used in the cheatcode handler to overwrite the block environment separately from the
    /// execution block environment.
    pub block: Option<BlockEnv>,

    /// The gas price
    ///
    /// Used in the cheatcode handler to overwrite the gas price separately from the gas price
    /// in the execution environment.
    pub gas_price: Option<U256>,

    /// Address labels
    pub labels: HashMap<Address, String>,

    /// Rememebered private keys
    pub script_wallets: Vec<LocalWallet>,

    /// Whether the skip cheatcode was activated
    pub skip: bool,

    /// Prank information
    pub prank: Option<Prank>,

    /// Expected revert information
    pub expected_revert: Option<ExpectedRevert>,

    /// Additional diagnostic for reverts
    pub fork_revert_diagnostic: Option<RevertDiagnostic>,

    /// Recorded storage reads and writes
    pub accesses: Option<RecordAccess>,

    /// Recorded logs
    pub recorded_logs: Option<Vec<crate::Vm::Log>>,

    /// Mocked calls
    // **Note**: inner must a BTreeMap because of special `Ord` impl for `MockCallDataContext`
    pub mocked_calls: HashMap<Address, BTreeMap<MockCallDataContext, MockCallReturnData>>,

    /// Expected calls
    pub expected_calls: ExpectedCallTracker,
    /// Expected emits
    pub expected_emits: VecDeque<ExpectedEmit>,

    /// Map of context depths to memory offset ranges that may be written to within the call depth.
    pub allowed_mem_writes: HashMap<u64, Vec<Range<u64>>>,

    /// Current broadcasting information
    pub broadcast: Option<Broadcast>,

    /// Used to correct the nonce of --sender after the initiating call. For more, check
    /// `docs/scripting`.
    pub corrected_nonce: bool,

    /// Scripting based transactions
    pub broadcastable_transactions: BroadcastableTransactions,

    /// Additional, user configurable context this Inspector has access to when inspecting a call
    pub config: Arc<CheatsConfig>,

    /// Test-scoped context holding data that needs to be reset every test run
    pub context: Context,

    /// Whether to commit FS changes such as file creations, writes and deletes.
    /// Used to prevent duplicate changes file executing non-committing calls.
    pub fs_commit: bool,

    /// Serialized JSON values.
    // **Note**: both must a BTreeMap to ensure the order of the keys is deterministic.
    pub serialized_jsons: BTreeMap<String, BTreeMap<String, Value>>,

    /// All recorded ETH `deal`s.
    pub eth_deals: Vec<DealRecord>,

    /// Holds the stored gas info for when we pause gas metering. It is an `Option<Option<..>>`
    /// because the `call` callback in an `Inspector` doesn't get access to
    /// the `revm::Interpreter` which holds the `revm::Gas` struct that
    /// we need to copy. So we convert it to a `Some(None)` in `apply_cheatcode`, and once we have
    /// the interpreter, we copy the gas struct. Then each time there is an execution of an
    /// operation, we reset the gas.
    pub gas_metering: Option<Option<Gas>>,

    /// Holds stored gas info for when we pause gas metering, and we're entering/inside
    /// CREATE / CREATE2 frames. This is needed to make gas meter pausing work correctly when
    /// paused and creating new contracts.
    pub gas_metering_create: Option<Option<Gas>>,

    /// Mapping slots.
    pub mapping_slots: Option<HashMap<Address, MappingSlots>>,

    /// The current program counter.
    pub pc: usize,
    /// Breakpoints supplied by the `breakpoint` cheatcode.
    /// `char -> (address, pc)`
    pub breakpoints: Breakpoints,

    /// Track if cool cheatcode was called on each address
    /// Mapping tracks if the address itself is cool and if each of it's slots are cool
    pub addresses: HashMap<Address, (AddressState, HashMap<U256, StorageSlotState>)>,
    /// How much gas to charge in the next step (op code) based on cool cheatcode calculations
    pub additional_gas_next_op: u64,
}

/// Whether an Address is accessed or not
#[derive(Clone, PartialEq, Debug)]
pub enum AddressState {
    /// if already accessed, then charge WARM_STORAGE_READ_COST (100)
    Warm,
    /// charge COLD_ACCOUNT_ACCESS_COST (2600)
    Cool,
}

/// Whether a Storage Slot is warm or already been modified
#[derive(Clone, PartialEq, Debug)]
pub enum StorageSlotState {
    /// charge extra based on SSTORE calculations
    WarmWithSLOAD,
    /// if SSTORE already happened, don't charge extra
    WarmWithSSTORE,
    /// same as if empty
    Cool,
}

/// Function to charge extra gas per opcode based on cool cheatcode
fn add_gas_from_cool_cheatcode<DB: DatabaseExt>(
    state: &mut Cheatcodes,
    interpreter: &mut Interpreter,
    data: &mut EVMData<'_, DB>,
) -> InstructionResult {
    if state.addresses.is_empty() {
        return InstructionResult::Continue
    }

    // For gas costs, see https://eips.ethereum.org/EIPS/eip-2200, https://eips.ethereum.org/EIPS/eip-2929

    // if previous step added gas, add it once
    // note that all the opcodes will already have a cost (usually 100)
    // so adding until it hits the gas expected for a cold key/address
    if state.additional_gas_next_op > 0 {
        interpreter.gas.record_cost(state.additional_gas_next_op);
        state.additional_gas_next_op = 0;
    }

    // if cool cheatcode was ever called on this address
    let contract_address = interpreter.contract().address;

    if state.addresses.get(&contract_address).is_some() {
        // check target itself
        match interpreter.current_opcode() {
            // via AccessListTracer
            opcode::EXTCODECOPY |
            opcode::EXTCODEHASH |
            opcode::EXTCODESIZE |
            opcode::BALANCE |
            opcode::SELFDESTRUCT => {
                // address is first parameter
                if let Ok(slot) = interpreter.stack().peek(0) {
                    let addr: Address = Address::from_word(slot.into());

                    // COLD_ACCOUNT_ACCESS_COST is 2600
                    // check this is done once per address, unless cheatcode is called again
                    // ignore if same as contract address
                    if let Some((ref mut address, _)) = state.addresses.get_mut(&addr) {
                        if *address == AddressState::Cool {
                            state.additional_gas_next_op = 2500;
                            *address = AddressState::Warm;
                        }
                    }
                }
            }
            // via AccessListTracer
            opcode::DELEGATECALL | opcode::CALL | opcode::STATICCALL | opcode::CALLCODE => {
                // address is second parameter
                if let Ok(slot) = interpreter.stack().peek(1) {
                    let addr: Address = Address::from_word(slot.into());

                    // COLD_ACCOUNT_ACCESS_COST is 2600
                    // check this is done once per address, unless cheatcode is called again
                    // ignore if same as contract address
                    if let Some((ref mut address, _)) = state.addresses.get_mut(&addr) {
                        if *address == AddressState::Cool {
                            state.additional_gas_next_op = 2500;
                            *address = AddressState::Warm;
                        }
                    }
                }
            }
            _ => {}
        }
        // check target's slots
        if let Some((_, slots)) = state.addresses.get_mut(&contract_address) {
            match interpreter.current_opcode() {
                opcode::SLOAD => {
                    let key = try_or_continue!(interpreter.stack().peek(0));

                    let account = data.journaled_state.state().get(&contract_address).unwrap();
                    if account.storage.get(&key).is_some() {
                        match slots.get(&key) {
                            None | Some(StorageSlotState::Cool) => {
                                // COLD_SLOAD_COST is 2100
                                state.additional_gas_next_op = 2000;
                                slots.insert(key, StorageSlotState::WarmWithSLOAD);
                            }
                            Some(_) => {}
                        }
                    } else {
                        slots.insert(key, StorageSlotState::WarmWithSLOAD);
                    }
                }
                opcode::SSTORE => {
                    let key = try_or_continue!(interpreter.stack().peek(0));
                    let val = try_or_continue!(interpreter.stack().peek(1));

                    let account = data.journaled_state.state().get(&contract_address).unwrap();
                    if account.storage.get(&key).is_some() {
                        // only add gas the first time the storage is touched again
                        match slots.get(&key) {
                            Some(StorageSlotState::WarmWithSLOAD) => {
                                // cool keeps the slot value changes
                                // as if the previous_or_original_value = present_value`
                                // so include the extra gas
                                let slot = account.storage.get(&key).unwrap();
                                if val != slot.present_value &&
                                    slot.present_value != slot.previous_or_original_value
                                {
                                    if slot.present_value == U256::ZERO {
                                        // SSTORE_SET_GAS is 20000
                                        state.additional_gas_next_op += 20000 - 100
                                    } else {
                                        // SSTORE_RESET_GAS is 5000 - COLD_SLOAD_COST (2100)
                                        state.additional_gas_next_op += 2900 - 100
                                    }
                                }

                                // set slot is_warm to true
                                slots.insert(key, StorageSlotState::WarmWithSSTORE);
                            }
                            None | Some(StorageSlotState::Cool) => {
                                // Means SSTORE was called without SLOAD before
                                // COLD_SLOAD_COST is 2100
                                state.additional_gas_next_op = 2100;

                                // cool keeps the slot value changes
                                // as if the previous_or_original_value = present_value`
                                // so include the extra gas
                                let slot = account.storage.get(&key).unwrap();
                                if val != slot.present_value &&
                                    slot.present_value != slot.previous_or_original_value
                                {
                                    if slot.present_value == U256::ZERO {
                                        // SSTORE_SET_GAS is 20000
                                        state.additional_gas_next_op += 20000 - 100
                                    } else {
                                        // SSTORE_RESET_GAS is 5000 - COLD_SLOAD_COST (2100)
                                        state.additional_gas_next_op += 2900 - 100
                                    }
                                }
                                slots.insert(key, StorageSlotState::WarmWithSSTORE);
                            }
                            Some(StorageSlotState::WarmWithSSTORE) => {}
                        }
                    } else {
                        slots.insert(key, StorageSlotState::WarmWithSSTORE);
                    }
                }
                _ => {}
            }
        }
    }

    InstructionResult::Continue
}

impl Cheatcodes {
    /// Creates a new `Cheatcodes` with the given settings.
    #[inline]
    pub fn new(config: Arc<CheatsConfig>) -> Self {
        Self { config, fs_commit: true, ..Default::default() }
    }

    fn apply_cheatcode<DB: DatabaseExt>(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: &CallInputs,
    ) -> Result {
        // decode the cheatcode call
        let decoded = Vm::VmCalls::abi_decode(&call.input, false)?;
        let caller = call.context.caller;

        // ensure the caller is allowed to execute cheatcodes,
        // but only if the backend is in forking mode
        data.db.ensure_cheatcode_access_forking_mode(caller)?;

        apply_dispatch(&decoded, &mut CheatsCtxt { state: self, data, caller })
    }

    /// Determines the address of the contract and marks it as allowed
    ///
    /// There may be cheatcodes in the constructor of the new contract, in order to allow them
    /// automatically we need to determine the new address
    fn allow_cheatcodes_on_create<DB: DatabaseExt>(
        &self,
        data: &mut EVMData<'_, DB>,
        inputs: &CreateInputs,
    ) {
        if data.journaled_state.depth > 1 && !data.db.has_cheatcode_access(inputs.caller) {
            // we only grant cheat code access for new contracts if the caller also has
            // cheatcode access and the new contract is created in top most call
            return
        }

        let old_nonce = data
            .journaled_state
            .state
            .get(&inputs.caller)
            .map(|acc| acc.info.nonce)
            .unwrap_or_default();
        let created_address = get_create_address(inputs, old_nonce);

        data.db.allow_cheatcode_access(created_address);
    }

    /// Called when there was a revert.
    ///
    /// Cleanup any previously applied cheatcodes that altered the state in such a way that revm's
    /// revert would run into issues.
    pub fn on_revert<DB: DatabaseExt>(&mut self, data: &mut EVMData<'_, DB>) {
        trace!(deals=?self.eth_deals.len(), "rolling back deals");

        // Delay revert clean up until expected revert is handled, if set.
        if self.expected_revert.is_some() {
            return
        }

        // we only want to apply cleanup top level
        if data.journaled_state.depth() > 0 {
            return
        }

        // Roll back all previously applied deals
        // This will prevent overflow issues in revm's [`JournaledState::journal_revert`] routine
        // which rolls back any transfers.
        while let Some(record) = self.eth_deals.pop() {
            if let Some(acc) = data.journaled_state.state.get_mut(&record.address) {
                acc.info.balance = record.old_balance;
            }
        }
    }
}

impl<DB: DatabaseExt> Inspector<DB> for Cheatcodes {
    #[inline]
    fn initialize_interp(
        &mut self,
        _: &mut Interpreter,
        data: &mut EVMData<'_, DB>,
    ) -> InstructionResult {
        // When the first interpreter is initialized we've circumvented the balance and gas checks,
        // so we apply our actual block data with the correct fees and all.
        if let Some(block) = self.block.take() {
            data.env.block = block;
        }
        if let Some(gas_price) = self.gas_price.take() {
            data.env.tx.gas_price = gas_price;
        }

        InstructionResult::Continue
    }

    fn step(
        &mut self,
        interpreter: &mut Interpreter,
        data: &mut EVMData<'_, DB>,
    ) -> InstructionResult {
        self.pc = interpreter.program_counter();

        // reset gas if gas metering is turned off
        match self.gas_metering {
            Some(None) => {
                // need to store gas metering
                self.gas_metering = Some(Some(interpreter.gas));
            }
            Some(Some(gas)) => {
                match interpreter.current_opcode() {
                    opcode::CREATE | opcode::CREATE2 => {
                        // set we're about to enter CREATE frame to meter its gas on first opcode
                        // inside it
                        self.gas_metering_create = Some(None)
                    }
                    opcode::STOP | opcode::RETURN | opcode::SELFDESTRUCT | opcode::REVERT => {
                        // If we are ending current execution frame, we want to just fully reset gas
                        // otherwise weird things with returning gas from a call happen
                        // ref: https://github.com/bluealloy/revm/blob/2cb991091d32330cfe085320891737186947ce5a/crates/revm/src/evm_impl.rs#L190
                        //
                        // It would be nice if we had access to the interpreter in `call_end`, as we
                        // could just do this there instead.
                        match self.gas_metering_create {
                            None | Some(None) => {
                                interpreter.gas = Gas::new(0);
                            }
                            Some(Some(gas)) => {
                                // If this was CREATE frame, set correct gas limit. This is needed
                                // because CREATE opcodes deduct additional gas for code storage,
                                // and deducted amount is compared to gas limit. If we set this to
                                // 0, the CREATE would fail with out of gas.
                                //
                                // If we however set gas limit to the limit of outer frame, it would
                                // cause a panic after erasing gas cost post-create. Reason for this
                                // is pre-create REVM records `gas_limit - (gas_limit / 64)` as gas
                                // used, and erases costs by `remaining` gas post-create.
                                // gas used ref: https://github.com/bluealloy/revm/blob/2cb991091d32330cfe085320891737186947ce5a/crates/revm/src/instructions/host.rs#L254-L258
                                // post-create erase ref: https://github.com/bluealloy/revm/blob/2cb991091d32330cfe085320891737186947ce5a/crates/revm/src/instructions/host.rs#L279
                                interpreter.gas = Gas::new(gas.limit());

                                // reset CREATE gas metering because we're about to exit its frame
                                self.gas_metering_create = None
                            }
                        }
                    }
                    _ => {
                        // if just starting with CREATE opcodes, record its inner frame gas
                        if let Some(None) = self.gas_metering_create {
                            self.gas_metering_create = Some(Some(interpreter.gas))
                        }

                        // dont monitor gas changes, keep it constant
                        interpreter.gas = gas;
                    }
                }
            }
            _ => {}
        }

        // Record writes and reads if `record` has been called
        if let Some(storage_accesses) = &mut self.accesses {
            match interpreter.current_opcode() {
                opcode::SLOAD => {
                    let key = try_or_continue!(interpreter.stack().peek(0));
                    storage_accesses
                        .reads
                        .entry(interpreter.contract().address)
                        .or_default()
                        .push(key);
                }
                opcode::SSTORE => {
                    let key = try_or_continue!(interpreter.stack().peek(0));

                    // An SSTORE does an SLOAD internally
                    storage_accesses
                        .reads
                        .entry(interpreter.contract().address)
                        .or_default()
                        .push(key);
                    storage_accesses
                        .writes
                        .entry(interpreter.contract().address)
                        .or_default()
                        .push(key);
                }
                _ => (),
            }
        }

        // If the allowed memory writes cheatcode is active at this context depth, check to see
        // if the current opcode can either mutate directly or expand memory. If the opcode at
        // the current program counter is a match, check if the modified memory lies within the
        // allowed ranges. If not, revert and fail the test.
        if let Some(ranges) = self.allowed_mem_writes.get(&data.journaled_state.depth()) {
            // The `mem_opcode_match` macro is used to match the current opcode against a list of
            // opcodes that can mutate memory (either directly or expansion via reading). If the
            // opcode is a match, the memory offsets that are being written to are checked to be
            // within the allowed ranges. If not, the test is failed and the transaction is
            // reverted. For all opcodes that can mutate memory aside from MSTORE,
            // MSTORE8, and MLOAD, the size and destination offset are on the stack, and
            // the macro expands all of these cases. For MSTORE, MSTORE8, and MLOAD, the
            // size of the memory write is implicit, so these cases are hard-coded.
            macro_rules! mem_opcode_match {
                ($(($opcode:ident, $offset_depth:expr, $size_depth:expr, $writes:expr)),* $(,)?) => {
                    match interpreter.current_opcode() {
                        ////////////////////////////////////////////////////////////////
                        //    OPERATIONS THAT CAN EXPAND/MUTATE MEMORY BY WRITING     //
                        ////////////////////////////////////////////////////////////////

                        opcode::MSTORE => {
                            // The offset of the mstore operation is at the top of the stack.
                            let offset = try_or_continue!(interpreter.stack().peek(0)).saturating_to::<u64>();

                            // If none of the allowed ranges contain [offset, offset + 32), memory has been
                            // unexpectedly mutated.
                            if !ranges.iter().any(|range| {
                                range.contains(&offset) && range.contains(&(offset + 31))
                            }) {
                                disallowed_mem_write(offset, 32, interpreter, ranges);
                                return InstructionResult::Revert
                            }
                        }
                        opcode::MSTORE8 => {
                            // The offset of the mstore8 operation is at the top of the stack.
                            let offset = try_or_continue!(interpreter.stack().peek(0)).saturating_to::<u64>();

                            // If none of the allowed ranges contain the offset, memory has been
                            // unexpectedly mutated.
                            if !ranges.iter().any(|range| range.contains(&offset)) {
                                disallowed_mem_write(offset, 1, interpreter, ranges);
                                return InstructionResult::Revert
                            }
                        }

                        ////////////////////////////////////////////////////////////////
                        //        OPERATIONS THAT CAN EXPAND MEMORY BY READING        //
                        ////////////////////////////////////////////////////////////////

                        opcode::MLOAD => {
                            // The offset of the mload operation is at the top of the stack
                            let offset = try_or_continue!(interpreter.stack().peek(0)).saturating_to::<u64>();

                            // If the offset being loaded is >= than the memory size, the
                            // memory is being expanded. If none of the allowed ranges contain
                            // [offset, offset + 32), memory has been unexpectedly mutated.
                            if offset >= interpreter.memory.len() as u64 && !ranges.iter().any(|range| {
                                range.contains(&offset) && range.contains(&(offset + 31))
                            }) {
                                disallowed_mem_write(offset, 32, interpreter, ranges);
                                return InstructionResult::Revert
                            }
                        }

                        ////////////////////////////////////////////////////////////////
                        //          OPERATIONS WITH OFFSET AND SIZE ON STACK          //
                        ////////////////////////////////////////////////////////////////

                        $(opcode::$opcode => {
                            // The destination offset of the operation is at the top of the stack.
                            let dest_offset = try_or_continue!(interpreter.stack().peek($offset_depth)).saturating_to::<u64>();

                            // The size of the data that will be copied is the third item on the stack.
                            let size = try_or_continue!(interpreter.stack().peek($size_depth)).saturating_to::<u64>();

                            // If none of the allowed ranges contain [dest_offset, dest_offset + size),
                            // memory outside of the expected ranges has been touched. If the opcode
                            // only reads from memory, this is okay as long as the memory is not expanded.
                            let fail_cond = !ranges.iter().any(|range| {
                                    range.contains(&dest_offset) &&
                                        range.contains(&(dest_offset + size.saturating_sub(1)))
                                }) && ($writes ||
                                    [dest_offset, (dest_offset + size).saturating_sub(1)].into_iter().any(|offset| {
                                        offset >= interpreter.memory.len() as u64
                                    })
                                );

                            // If the failure condition is met, set the output buffer to a revert string
                            // that gives information about the allowed ranges and revert.
                            if fail_cond {
                                disallowed_mem_write(dest_offset, size, interpreter, ranges);
                                return InstructionResult::Revert
                            }
                        })*
                        _ => ()
                    }
                }
            }

            // Check if the current opcode can write to memory, and if so, check if the memory
            // being written to is registered as safe to modify.
            mem_opcode_match!(
                (CALLDATACOPY, 0, 2, true),
                (CODECOPY, 0, 2, true),
                (RETURNDATACOPY, 0, 2, true),
                (EXTCODECOPY, 1, 3, true),
                (CALL, 5, 6, true),
                (CALLCODE, 5, 6, true),
                (STATICCALL, 4, 5, true),
                (DELEGATECALL, 4, 5, true),
                (KECCAK256, 0, 1, false),
                (LOG0, 0, 1, false),
                (LOG1, 0, 1, false),
                (LOG2, 0, 1, false),
                (LOG3, 0, 1, false),
                (LOG4, 0, 1, false),
                (CREATE, 1, 2, false),
                (CREATE2, 1, 2, false),
                (RETURN, 0, 1, false),
                (REVERT, 0, 1, false),
            )
        }

        // Record writes with sstore (and sha3) if `StartMappingRecording` has been called
        if let Some(mapping_slots) = &mut self.mapping_slots {
            mapping::step(mapping_slots, interpreter);
        }

        InstructionResult::Continue
    }

    fn step_end(
        &mut self,
        interpreter: &mut Interpreter,
        data: &mut EVMData<'_, DB>,
        eval: InstructionResult,
    ) -> InstructionResult {
        add_gas_from_cool_cheatcode(self, interpreter, data);
        eval
    }

    fn log(&mut self, _: &mut EVMData<'_, DB>, address: &Address, topics: &[B256], data: &Bytes) {
        if !self.expected_emits.is_empty() {
            expect::handle_expect_emit(self, address, topics, data);
        }

        // Stores this log if `recordLogs` has been called
        if let Some(storage_recorded_logs) = &mut self.recorded_logs {
            storage_recorded_logs.push(Vm::Log {
                topics: topics.to_vec(),
                data: data.to_vec(),
                emitter: *address,
            });
        }
    }

    fn call(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: &mut CallInputs,
    ) -> (InstructionResult, Gas, Bytes) {
        let gas = Gas::new(call.gas_limit);

        if call.contract == CHEATCODE_ADDRESS {
            return match self.apply_cheatcode(data, call) {
                Ok(retdata) => (InstructionResult::Return, gas, retdata.into()),
                Err(err) => (InstructionResult::Revert, gas, err.abi_encode().into()),
            }
        }

        if call.contract == HARDHAT_CONSOLE_ADDRESS {
            return (InstructionResult::Continue, gas, Bytes::new())
        }

        // Handle expected calls

        // Grab the different calldatas expected.
        if let Some(expected_calls_for_target) = self.expected_calls.get_mut(&(call.contract)) {
            // Match every partial/full calldata
            for (calldata, (expected, actual_count)) in expected_calls_for_target {
                // Increment actual times seen if...
                // The calldata is at most, as big as this call's input, and
                if calldata.len() <= call.input.len() &&
                    // Both calldata match, taking the length of the assumed smaller one (which will have at least the selector), and
                    *calldata == call.input[..calldata.len()] &&
                    // The value matches, if provided
                    expected
                        .value
                        .map_or(true, |value| value == call.transfer.value) &&
                    // The gas matches, if provided
                    expected.gas.map_or(true, |gas| gas == call.gas_limit) &&
                    // The minimum gas matches, if provided
                    expected.min_gas.map_or(true, |min_gas| min_gas <= call.gas_limit)
                {
                    *actual_count += 1;
                }
            }
        }

        // Handle mocked calls
        if let Some(mocks) = self.mocked_calls.get(&call.contract) {
            let ctx = MockCallDataContext {
                calldata: call.input.clone(),
                value: Some(call.transfer.value),
            };
            if let Some(return_data) = mocks.get(&ctx).or_else(|| {
                mocks
                    .iter()
                    .find(|(mock, _)| {
                        call.input.get(..mock.calldata.len()) == Some(&mock.calldata[..]) &&
                            mock.value.map_or(true, |value| value == call.transfer.value)
                    })
                    .map(|(_, v)| v)
            }) {
                return (return_data.ret_type, gas, return_data.data.clone())
            }
        }

        // Apply our prank
        if let Some(prank) = &self.prank {
            if data.journaled_state.depth() >= prank.depth &&
                call.context.caller == prank.prank_caller
            {
                let mut prank_applied = false;

                // At the target depth we set `msg.sender`
                if data.journaled_state.depth() == prank.depth {
                    call.context.caller = prank.new_caller;
                    call.transfer.source = prank.new_caller;
                    prank_applied = true;
                }

                // At the target depth, or deeper, we set `tx.origin`
                if let Some(new_origin) = prank.new_origin {
                    data.env.tx.caller = new_origin;
                    prank_applied = true;
                }

                // If prank applied for first time, then update
                if prank_applied {
                    if let Some(applied_prank) = prank.first_time_applied() {
                        self.prank = Some(applied_prank);
                    }
                }
            }
        }

        // Apply our broadcast
        if let Some(broadcast) = &self.broadcast {
            // We only apply a broadcast *to a specific depth*.
            //
            // We do this because any subsequent contract calls *must* exist on chain and
            // we only want to grab *this* call, not internal ones
            if data.journaled_state.depth() == broadcast.depth &&
                call.context.caller == broadcast.original_caller
            {
                // At the target depth we set `msg.sender` & tx.origin.
                // We are simulating the caller as being an EOA, so *both* must be set to the
                // broadcast.origin.
                data.env.tx.caller = broadcast.new_origin;

                call.context.caller = broadcast.new_origin;
                call.transfer.source = broadcast.new_origin;
                // Add a `legacy` transaction to the VecDeque. We use a legacy transaction here
                // because we only need the from, to, value, and data. We can later change this
                // into 1559, in the cli package, relatively easily once we
                // know the target chain supports EIP-1559.
                if !call.is_static {
                    if let Err(err) =
                        data.journaled_state.load_account(broadcast.new_origin, data.db)
                    {
                        return (InstructionResult::Revert, gas, Error::encode(err))
                    }

                    let is_fixed_gas_limit = check_if_fixed_gas_limit(data, call.gas_limit);

                    let account =
                        data.journaled_state.state().get_mut(&broadcast.new_origin).unwrap();

                    self.broadcastable_transactions.push_back(BroadcastableTransaction {
                        rpc: data.db.active_fork_url(),
                        transaction: TypedTransaction::Legacy(TransactionRequest {
                            from: Some(broadcast.new_origin.to_ethers()),
                            to: Some(NameOrAddress::Address(call.contract.to_ethers())),
                            value: Some(call.transfer.value.to_ethers()),
                            data: Some(call.input.clone().0.into()),
                            nonce: Some(account.info.nonce.into()),
                            gas: if is_fixed_gas_limit {
                                Some(call.gas_limit.into())
                            } else {
                                None
                            },
                            ..Default::default()
                        }),
                    });
                    debug!(target: "cheatcodes", tx=?self.broadcastable_transactions.back().unwrap(), "broadcastable call");

                    let prev = account.info.nonce;
                    account.info.nonce += 1;
                    debug!(target: "cheatcodes", address=%broadcast.new_origin, nonce=prev+1, prev, "incremented nonce");
                } else if broadcast.single_call {
                    let msg = "`staticcall`s are not allowed after `broadcast`; use `startBroadcast` instead";
                    return (InstructionResult::Revert, Gas::new(0), Error::encode(msg))
                }
            }
        }

        (InstructionResult::Continue, gas, Bytes::new())
    }

    fn call_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: &CallInputs,
        remaining_gas: Gas,
        status: InstructionResult,
        retdata: Bytes,
    ) -> (InstructionResult, Gas, Bytes) {
        if call.contract == CHEATCODE_ADDRESS || call.contract == HARDHAT_CONSOLE_ADDRESS {
            return (status, remaining_gas, retdata)
        }

        if data.journaled_state.depth() == 0 && self.skip {
            return (
                InstructionResult::Revert,
                remaining_gas,
                super::Error::from(MAGIC_SKIP).abi_encode().into(),
            )
        }

        // Clean up pranks
        if let Some(prank) = &self.prank {
            if data.journaled_state.depth() == prank.depth {
                data.env.tx.caller = prank.prank_origin;

                // Clean single-call prank once we have returned to the original depth
                if prank.single_call {
                    let _ = self.prank.take();
                }
            }
        }

        // Clean up broadcast
        if let Some(broadcast) = &self.broadcast {
            if data.journaled_state.depth() == broadcast.depth {
                data.env.tx.caller = broadcast.original_origin;

                // Clean single-call broadcast once we have returned to the original depth
                if broadcast.single_call {
                    let _ = self.broadcast.take();
                }
            }
        }

        // Handle expected reverts
        if let Some(expected_revert) = &self.expected_revert {
            if data.journaled_state.depth() <= expected_revert.depth {
                let expected_revert = std::mem::take(&mut self.expected_revert).unwrap();
                return match expect::handle_expect_revert(
                    false,
                    expected_revert.reason.as_ref(),
                    status,
                    retdata,
                ) {
                    Err(error) => {
                        trace!(expected=?expected_revert, ?error, ?status, "Expected revert mismatch");
                        (InstructionResult::Revert, remaining_gas, error.abi_encode().into())
                    }
                    Ok((_, retdata)) => (InstructionResult::Return, remaining_gas, retdata),
                }
            }
        }

        // At the end of the call,
        // we need to check if we've found all the emits.
        // We know we've found all the expected emits in the right order
        // if the queue is fully matched.
        // If it's not fully matched, then either:
        // 1. Not enough events were emitted (we'll know this because the amount of times we
        // inspected events will be less than the size of the queue) 2. The wrong events
        // were emitted (The inspected events should match the size of the queue, but still some
        // events will not be matched)

        // First, check that we're at the call depth where the emits were declared from.
        let should_check_emits = self
            .expected_emits
            .iter()
            .any(|expected| expected.depth == data.journaled_state.depth()) &&
            // Ignore staticcalls
            !call.is_static;
        if should_check_emits {
            // Not all emits were matched.
            if self.expected_emits.iter().any(|expected| !expected.found) {
                return (
                    InstructionResult::Revert,
                    remaining_gas,
                    "log != expected log".abi_encode().into(),
                )
            } else {
                // All emits were found, we're good.
                // Clear the queue, as we expect the user to declare more events for the next call
                // if they wanna match further events.
                self.expected_emits.clear()
            }
        }

        // this will ensure we don't have false positives when trying to diagnose reverts in fork
        // mode
        let diag = self.fork_revert_diagnostic.take();

        // if there's a revert and a previous call was diagnosed as fork related revert then we can
        // return a better error here
        if status == InstructionResult::Revert {
            if let Some(err) = diag {
                return (status, remaining_gas, Error::encode(err.to_error_msg(&self.labels)))
            }
        }

        // try to diagnose reverts in multi-fork mode where a call is made to an address that does
        // not exist
        if let TransactTo::Call(test_contract) = data.env.tx.transact_to {
            // if a call to a different contract than the original test contract returned with
            // `Stop` we check if the contract actually exists on the active fork
            if data.db.is_forked_mode() &&
                status == InstructionResult::Stop &&
                call.contract != test_contract
            {
                self.fork_revert_diagnostic =
                    data.db.diagnose_revert(call.contract, &data.journaled_state);
            }
        }

        // If the depth is 0, then this is the root call terminating
        if data.journaled_state.depth() == 0 {
            // If we already have a revert, we shouldn't run the below logic as it can obfuscate an
            // earlier error that happened first with unrelated information about
            // another error when using cheatcodes.
            if status == InstructionResult::Revert {
                return (status, remaining_gas, retdata)
            }

            // If there's not a revert, we can continue on to run the last logic for expect*
            // cheatcodes. Match expected calls
            for (address, calldatas) in &self.expected_calls {
                // Loop over each address, and for each address, loop over each calldata it expects.
                for (calldata, (expected, actual_count)) in calldatas {
                    // Grab the values we expect to see
                    let ExpectedCallData { gas, min_gas, value, count, call_type } = expected;

                    let failed = match call_type {
                        // If the cheatcode was called with a `count` argument,
                        // we must check that the EVM performed a CALL with this calldata exactly
                        // `count` times.
                        ExpectedCallType::Count => *count != *actual_count,
                        // If the cheatcode was called without a `count` argument,
                        // we must check that the EVM performed a CALL with this calldata at least
                        // `count` times. The amount of times to check was
                        // the amount of time the cheatcode was called.
                        ExpectedCallType::NonCount => *count > *actual_count,
                    };
                    if failed {
                        let expected_values = [
                            Some(format!("data {}", hex::encode_prefixed(calldata))),
                            value.as_ref().map(|v| format!("value {v}")),
                            gas.map(|g| format!("gas {g}")),
                            min_gas.map(|g| format!("minimum gas {g}")),
                        ]
                        .into_iter()
                        .flatten()
                        .join(", ");
                        let but = if status.is_ok() {
                            let s = if *actual_count == 1 { "" } else { "s" };
                            format!("was called {actual_count} time{s}")
                        } else {
                            "the call reverted instead; \
                             ensure you're testing the happy path when using `expectCall`"
                                .to_string()
                        };
                        let s = if *count == 1 { "" } else { "s" };
                        let msg = format!(
                            "Expected call to {address} with {expected_values} \
                             to be called {count} time{s}, but {but}"
                        );
                        return (InstructionResult::Revert, remaining_gas, Error::encode(msg))
                    }
                }
            }

            // Check if we have any leftover expected emits
            // First, if any emits were found at the root call, then we its ok and we remove them.
            self.expected_emits.retain(|expected| !expected.found);
            // If not empty, we got mismatched emits
            if !self.expected_emits.is_empty() {
                let msg = if status.is_ok() {
                    "expected an emit, but no logs were emitted afterwards. \
                     you might have mismatched events or not enough events were emitted"
                } else {
                    "expected an emit, but the call reverted instead. \
                     ensure you're testing the happy path when using `expectEmit`"
                };
                return (InstructionResult::Revert, remaining_gas, Error::encode(msg))
            }
        }

        (status, remaining_gas, retdata)
    }

    fn create(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: &mut CreateInputs,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        let gas = Gas::new(call.gas_limit);

        // allow cheatcodes from the address of the new contract
        self.allow_cheatcodes_on_create(data, call);

        // Apply our prank
        if let Some(prank) = &self.prank {
            if data.journaled_state.depth() >= prank.depth && call.caller == prank.prank_caller {
                // At the target depth we set `msg.sender`
                if data.journaled_state.depth() == prank.depth {
                    call.caller = prank.new_caller;
                }

                // At the target depth, or deeper, we set `tx.origin`
                if let Some(new_origin) = prank.new_origin {
                    data.env.tx.caller = new_origin;
                }
            }
        }

        // Apply our broadcast
        if let Some(broadcast) = &self.broadcast {
            if data.journaled_state.depth() >= broadcast.depth &&
                call.caller == broadcast.original_caller
            {
                if let Err(err) = data.journaled_state.load_account(broadcast.new_origin, data.db) {
                    return (InstructionResult::Revert, None, gas, Error::encode(err))
                }

                data.env.tx.caller = broadcast.new_origin;

                if data.journaled_state.depth() == broadcast.depth {
                    let (bytecode, to, nonce) = match process_create(
                        broadcast.new_origin,
                        call.init_code.clone(),
                        data,
                        call,
                    ) {
                        Ok(val) => val,
                        Err(err) => {
                            return (InstructionResult::Revert, None, gas, Error::encode(err))
                        }
                    };

                    let is_fixed_gas_limit = check_if_fixed_gas_limit(data, call.gas_limit);

                    self.broadcastable_transactions.push_back(BroadcastableTransaction {
                        rpc: data.db.active_fork_url(),
                        transaction: TypedTransaction::Legacy(TransactionRequest {
                            from: Some(broadcast.new_origin.to_ethers()),
                            to: to.map(|a| NameOrAddress::Address(a.to_ethers())),
                            value: Some(call.value.to_ethers()),
                            data: Some(bytecode.0.into()),
                            nonce: Some(nonce.into()),
                            gas: if is_fixed_gas_limit {
                                Some(call.gas_limit.into())
                            } else {
                                None
                            },
                            ..Default::default()
                        }),
                    });
                    let kind = match call.scheme {
                        CreateScheme::Create => "create",
                        CreateScheme::Create2 { .. } => "create2",
                    };
                    debug!(target: "cheatcodes", tx=?self.broadcastable_transactions.back().unwrap(), "broadcastable {kind}");
                }
            }
        }

        (InstructionResult::Continue, None, gas, Bytes::new())
    }

    fn create_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        _: &CreateInputs,
        status: InstructionResult,
        address: Option<Address>,
        remaining_gas: Gas,
        retdata: Bytes,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        // Clean up pranks
        if let Some(prank) = &self.prank {
            if data.journaled_state.depth() == prank.depth {
                data.env.tx.caller = prank.prank_origin;

                // Clean single-call prank once we have returned to the original depth
                if prank.single_call {
                    std::mem::take(&mut self.prank);
                }
            }
        }

        // Clean up broadcasts
        if let Some(broadcast) = &self.broadcast {
            if data.journaled_state.depth() == broadcast.depth {
                data.env.tx.caller = broadcast.original_origin;

                // Clean single-call broadcast once we have returned to the original depth
                if broadcast.single_call {
                    std::mem::take(&mut self.broadcast);
                }
            }
        }

        // Handle expected reverts
        if let Some(expected_revert) = &self.expected_revert {
            if data.journaled_state.depth() <= expected_revert.depth {
                let expected_revert = std::mem::take(&mut self.expected_revert).unwrap();
                return match expect::handle_expect_revert(
                    true,
                    expected_revert.reason.as_ref(),
                    status,
                    retdata,
                ) {
                    Ok((address, retdata)) => {
                        (InstructionResult::Return, address, remaining_gas, retdata)
                    }
                    Err(err) => {
                        (InstructionResult::Revert, None, remaining_gas, err.abi_encode().into())
                    }
                }
            }
        }

        (status, address, remaining_gas, retdata)
    }
}

/// Helper that expands memory, stores a revert string pertaining to a disallowed memory write,
/// and sets the return range to the revert string's location in memory.
fn disallowed_mem_write(
    dest_offset: u64,
    size: u64,
    interpreter: &mut Interpreter,
    ranges: &[Range<u64>],
) {
    let revert_string = format!(
        "memory write at offset 0x{:02X} of size 0x{:02X} not allowed; safe range: {}",
        dest_offset,
        size,
        ranges.iter().map(|r| format!("(0x{:02X}, 0x{:02X}]", r.start, r.end)).join(" ∪ ")
    )
    .abi_encode();
    mstore_revert_string(interpreter, &revert_string);
}

/// Expands memory, stores a revert string, and sets the return range to the revert
/// string's location in memory.
fn mstore_revert_string(interpreter: &mut Interpreter, bytes: &[u8]) {
    let starting_offset = interpreter.memory.len();
    interpreter.memory.resize(starting_offset + bytes.len());
    interpreter.memory.set_data(starting_offset, 0, bytes.len(), bytes);
    interpreter.return_offset = starting_offset;
    interpreter.return_len = interpreter.memory.len() - starting_offset
}

fn process_create<DB: DatabaseExt>(
    broadcast_sender: Address,
    bytecode: Bytes,
    data: &mut EVMData<'_, DB>,
    call: &mut CreateInputs,
) -> Result<(Bytes, Option<Address>, u64), DB::Error> {
    match call.scheme {
        CreateScheme::Create => {
            call.caller = broadcast_sender;
            Ok((bytecode, None, data.journaled_state.account(broadcast_sender).info.nonce))
        }
        CreateScheme::Create2 { salt } => {
            // Sanity checks for our CREATE2 deployer
            let info =
                &data.journaled_state.load_account(DEFAULT_CREATE2_DEPLOYER, data.db)?.0.info;
            match &info.code {
                Some(code) if code.is_empty() => return Err(DatabaseError::MissingCreate2Deployer),
                None if data.db.code_by_hash(info.code_hash)?.is_empty() => {
                    return Err(DatabaseError::MissingCreate2Deployer)
                }
                _ => {}
            }

            call.caller = DEFAULT_CREATE2_DEPLOYER;

            // We have to increment the nonce of the user address, since this create2 will be done
            // by the create2_deployer
            let account = data.journaled_state.state().get_mut(&broadcast_sender).unwrap();
            let prev = account.info.nonce;
            account.info.nonce += 1;
            debug!(target: "cheatcodes", address=%broadcast_sender, nonce=prev+1, prev, "incremented nonce in create2");

            // Proxy deployer requires the data to be `salt ++ init_code`
            let calldata = [&salt.to_be_bytes::<32>()[..], &bytecode[..]].concat();
            Ok((calldata.into(), Some(DEFAULT_CREATE2_DEPLOYER), prev))
        }
    }
}

// Determines if the gas limit on a given call was manually set in the script and should therefore
// not be overwritten by later estimations
fn check_if_fixed_gas_limit<DB: DatabaseExt>(data: &EVMData<'_, DB>, call_gas_limit: u64) -> bool {
    // If the gas limit was not set in the source code it is set to the estimated gas left at the
    // time of the call, which should be rather close to configured gas limit.
    // TODO: Find a way to reliably make this determination.
    // For example by generating it in the compilation or EVM simulation process
    U256::from(data.env.tx.gas_limit) > data.env.block.gas_limit &&
        U256::from(call_gas_limit) <= data.env.block.gas_limit
        // Transfers in forge scripts seem to be estimated at 2300 by revm leading to "Intrinsic
        // gas too low" failure when simulated on chain
        && call_gas_limit > 2300
}

/// Dispatches the cheatcode call to the appropriate function.
fn apply_dispatch<DB: DatabaseExt>(calls: &Vm::VmCalls, ccx: &mut CheatsCtxt<DB>) -> Result {
    macro_rules! match_ {
        ($($variant:ident),*) => {
            match calls {
                $(Vm::VmCalls::$variant(cheat) => crate::Cheatcode::apply_traced(cheat, ccx),)*
            }
        };
    }
    vm_calls!(match_)
}
