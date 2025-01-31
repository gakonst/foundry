//! Cheatcode EVM inspector.

use crate::{
    evm::{
        mapping::{self, MappingSlots},
        mock::{MockCallDataContext, MockCallReturnData},
        prank::Prank,
        DealRecord, GasRecord, RecordAccess,
    },
    inspector::utils::CommonCreateInput,
    script::{Broadcast, Wallets},
    test::{
        assume::AssumeNoRevert,
        expect::{
            self, ExpectedCallData, ExpectedCallTracker, ExpectedCallType, ExpectedEmitTracker,
            ExpectedRevert, ExpectedRevertKind,
        },
        revert_handlers,
    },
    utils::IgnoredTraces,
    CheatsConfig, CheatsCtxt, DynCheatcode, Error, Result,
    Vm::{self, AccountAccess},
};
use alloy_primitives::{
    hex,
    map::{AddressHashMap, HashMap},
    Address, Bytes, Log, TxKind, B256, U256,
};
use alloy_rpc_types::request::{TransactionInput, TransactionRequest};
use alloy_sol_types::{SolCall, SolInterface, SolValue};
use foundry_common::{evm::Breakpoints, TransactionMaybeSigned, SELECTOR_LEN};
use foundry_evm_core::{
    abi::Vm::stopExpectSafeMemoryCall,
    backend::{DatabaseError, DatabaseExt, RevertDiagnostic},
    constants::{CHEATCODE_ADDRESS, HARDHAT_CONSOLE_ADDRESS, MAGIC_ASSUME},
    utils::new_evm_with_existing_context,
    InspectorExt,
};
use foundry_evm_traces::{TracingInspector, TracingInspectorConfig};
use foundry_wallets::multi_wallet::MultiWallet;
use itertools::Itertools;
use proptest::test_runner::{RngAlgorithm, TestRng, TestRunner};
use rand::Rng;
use revm::{
    interpreter::{
        opcode as op, CallInputs, CallOutcome, CallScheme, CallValue, CreateInputs, CreateOutcome,
        EOFCreateInputs, EOFCreateKind, Gas, InstructionResult, Interpreter, InterpreterAction,
        InterpreterResult,
    },
    primitives::{
        BlockEnv, CreateScheme, EVMError, EvmStorageSlot, SignedAuthorization, SpecId,
        EOF_MAGIC_BYTES,
    },
    EvmContext, InnerEvmContext, Inspector,
};
use serde_json::Value;
use std::{
    cmp::max,
    collections::{BTreeMap, VecDeque},
    fs::File,
    io::BufReader,
    ops::Range,
    path::PathBuf,
    sync::Arc,
};

mod utils;

pub type Ecx<'a, 'b, 'c> = &'a mut EvmContext<&'b mut (dyn DatabaseExt + 'c)>;
pub type InnerEcx<'a, 'b, 'c> = &'a mut InnerEvmContext<&'b mut (dyn DatabaseExt + 'c)>;

/// Helper trait for obtaining complete [revm::Inspector] instance from mutable reference to
/// [Cheatcodes].
///
/// This is needed for cases when inspector itself needs mutable access to [Cheatcodes] state and
/// allows us to correctly execute arbitrary EVM frames from inside cheatcode implementations.
pub trait CheatcodesExecutor {
    /// Core trait method accepting mutable reference to [Cheatcodes] and returning
    /// [revm::Inspector].
    fn get_inspector<'a>(&'a mut self, cheats: &'a mut Cheatcodes) -> Box<dyn InspectorExt + 'a>;

    /// Obtains [revm::Evm] instance and executes the given CREATE frame.
    fn exec_create(
        &mut self,
        inputs: CreateInputs,
        ccx: &mut CheatsCtxt,
    ) -> Result<CreateOutcome, EVMError<DatabaseError>> {
        with_evm(self, ccx, |evm| {
            evm.context.evm.inner.journaled_state.depth += 1;

            // Handle EOF bytecode
            let first_frame_or_result = if evm.handler.cfg.spec_id.is_enabled_in(SpecId::OSAKA) &&
                inputs.scheme == CreateScheme::Create &&
                inputs.init_code.starts_with(&EOF_MAGIC_BYTES)
            {
                evm.handler.execution().eofcreate(
                    &mut evm.context,
                    Box::new(EOFCreateInputs::new(
                        inputs.caller,
                        inputs.value,
                        inputs.gas_limit,
                        EOFCreateKind::Tx { initdata: inputs.init_code },
                    )),
                )?
            } else {
                evm.handler.execution().create(&mut evm.context, Box::new(inputs))?
            };

            let mut result = match first_frame_or_result {
                revm::FrameOrResult::Frame(first_frame) => evm.run_the_loop(first_frame)?,
                revm::FrameOrResult::Result(result) => result,
            };

            evm.handler.execution().last_frame_return(&mut evm.context, &mut result)?;

            let outcome = match result {
                revm::FrameResult::Call(_) => unreachable!(),
                revm::FrameResult::Create(create) | revm::FrameResult::EOFCreate(create) => create,
            };

            evm.context.evm.inner.journaled_state.depth -= 1;

            Ok(outcome)
        })
    }

    fn console_log(&mut self, ccx: &mut CheatsCtxt, msg: &str) {
        self.get_inspector(ccx.state).console_log(msg);
    }

    /// Returns a mutable reference to the tracing inspector if it is available.
    fn tracing_inspector(&mut self) -> Option<&mut Option<TracingInspector>> {
        None
    }
}

/// Constructs [revm::Evm] and runs a given closure with it.
fn with_evm<E, F, O>(
    executor: &mut E,
    ccx: &mut CheatsCtxt,
    f: F,
) -> Result<O, EVMError<DatabaseError>>
where
    E: CheatcodesExecutor + ?Sized,
    F: for<'a, 'b> FnOnce(
        &mut revm::Evm<'_, &'b mut dyn InspectorExt, &'a mut dyn DatabaseExt>,
    ) -> Result<O, EVMError<DatabaseError>>,
{
    let mut inspector = executor.get_inspector(ccx.state);
    let error = std::mem::replace(&mut ccx.ecx.error, Ok(()));
    let l1_block_info = std::mem::take(&mut ccx.ecx.l1_block_info);

    let inner = revm::InnerEvmContext {
        env: ccx.ecx.env.clone(),
        journaled_state: std::mem::replace(
            &mut ccx.ecx.journaled_state,
            revm::JournaledState::new(Default::default(), Default::default()),
        ),
        db: &mut ccx.ecx.db as &mut dyn DatabaseExt,
        error,
        l1_block_info,
    };

    let mut evm = new_evm_with_existing_context(inner, &mut *inspector);

    let res = f(&mut evm)?;

    ccx.ecx.journaled_state = evm.context.evm.inner.journaled_state;
    ccx.ecx.env = evm.context.evm.inner.env;
    ccx.ecx.l1_block_info = evm.context.evm.inner.l1_block_info;
    ccx.ecx.error = evm.context.evm.inner.error;

    Ok(res)
}

/// Basic implementation of [CheatcodesExecutor] that simply returns the [Cheatcodes] instance as an
/// inspector.
#[derive(Debug, Default, Clone, Copy)]
struct TransparentCheatcodesExecutor;

impl CheatcodesExecutor for TransparentCheatcodesExecutor {
    fn get_inspector<'a>(&'a mut self, cheats: &'a mut Cheatcodes) -> Box<dyn InspectorExt + 'a> {
        Box::new(cheats)
    }
}

macro_rules! try_or_return {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(_) => return,
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
#[derive(Clone, Debug)]
pub struct BroadcastableTransaction {
    /// The optional RPC URL.
    pub rpc: Option<String>,
    /// The transaction to broadcast.
    pub transaction: TransactionMaybeSigned,
}

#[derive(Clone, Debug, Copy)]
pub struct RecordDebugStepInfo {
    /// The debug trace node index when the recording starts.
    pub start_node_idx: usize,
    /// The original tracer config when the recording starts.
    pub original_tracer_config: TracingInspectorConfig,
}

/// Holds gas metering state.
#[derive(Clone, Debug, Default)]
pub struct GasMetering {
    /// True if gas metering is paused.
    pub paused: bool,
    /// True if gas metering was resumed or reset during the test.
    /// Used to reconcile gas when frame ends (if spent less than refunded).
    pub touched: bool,
    /// True if gas metering should be reset to frame limit.
    pub reset: bool,
    /// Stores paused gas frames.
    pub paused_frames: Vec<Gas>,

    /// The group and name of the active snapshot.
    pub active_gas_snapshot: Option<(String, String)>,

    /// Cache of the amount of gas used in previous call.
    /// This is used by the `lastCallGas` cheatcode.
    pub last_call_gas: Option<crate::Vm::Gas>,

    /// True if gas recording is enabled.
    pub recording: bool,
    /// The gas used in the last frame.
    pub last_gas_used: u64,
    /// Gas records for the active snapshots.
    pub gas_records: Vec<GasRecord>,
}

impl GasMetering {
    /// Start the gas recording.
    pub fn start(&mut self) {
        self.recording = true;
    }

    /// Stop the gas recording.
    pub fn stop(&mut self) {
        self.recording = false;
    }

    /// Resume paused gas metering.
    pub fn resume(&mut self) {
        if self.paused {
            self.paused = false;
            self.touched = true;
        }
        self.paused_frames.clear();
    }

    /// Reset gas to limit.
    pub fn reset(&mut self) {
        self.paused = false;
        self.touched = true;
        self.reset = true;
        self.paused_frames.clear();
    }
}

/// Holds data about arbitrary storage.
#[derive(Clone, Debug, Default)]
pub struct ArbitraryStorage {
    /// Mapping of arbitrary storage addresses to generated values (slot, arbitrary value).
    /// (SLOADs return random value if storage slot wasn't accessed).
    /// Changed values are recorded and used to copy storage to different addresses.
    pub values: HashMap<Address, HashMap<U256, U256>>,
    /// Mapping of address with storage copied to arbitrary storage address source.
    pub copies: HashMap<Address, Address>,
}

impl ArbitraryStorage {
    /// Marks an address with arbitrary storage.
    pub fn mark_arbitrary(&mut self, address: &Address) {
        self.values.insert(*address, HashMap::default());
    }

    /// Maps an address that copies storage with the arbitrary storage address.
    pub fn mark_copy(&mut self, from: &Address, to: &Address) {
        if self.values.contains_key(from) {
            self.copies.insert(*to, *from);
        }
    }

    /// Saves arbitrary storage value for a given address:
    /// - store value in changed values cache.
    /// - update account's storage with given value.
    pub fn save(&mut self, ecx: InnerEcx, address: Address, slot: U256, data: U256) {
        self.values.get_mut(&address).expect("missing arbitrary address entry").insert(slot, data);
        if let Ok(mut account) = ecx.load_account(address) {
            account.storage.insert(slot, EvmStorageSlot::new(data));
        }
    }

    /// Copies arbitrary storage value from source address to the given target address:
    /// - if a value is present in arbitrary values cache, then update target storage and return
    ///   existing value.
    /// - if no value was yet generated for given slot, then save new value in cache and update both
    ///   source and target storages.
    pub fn copy(&mut self, ecx: InnerEcx, target: Address, slot: U256, new_value: U256) -> U256 {
        let source = self.copies.get(&target).expect("missing arbitrary copy target entry");
        let storage_cache = self.values.get_mut(source).expect("missing arbitrary source storage");
        let value = match storage_cache.get(&slot) {
            Some(value) => *value,
            None => {
                storage_cache.insert(slot, new_value);
                // Update source storage with new value.
                if let Ok(mut source_account) = ecx.load_account(*source) {
                    source_account.storage.insert(slot, EvmStorageSlot::new(new_value));
                }
                new_value
            }
        };
        // Update target storage with new value.
        if let Ok(mut target_account) = ecx.load_account(target) {
            target_account.storage.insert(slot, EvmStorageSlot::new(value));
        }
        value
    }
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
#[derive(Clone, Debug)]
pub struct Cheatcodes {
    /// The block environment
    ///
    /// Used in the cheatcode handler to overwrite the block environment separately from the
    /// execution block environment.
    pub block: Option<BlockEnv>,

    /// Currently active EIP-7702 delegation that will be consumed when building the next
    /// transaction. Set by `vm.attachDelegation()` and consumed via `.take()` during
    /// transaction construction.
    pub active_delegation: Option<SignedAuthorization>,

    /// The gas price.
    ///
    /// Used in the cheatcode handler to overwrite the gas price separately from the gas price
    /// in the execution environment.
    pub gas_price: Option<U256>,

    /// Address labels
    pub labels: AddressHashMap<String>,

    /// Prank information
    pub prank: Option<Prank>,

    /// Expected revert information
    pub expected_revert: Option<ExpectedRevert>,

    /// Assume next call can revert and discard fuzz run if it does.
    pub assume_no_revert: Option<AssumeNoRevert>,

    /// Additional diagnostic for reverts
    pub fork_revert_diagnostic: Option<RevertDiagnostic>,

    /// Recorded storage reads and writes
    pub accesses: Option<RecordAccess>,

    /// Recorded account accesses (calls, creates) organized by relative call depth, where the
    /// topmost vector corresponds to accesses at the depth at which account access recording
    /// began. Each vector in the matrix represents a list of accesses at a specific call
    /// depth. Once that call context has ended, the last vector is removed from the matrix and
    /// merged into the previous vector.
    pub recorded_account_diffs_stack: Option<Vec<Vec<AccountAccess>>>,

    /// The information of the debug step recording.
    pub record_debug_steps_info: Option<RecordDebugStepInfo>,

    /// Recorded logs
    pub recorded_logs: Option<Vec<crate::Vm::Log>>,

    /// Mocked calls
    // **Note**: inner must a BTreeMap because of special `Ord` impl for `MockCallDataContext`
    pub mocked_calls: HashMap<Address, BTreeMap<MockCallDataContext, VecDeque<MockCallReturnData>>>,

    /// Mocked functions. Maps target address to be mocked to pair of (calldata, mock address).
    pub mocked_functions: HashMap<Address, HashMap<Bytes, Address>>,

    /// Expected calls
    pub expected_calls: ExpectedCallTracker,
    /// Expected emits
    pub expected_emits: ExpectedEmitTracker,

    /// Map of context depths to memory offset ranges that may be written to within the call depth.
    pub allowed_mem_writes: HashMap<u64, Vec<Range<u64>>>,

    /// Current broadcasting information
    pub broadcast: Option<Broadcast>,

    /// Scripting based transactions
    pub broadcastable_transactions: BroadcastableTransactions,

    /// Additional, user configurable context this Inspector has access to when inspecting a call.
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

    /// Gas metering state.
    pub gas_metering: GasMetering,

    /// Contains gas snapshots made over the course of a test suite.
    // **Note**: both must a BTreeMap to ensure the order of the keys is deterministic.
    pub gas_snapshots: BTreeMap<String, BTreeMap<String, String>>,

    /// Mapping slots.
    pub mapping_slots: Option<AddressHashMap<MappingSlots>>,

    /// The current program counter.
    pub pc: usize,
    /// Breakpoints supplied by the `breakpoint` cheatcode.
    /// `char -> (address, pc)`
    pub breakpoints: Breakpoints,

    /// Optional cheatcodes `TestRunner`. Used for generating random values from uint and int
    /// strategies.
    test_runner: Option<TestRunner>,

    /// Ignored traces.
    pub ignored_traces: IgnoredTraces,

    /// Addresses with arbitrary storage.
    pub arbitrary_storage: Option<ArbitraryStorage>,

    /// Deprecated cheatcodes mapped to the reason. Used to report warnings on test results.
    pub deprecated_cheatcodes: HashMap<&'static str, Option<&'static str>>,

    /// Isolated cheatcodes. Used to report warnings on test results.
    pub isolated_cheatcodes: Vec<&'static str>,

    /// Unlocked wallets used in scripts and testing of scripts.
    pub wallets: Option<Wallets>,
}

// This is not derived because calling this in `fn new` with `..Default::default()` creates a second
// `CheatsConfig` which is unused, and inside it `ProjectPathsConfig` is relatively expensive to
// create.
impl Default for Cheatcodes {
    fn default() -> Self {
        Self::new(Arc::default())
    }
}

impl Cheatcodes {
    /// Creates a new `Cheatcodes` with the given settings.
    pub fn new(config: Arc<CheatsConfig>) -> Self {
        Self {
            fs_commit: true,
            labels: config.labels.clone(),
            config,
            block: Default::default(),
            active_delegation: Default::default(),
            gas_price: Default::default(),
            prank: Default::default(),
            expected_revert: Default::default(),
            assume_no_revert: Default::default(),
            fork_revert_diagnostic: Default::default(),
            accesses: Default::default(),
            recorded_account_diffs_stack: Default::default(),
            recorded_logs: Default::default(),
            record_debug_steps_info: Default::default(),
            mocked_calls: Default::default(),
            mocked_functions: Default::default(),
            expected_calls: Default::default(),
            expected_emits: Default::default(),
            allowed_mem_writes: Default::default(),
            broadcast: Default::default(),
            broadcastable_transactions: Default::default(),
            context: Default::default(),
            serialized_jsons: Default::default(),
            eth_deals: Default::default(),
            gas_metering: Default::default(),
            gas_snapshots: Default::default(),
            mapping_slots: Default::default(),
            pc: Default::default(),
            breakpoints: Default::default(),
            test_runner: Default::default(),
            ignored_traces: Default::default(),
            arbitrary_storage: Default::default(),
            deprecated_cheatcodes: Default::default(),
            isolated_cheatcodes: Default::default(),
            wallets: Default::default(),
        }
    }

    /// Returns the configured wallets if available, else creates a new instance.
    pub fn wallets(&mut self) -> &Wallets {
        self.wallets.get_or_insert_with(|| Wallets::new(MultiWallet::default(), None))
    }

    /// Sets the unlocked wallets.
    pub fn set_wallets(&mut self, wallets: Wallets) {
        self.wallets = Some(wallets);
    }

    /// Decodes the input data and applies the cheatcode.
    fn apply_cheatcode(
        &mut self,
        ecx: Ecx,
        call: &CallInputs,
        executor: &mut dyn CheatcodesExecutor,
    ) -> Result {
        // decode the cheatcode call
        let decoded = Vm::VmCalls::abi_decode(&call.input, false).map_err(|e| {
            if let alloy_sol_types::Error::UnknownSelector { name: _, selector } = e {
                let msg = format!(
                    "unknown cheatcode with selector {selector}; \
                     you may have a mismatch between the `Vm` interface (likely in `forge-std`) \
                     and the `forge` version"
                );
                return alloy_sol_types::Error::Other(std::borrow::Cow::Owned(msg));
            }
            e
        })?;

        let caller = call.caller;

        // ensure the caller is allowed to execute cheatcodes,
        // but only if the backend is in forking mode
        ecx.db.ensure_cheatcode_access_forking_mode(&caller)?;

        apply_dispatch(
            &decoded,
            &mut CheatsCtxt {
                state: self,
                ecx: &mut ecx.inner,
                precompiles: &mut ecx.precompiles,
                gas_limit: call.gas_limit,
                caller,
            },
            executor,
        )
    }

    /// Grants cheat code access for new contracts if the caller also has
    /// cheatcode access or the new contract is created in top most call.
    ///
    /// There may be cheatcodes in the constructor of the new contract, in order to allow them
    /// automatically we need to determine the new address.
    fn allow_cheatcodes_on_create(&self, ecx: InnerEcx, caller: Address, created_address: Address) {
        if ecx.journaled_state.depth <= 1 || ecx.db.has_cheatcode_access(&caller) {
            ecx.db.allow_cheatcode_access(created_address);
        }
    }

    /// Called when there was a revert.
    ///
    /// Cleanup any previously applied cheatcodes that altered the state in such a way that revm's
    /// revert would run into issues.
    pub fn on_revert(&mut self, ecx: Ecx) {
        trace!(deals=?self.eth_deals.len(), "rolling back deals");

        // Delay revert clean up until expected revert is handled, if set.
        if self.expected_revert.is_some() {
            return;
        }

        // we only want to apply cleanup top level
        if ecx.journaled_state.depth() > 0 {
            return;
        }

        // Roll back all previously applied deals
        // This will prevent overflow issues in revm's [`JournaledState::journal_revert`] routine
        // which rolls back any transfers.
        while let Some(record) = self.eth_deals.pop() {
            if let Some(acc) = ecx.journaled_state.state.get_mut(&record.address) {
                acc.info.balance = record.old_balance;
            }
        }
    }

    // common create functionality for both legacy and EOF.
    fn create_common<Input>(&mut self, ecx: Ecx, mut input: Input) -> Option<CreateOutcome>
    where
        Input: CommonCreateInput,
    {
        let ecx = &mut ecx.inner;
        let gas = Gas::new(input.gas_limit());

        // Apply our prank
        if let Some(prank) = &self.prank {
            if ecx.journaled_state.depth() >= prank.depth && input.caller() == prank.prank_caller {
                // At the target depth we set `msg.sender`
                if ecx.journaled_state.depth() == prank.depth {
                    input.set_caller(prank.new_caller);
                }

                // At the target depth, or deeper, we set `tx.origin`
                if let Some(new_origin) = prank.new_origin {
                    ecx.env.tx.caller = new_origin;
                }
            }
        }

        // Apply our broadcast
        if let Some(broadcast) = &self.broadcast {
            if ecx.journaled_state.depth() >= broadcast.depth &&
                input.caller() == broadcast.original_caller
            {
                if let Err(err) =
                    ecx.journaled_state.load_account(broadcast.new_origin, &mut ecx.db)
                {
                    return Some(CreateOutcome {
                        result: InterpreterResult {
                            result: InstructionResult::Revert,
                            output: Error::encode(err),
                            gas,
                        },
                        address: None,
                    });
                }

                ecx.env.tx.caller = broadcast.new_origin;

                if ecx.journaled_state.depth() == broadcast.depth {
                    input.set_caller(broadcast.new_origin);
                    let is_fixed_gas_limit = check_if_fixed_gas_limit(ecx, input.gas_limit());

                    let account = &ecx.journaled_state.state()[&broadcast.new_origin];
                    self.broadcastable_transactions.push_back(BroadcastableTransaction {
                        rpc: ecx.db.active_fork_url(),
                        transaction: TransactionRequest {
                            from: Some(broadcast.new_origin),
                            to: None,
                            value: Some(input.value()),
                            input: TransactionInput::new(input.init_code()),
                            nonce: Some(account.info.nonce),
                            gas: if is_fixed_gas_limit { Some(input.gas_limit()) } else { None },
                            ..Default::default()
                        }
                        .into(),
                    });

                    input.log_debug(self, &input.scheme().unwrap_or(CreateScheme::Create));
                }
            }
        }

        // Allow cheatcodes from the address of the new contract
        let address = input.allow_cheatcodes(self, ecx);

        // If `recordAccountAccesses` has been called, record the create
        if let Some(recorded_account_diffs_stack) = &mut self.recorded_account_diffs_stack {
            recorded_account_diffs_stack.push(vec![AccountAccess {
                chainInfo: crate::Vm::ChainInfo {
                    forkId: ecx.db.active_fork_id().unwrap_or_default(),
                    chainId: U256::from(ecx.env.cfg.chain_id),
                },
                accessor: input.caller(),
                account: address,
                kind: crate::Vm::AccountAccessKind::Create,
                initialized: true,
                oldBalance: U256::ZERO, // updated on (eof)create_end
                newBalance: U256::ZERO, // updated on (eof)create_end
                value: input.value(),
                data: input.init_code(),
                reverted: false,
                deployedCode: Bytes::new(), // updated on (eof)create_end
                storageAccesses: vec![],    // updated on (eof)create_end
                depth: ecx.journaled_state.depth(),
            }]);
        }

        None
    }

    // common create_end functionality for both legacy and EOF.
    fn create_end_common(&mut self, ecx: Ecx, mut outcome: CreateOutcome) -> CreateOutcome
where {
        let ecx = &mut ecx.inner;

        // Clean up pranks
        if let Some(prank) = &self.prank {
            if ecx.journaled_state.depth() == prank.depth {
                ecx.env.tx.caller = prank.prank_origin;

                // Clean single-call prank once we have returned to the original depth
                if prank.single_call {
                    std::mem::take(&mut self.prank);
                }
            }
        }

        // Clean up broadcasts
        if let Some(broadcast) = &self.broadcast {
            if ecx.journaled_state.depth() == broadcast.depth {
                ecx.env.tx.caller = broadcast.original_origin;

                // Clean single-call broadcast once we have returned to the original depth
                if broadcast.single_call {
                    std::mem::take(&mut self.broadcast);
                }
            }
        }

        // Handle expected reverts
        if let Some(expected_revert) = &self.expected_revert {
            if ecx.journaled_state.depth() <= expected_revert.depth &&
                matches!(expected_revert.kind, ExpectedRevertKind::Default)
            {
                let mut expected_revert = std::mem::take(&mut self.expected_revert).unwrap();
                return match revert_handlers::handle_expect_revert(
                    false,
                    true,
                    self.config.internal_expect_revert,
                    &expected_revert,
                    outcome.result.result,
                    outcome.result.output.clone(),
                    &self.config.available_artifacts,
                ) {
                    Ok((address, retdata)) => {
                        expected_revert.actual_count += 1;
                        if expected_revert.actual_count < expected_revert.count {
                            self.expected_revert = Some(expected_revert.clone());
                        }

                        outcome.result.result = InstructionResult::Return;
                        outcome.result.output = retdata;
                        outcome.address = address;
                        outcome
                    }
                    Err(err) => {
                        outcome.result.result = InstructionResult::Revert;
                        outcome.result.output = err.abi_encode().into();
                        outcome
                    }
                };
            }
        }

        // If `startStateDiffRecording` has been called, update the `reverted` status of the
        // previous call depth's recorded accesses, if any
        if let Some(recorded_account_diffs_stack) = &mut self.recorded_account_diffs_stack {
            // The root call cannot be recorded.
            if ecx.journaled_state.depth() > 0 {
                let mut last_depth =
                    recorded_account_diffs_stack.pop().expect("missing CREATE account accesses");
                // Update the reverted status of all deeper calls if this call reverted, in
                // accordance with EVM behavior
                if outcome.result.is_revert() {
                    last_depth.iter_mut().for_each(|element| {
                        element.reverted = true;
                        element
                            .storageAccesses
                            .iter_mut()
                            .for_each(|storage_access| storage_access.reverted = true);
                    })
                }
                let create_access = last_depth.first_mut().expect("empty AccountAccesses");
                // Assert that we're at the correct depth before recording post-create state
                // changes. Depending on what depth the cheat was called at, there
                // may not be any pending calls to update if execution has
                // percolated up to a higher depth.
                if create_access.depth == ecx.journaled_state.depth() {
                    debug_assert_eq!(
                        create_access.kind as u8,
                        crate::Vm::AccountAccessKind::Create as u8
                    );
                    if let Some(address) = outcome.address {
                        if let Ok(created_acc) =
                            ecx.journaled_state.load_account(address, &mut ecx.db)
                        {
                            create_access.newBalance = created_acc.info.balance;
                            create_access.deployedCode =
                                created_acc.info.code.clone().unwrap_or_default().original_bytes();
                        }
                    }
                }
                // Merge the last depth's AccountAccesses into the AccountAccesses at the current
                // depth, or push them back onto the pending vector if higher depths were not
                // recorded. This preserves ordering of accesses.
                if let Some(last) = recorded_account_diffs_stack.last_mut() {
                    last.append(&mut last_depth);
                } else {
                    recorded_account_diffs_stack.push(last_depth);
                }
            }
        }
        outcome
    }

    pub fn call_with_executor(
        &mut self,
        ecx: Ecx,
        call: &mut CallInputs,
        executor: &mut impl CheatcodesExecutor,
    ) -> Option<CallOutcome> {
        let gas = Gas::new(call.gas_limit);

        // At the root call to test function or script `run()`/`setUp()` functions, we are
        // decreasing sender nonce to ensure that it matches on-chain nonce once we start
        // broadcasting.
        if ecx.journaled_state.depth == 0 {
            let sender = ecx.env.tx.caller;
            let account = match super::evm::journaled_account(ecx, sender) {
                Ok(account) => account,
                Err(err) => {
                    return Some(CallOutcome {
                        result: InterpreterResult {
                            result: InstructionResult::Revert,
                            output: err.abi_encode().into(),
                            gas,
                        },
                        memory_offset: call.return_memory_offset.clone(),
                    })
                }
            };
            let prev = account.info.nonce;
            account.info.nonce = prev.saturating_sub(1);

            trace!(target: "cheatcodes", %sender, nonce=account.info.nonce, prev, "corrected nonce");
        }

        if call.target_address == CHEATCODE_ADDRESS {
            return match self.apply_cheatcode(ecx, call, executor) {
                Ok(retdata) => Some(CallOutcome {
                    result: InterpreterResult {
                        result: InstructionResult::Return,
                        output: retdata.into(),
                        gas,
                    },
                    memory_offset: call.return_memory_offset.clone(),
                }),
                Err(err) => Some(CallOutcome {
                    result: InterpreterResult {
                        result: InstructionResult::Revert,
                        output: err.abi_encode().into(),
                        gas,
                    },
                    memory_offset: call.return_memory_offset.clone(),
                }),
            };
        }

        let ecx = &mut ecx.inner;

        if call.target_address == HARDHAT_CONSOLE_ADDRESS {
            return None;
        }

        // Handle expected calls

        // Grab the different calldatas expected.
        if let Some(expected_calls_for_target) = self.expected_calls.get_mut(&call.bytecode_address)
        {
            // Match every partial/full calldata
            for (calldata, (expected, actual_count)) in expected_calls_for_target {
                // Increment actual times seen if...
                // The calldata is at most, as big as this call's input, and
                if calldata.len() <= call.input.len() &&
                    // Both calldata match, taking the length of the assumed smaller one (which will have at least the selector), and
                    *calldata == call.input[..calldata.len()] &&
                    // The value matches, if provided
                    expected
                        .value.is_none_or(|value| Some(value) == call.transfer_value()) &&
                    // The gas matches, if provided
                    expected.gas.is_none_or(|gas| gas == call.gas_limit) &&
                    // The minimum gas matches, if provided
                    expected.min_gas.is_none_or(|min_gas| min_gas <= call.gas_limit)
                {
                    *actual_count += 1;
                }
            }
        }

        // Handle mocked calls
        if let Some(mocks) = self.mocked_calls.get_mut(&call.bytecode_address) {
            let ctx =
                MockCallDataContext { calldata: call.input.clone(), value: call.transfer_value() };

            if let Some(return_data_queue) = match mocks.get_mut(&ctx) {
                Some(queue) => Some(queue),
                None => mocks
                    .iter_mut()
                    .find(|(mock, _)| {
                        call.input.get(..mock.calldata.len()) == Some(&mock.calldata[..]) &&
                            mock.value.is_none_or(|value| Some(value) == call.transfer_value())
                    })
                    .map(|(_, v)| v),
            } {
                if let Some(return_data) = if return_data_queue.len() == 1 {
                    // If the mocked calls stack has a single element in it, don't empty it
                    return_data_queue.front().map(|x| x.to_owned())
                } else {
                    // Else, we pop the front element
                    return_data_queue.pop_front()
                } {
                    return Some(CallOutcome {
                        result: InterpreterResult {
                            result: return_data.ret_type,
                            output: return_data.data,
                            gas,
                        },
                        memory_offset: call.return_memory_offset.clone(),
                    });
                }
            }
        }

        // Apply our prank
        if let Some(prank) = &self.prank {
            // Apply delegate call, `call.caller`` will not equal `prank.prank_caller`
            if let CallScheme::DelegateCall | CallScheme::ExtDelegateCall = call.scheme {
                if prank.delegate_call {
                    call.target_address = prank.new_caller;
                    call.caller = prank.new_caller;
                    let acc = ecx.journaled_state.account(prank.new_caller);
                    call.value = CallValue::Apparent(acc.info.balance);
                    if let Some(new_origin) = prank.new_origin {
                        ecx.env.tx.caller = new_origin;
                    }
                }
            }

            if ecx.journaled_state.depth() >= prank.depth && call.caller == prank.prank_caller {
                let mut prank_applied = false;

                // At the target depth we set `msg.sender`
                if ecx.journaled_state.depth() == prank.depth {
                    call.caller = prank.new_caller;
                    prank_applied = true;
                }

                // At the target depth, or deeper, we set `tx.origin`
                if let Some(new_origin) = prank.new_origin {
                    ecx.env.tx.caller = new_origin;
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
            if ecx.journaled_state.depth() == broadcast.depth &&
                call.caller == broadcast.original_caller
            {
                // At the target depth we set `msg.sender` & tx.origin.
                // We are simulating the caller as being an EOA, so *both* must be set to the
                // broadcast.origin.
                ecx.env.tx.caller = broadcast.new_origin;

                call.caller = broadcast.new_origin;
                // Add a `legacy` transaction to the VecDeque. We use a legacy transaction here
                // because we only need the from, to, value, and data. We can later change this
                // into 1559, in the cli package, relatively easily once we
                // know the target chain supports EIP-1559.
                if !call.is_static {
                    if let Err(err) = ecx.load_account(broadcast.new_origin) {
                        return Some(CallOutcome {
                            result: InterpreterResult {
                                result: InstructionResult::Revert,
                                output: Error::encode(err),
                                gas,
                            },
                            memory_offset: call.return_memory_offset.clone(),
                        });
                    }

                    let is_fixed_gas_limit = check_if_fixed_gas_limit(ecx, call.gas_limit);

                    let account =
                        ecx.journaled_state.state().get_mut(&broadcast.new_origin).unwrap();

                    let mut tx_req = TransactionRequest {
                        from: Some(broadcast.new_origin),
                        to: Some(TxKind::from(Some(call.target_address))),
                        value: call.transfer_value(),
                        input: TransactionInput::new(call.input.clone()),
                        nonce: Some(account.info.nonce),
                        chain_id: Some(ecx.env.cfg.chain_id),
                        gas: if is_fixed_gas_limit { Some(call.gas_limit) } else { None },
                        ..Default::default()
                    };

                    if let Some(auth_list) = self.active_delegation.take() {
                        tx_req.authorization_list = Some(vec![auth_list]);
                    } else {
                        tx_req.authorization_list = None;
                    }

                    self.broadcastable_transactions.push_back(BroadcastableTransaction {
                        rpc: ecx.db.active_fork_url(),
                        transaction: tx_req.into(),
                    });
                    debug!(target: "cheatcodes", tx=?self.broadcastable_transactions.back().unwrap(), "broadcastable call");

                    // Explicitly increment nonce if calls are not isolated.
                    if !self.config.evm_opts.isolate {
                        let prev = account.info.nonce;
                        account.info.nonce += 1;
                        debug!(target: "cheatcodes", address=%broadcast.new_origin, nonce=prev+1, prev, "incremented nonce");
                    }
                } else if broadcast.single_call {
                    let msg =
                    "`staticcall`s are not allowed after `broadcast`; use `startBroadcast` instead";
                    return Some(CallOutcome {
                        result: InterpreterResult {
                            result: InstructionResult::Revert,
                            output: Error::encode(msg),
                            gas,
                        },
                        memory_offset: call.return_memory_offset.clone(),
                    });
                }
            }
        }

        // Record called accounts if `startStateDiffRecording` has been called
        if let Some(recorded_account_diffs_stack) = &mut self.recorded_account_diffs_stack {
            // Determine if account is "initialized," ie, it has a non-zero balance, a non-zero
            // nonce, a non-zero KECCAK_EMPTY codehash, or non-empty code
            let initialized;
            let old_balance;
            if let Ok(acc) = ecx.load_account(call.target_address) {
                initialized = acc.info.exists();
                old_balance = acc.info.balance;
            } else {
                initialized = false;
                old_balance = U256::ZERO;
            }
            let kind = match call.scheme {
                CallScheme::Call => crate::Vm::AccountAccessKind::Call,
                CallScheme::CallCode => crate::Vm::AccountAccessKind::CallCode,
                CallScheme::DelegateCall => crate::Vm::AccountAccessKind::DelegateCall,
                CallScheme::StaticCall => crate::Vm::AccountAccessKind::StaticCall,
                CallScheme::ExtCall => crate::Vm::AccountAccessKind::Call,
                CallScheme::ExtStaticCall => crate::Vm::AccountAccessKind::StaticCall,
                CallScheme::ExtDelegateCall => crate::Vm::AccountAccessKind::DelegateCall,
            };
            // Record this call by pushing it to a new pending vector; all subsequent calls at
            // that depth will be pushed to the same vector. When the call ends, the
            // RecordedAccountAccess (and all subsequent RecordedAccountAccesses) will be
            // updated with the revert status of this call, since the EVM does not mark accounts
            // as "warm" if the call from which they were accessed is reverted
            recorded_account_diffs_stack.push(vec![AccountAccess {
                chainInfo: crate::Vm::ChainInfo {
                    forkId: ecx.db.active_fork_id().unwrap_or_default(),
                    chainId: U256::from(ecx.env.cfg.chain_id),
                },
                accessor: call.caller,
                account: call.bytecode_address,
                kind,
                initialized,
                oldBalance: old_balance,
                newBalance: U256::ZERO, // updated on call_end
                value: call.call_value(),
                data: call.input.clone(),
                reverted: false,
                deployedCode: Bytes::new(),
                storageAccesses: vec![], // updated on step
                depth: ecx.journaled_state.depth(),
            }]);
        }

        None
    }

    pub fn rng(&mut self) -> &mut impl Rng {
        self.test_runner().rng()
    }

    pub fn test_runner(&mut self) -> &mut TestRunner {
        self.test_runner.get_or_insert_with(|| match self.config.seed {
            Some(seed) => TestRunner::new_with_rng(
                proptest::test_runner::Config::default(),
                TestRng::from_seed(RngAlgorithm::ChaCha, &seed.to_be_bytes::<32>()),
            ),
            None => TestRunner::new(proptest::test_runner::Config::default()),
        })
    }

    /// Returns existing or set a default `ArbitraryStorage` option.
    /// Used by `setArbitraryStorage` cheatcode to track addresses with arbitrary storage.
    pub fn arbitrary_storage(&mut self) -> &mut ArbitraryStorage {
        self.arbitrary_storage.get_or_insert_with(ArbitraryStorage::default)
    }

    /// Whether the given address has arbitrary storage.
    pub fn has_arbitrary_storage(&self, address: &Address) -> bool {
        match &self.arbitrary_storage {
            Some(storage) => storage.values.contains_key(address),
            None => false,
        }
    }

    /// Whether the given address is a copy of an address with arbitrary storage.
    pub fn is_arbitrary_storage_copy(&self, address: &Address) -> bool {
        match &self.arbitrary_storage {
            Some(storage) => storage.copies.contains_key(address),
            None => false,
        }
    }
}

impl Inspector<&mut dyn DatabaseExt> for Cheatcodes {
    #[inline]
    fn initialize_interp(&mut self, interpreter: &mut Interpreter, ecx: Ecx) {
        // When the first interpreter is initialized we've circumvented the balance and gas checks,
        // so we apply our actual block data with the correct fees and all.
        if let Some(block) = self.block.take() {
            ecx.env.block = block;
        }
        if let Some(gas_price) = self.gas_price.take() {
            ecx.env.tx.gas_price = gas_price;
        }

        // Record gas for current frame.
        if self.gas_metering.paused {
            self.gas_metering.paused_frames.push(interpreter.gas);
        }

        // `expectRevert`: track the max call depth during `expectRevert`
        if let Some(expected) = &mut self.expected_revert {
            expected.max_depth = max(ecx.journaled_state.depth(), expected.max_depth);
        }
    }

    #[inline]
    fn step(&mut self, interpreter: &mut Interpreter, ecx: Ecx) {
        self.pc = interpreter.program_counter();

        // `pauseGasMetering`: pause / resume interpreter gas.
        if self.gas_metering.paused {
            self.meter_gas(interpreter);
        }

        // `resetGasMetering`: reset interpreter gas.
        if self.gas_metering.reset {
            self.meter_gas_reset(interpreter);
        }

        // `record`: record storage reads and writes.
        if self.accesses.is_some() {
            self.record_accesses(interpreter);
        }

        // `startStateDiffRecording`: record granular ordered storage accesses.
        if self.recorded_account_diffs_stack.is_some() {
            self.record_state_diffs(interpreter, ecx);
        }

        // `expectSafeMemory`: check if the current opcode is allowed to interact with memory.
        if !self.allowed_mem_writes.is_empty() {
            self.check_mem_opcodes(interpreter, ecx.journaled_state.depth());
        }

        // `startMappingRecording`: record SSTORE and KECCAK256.
        if let Some(mapping_slots) = &mut self.mapping_slots {
            mapping::step(mapping_slots, interpreter);
        }

        // `snapshotGas*`: take a snapshot of the current gas.
        if self.gas_metering.recording {
            self.meter_gas_record(interpreter, ecx);
        }
    }

    #[inline]
    fn step_end(&mut self, interpreter: &mut Interpreter, ecx: Ecx) {
        if self.gas_metering.paused {
            self.meter_gas_end(interpreter);
        }

        if self.gas_metering.touched {
            self.meter_gas_check(interpreter);
        }

        // `setArbitraryStorage` and `copyStorage`: add arbitrary values to storage.
        if self.arbitrary_storage.is_some() {
            self.arbitrary_storage_end(interpreter, ecx);
        }
    }

    fn log(&mut self, interpreter: &mut Interpreter, _ecx: Ecx, log: &Log) {
        if !self.expected_emits.is_empty() {
            expect::handle_expect_emit(self, log, interpreter);
        }

        // `recordLogs`
        if let Some(storage_recorded_logs) = &mut self.recorded_logs {
            storage_recorded_logs.push(Vm::Log {
                topics: log.data.topics().to_vec(),
                data: log.data.data.clone(),
                emitter: log.address,
            });
        }
    }

    fn call(&mut self, ecx: Ecx, inputs: &mut CallInputs) -> Option<CallOutcome> {
        Self::call_with_executor(self, ecx, inputs, &mut TransparentCheatcodesExecutor)
    }

    fn call_end(&mut self, ecx: Ecx, call: &CallInputs, mut outcome: CallOutcome) -> CallOutcome {
        let ecx = &mut ecx.inner;
        let cheatcode_call = call.target_address == CHEATCODE_ADDRESS ||
            call.target_address == HARDHAT_CONSOLE_ADDRESS;

        // Clean up pranks/broadcasts if it's not a cheatcode call end. We shouldn't do
        // it for cheatcode calls because they are not applied for cheatcodes in the `call` hook.
        // This should be placed before the revert handling, because we might exit early there
        if !cheatcode_call {
            // Clean up pranks
            if let Some(prank) = &self.prank {
                if ecx.journaled_state.depth() == prank.depth {
                    ecx.env.tx.caller = prank.prank_origin;

                    // Clean single-call prank once we have returned to the original depth
                    if prank.single_call {
                        let _ = self.prank.take();
                    }
                }
            }

            // Clean up broadcast
            if let Some(broadcast) = &self.broadcast {
                if ecx.journaled_state.depth() == broadcast.depth {
                    ecx.env.tx.caller = broadcast.original_origin;

                    // Clean single-call broadcast once we have returned to the original depth
                    if broadcast.single_call {
                        let _ = self.broadcast.take();
                    }
                }
            }
        }

        // Handle assume no revert cheatcode.
        if let Some(assume_no_revert) = &mut self.assume_no_revert {
            // Record current reverter address before processing the expect revert if call reverted,
            // expect revert is set with expected reverter address and no actual reverter set yet.
            if outcome.result.is_revert() && assume_no_revert.reverted_by.is_none() {
                assume_no_revert.reverted_by = Some(call.target_address);
            }
            // allow multiple cheatcode calls at the same depth
            if ecx.journaled_state.depth() <= assume_no_revert.depth && !cheatcode_call {
                // Discard run if we're at the same depth as cheatcode, call reverted, and no
                // specific reason was supplied
                if outcome.result.is_revert() {
                    let assume_no_revert = std::mem::take(&mut self.assume_no_revert).unwrap();
                    return match revert_handlers::handle_assume_no_revert(
                        &assume_no_revert,
                        outcome.result.result,
                        &outcome.result.output,
                        &self.config.available_artifacts,
                    ) {
                        // if result is Ok, it was an anticipated revert; return an "assume" error
                        // to reject this run
                        Ok(_) => {
                            outcome.result.output = Error::from(MAGIC_ASSUME).abi_encode().into();
                            outcome
                        }
                        // if result is Error, it was an unanticipated revert; should revert
                        // normally
                        Err(error) => {
                            trace!(expected=?assume_no_revert, ?error, status=?outcome.result.result, "Expected revert mismatch");
                            outcome.result.result = InstructionResult::Revert;
                            outcome.result.output = error.abi_encode().into();
                            outcome
                        }
                    }
                } else {
                    // Call didn't revert, reset `assume_no_revert` state.
                    self.assume_no_revert = None;
                    return outcome;
                }
            }
        }

        // Handle expected reverts.
        if let Some(expected_revert) = &mut self.expected_revert {
            // Record current reverter address and call scheme before processing the expect revert
            // if call reverted.
            if outcome.result.is_revert() {
                // Record current reverter address if expect revert is set with expected reverter
                // address and no actual reverter was set yet or if we're expecting more than one
                // revert.
                if expected_revert.reverter.is_some() &&
                    (expected_revert.reverted_by.is_none() || expected_revert.count > 1)
                {
                    expected_revert.reverted_by = Some(call.target_address);
                }
            }

            if ecx.journaled_state.depth() <= expected_revert.depth {
                let needs_processing = match expected_revert.kind {
                    ExpectedRevertKind::Default => !cheatcode_call,
                    // `pending_processing` == true means that we're in the `call_end` hook for
                    // `vm.expectCheatcodeRevert` and shouldn't expect revert here
                    ExpectedRevertKind::Cheatcode { pending_processing } => {
                        cheatcode_call && !pending_processing
                    }
                };

                if needs_processing {
                    let mut expected_revert = std::mem::take(&mut self.expected_revert).unwrap();
                    return match revert_handlers::handle_expect_revert(
                        cheatcode_call,
                        false,
                        self.config.internal_expect_revert,
                        &expected_revert,
                        outcome.result.result,
                        outcome.result.output.clone(),
                        &self.config.available_artifacts,
                    ) {
                        Err(error) => {
                            trace!(expected=?expected_revert, ?error, status=?outcome.result.result, "Expected revert mismatch");
                            outcome.result.result = InstructionResult::Revert;
                            outcome.result.output = error.abi_encode().into();
                            outcome
                        }
                        Ok((_, retdata)) => {
                            expected_revert.actual_count += 1;
                            if expected_revert.actual_count < expected_revert.count {
                                self.expected_revert = Some(expected_revert.clone());
                            }
                            outcome.result.result = InstructionResult::Return;
                            outcome.result.output = retdata;
                            outcome
                        }
                    };
                }

                // Flip `pending_processing` flag for cheatcode revert expectations, marking that
                // we've exited the `expectCheatcodeRevert` call scope
                if let ExpectedRevertKind::Cheatcode { pending_processing } =
                    &mut self.expected_revert.as_mut().unwrap().kind
                {
                    *pending_processing = false;
                }
            }
        }

        // Exit early for calls to cheatcodes as other logic is not relevant for cheatcode
        // invocations
        if cheatcode_call {
            return outcome;
        }

        // Record the gas usage of the call, this allows the `lastCallGas` cheatcode to
        // retrieve the gas usage of the last call.
        let gas = outcome.result.gas;
        self.gas_metering.last_call_gas = Some(crate::Vm::Gas {
            gasLimit: gas.limit(),
            gasTotalUsed: gas.spent(),
            gasMemoryUsed: 0,
            gasRefunded: gas.refunded(),
            gasRemaining: gas.remaining(),
        });

        // If `startStateDiffRecording` has been called, update the `reverted` status of the
        // previous call depth's recorded accesses, if any
        if let Some(recorded_account_diffs_stack) = &mut self.recorded_account_diffs_stack {
            // The root call cannot be recorded.
            if ecx.journaled_state.depth() > 0 {
                let mut last_recorded_depth =
                    recorded_account_diffs_stack.pop().expect("missing CALL account accesses");
                // Update the reverted status of all deeper calls if this call reverted, in
                // accordance with EVM behavior
                if outcome.result.is_revert() {
                    last_recorded_depth.iter_mut().for_each(|element| {
                        element.reverted = true;
                        element
                            .storageAccesses
                            .iter_mut()
                            .for_each(|storage_access| storage_access.reverted = true);
                    })
                }
                let call_access = last_recorded_depth.first_mut().expect("empty AccountAccesses");
                // Assert that we're at the correct depth before recording post-call state changes.
                // Depending on the depth the cheat was called at, there may not be any pending
                // calls to update if execution has percolated up to a higher depth.
                if call_access.depth == ecx.journaled_state.depth() {
                    if let Ok(acc) = ecx.load_account(call.target_address) {
                        debug_assert!(access_is_call(call_access.kind));
                        call_access.newBalance = acc.info.balance;
                    }
                }
                // Merge the last depth's AccountAccesses into the AccountAccesses at the current
                // depth, or push them back onto the pending vector if higher depths were not
                // recorded. This preserves ordering of accesses.
                if let Some(last) = recorded_account_diffs_stack.last_mut() {
                    last.append(&mut last_recorded_depth);
                } else {
                    recorded_account_diffs_stack.push(last_recorded_depth);
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
            .any(|(expected, _)| expected.depth == ecx.journaled_state.depth()) &&
            // Ignore staticcalls
            !call.is_static;
        if should_check_emits {
            let expected_counts = self
                .expected_emits
                .iter()
                .filter_map(|(expected, count_map)| {
                    let count = match expected.address {
                        Some(emitter) => match count_map.get(&emitter) {
                            Some(log_count) => expected
                                .log
                                .as_ref()
                                .map(|l| log_count.count(l))
                                .unwrap_or_else(|| log_count.count_unchecked()),
                            None => 0,
                        },
                        None => match &expected.log {
                            Some(log) => count_map.values().map(|logs| logs.count(log)).sum(),
                            None => count_map.values().map(|logs| logs.count_unchecked()).sum(),
                        },
                    };

                    if count != expected.count {
                        Some((expected, count))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            // Not all emits were matched.
            if self.expected_emits.iter().any(|(expected, _)| !expected.found) {
                outcome.result.result = InstructionResult::Revert;
                outcome.result.output = "log != expected log".abi_encode().into();
                return outcome;
            }

            if !expected_counts.is_empty() {
                let msg = if outcome.result.is_ok() {
                    let (expected, count) = expected_counts.first().unwrap();
                    format!("log emitted {count} times, expected {}", expected.count)
                } else {
                    "expected an emit, but the call reverted instead. \
                     ensure you're testing the happy path when using `expectEmit`"
                        .to_string()
                };

                outcome.result.result = InstructionResult::Revert;
                outcome.result.output = Error::encode(msg);
                return outcome;
            }

            // All emits were found, we're good.
            // Clear the queue, as we expect the user to declare more events for the next call
            // if they wanna match further events.
            self.expected_emits.clear()
        }

        // this will ensure we don't have false positives when trying to diagnose reverts in fork
        // mode
        let diag = self.fork_revert_diagnostic.take();

        // if there's a revert and a previous call was diagnosed as fork related revert then we can
        // return a better error here
        if outcome.result.is_revert() {
            if let Some(err) = diag {
                outcome.result.output = Error::encode(err.to_error_msg(&self.labels));
                return outcome;
            }
        }

        // try to diagnose reverts in multi-fork mode where a call is made to an address that does
        // not exist
        if let TxKind::Call(test_contract) = ecx.env.tx.transact_to {
            // if a call to a different contract than the original test contract returned with
            // `Stop` we check if the contract actually exists on the active fork
            if ecx.db.is_forked_mode() &&
                outcome.result.result == InstructionResult::Stop &&
                call.target_address != test_contract
            {
                self.fork_revert_diagnostic =
                    ecx.db.diagnose_revert(call.target_address, &ecx.journaled_state);
            }
        }

        // If the depth is 0, then this is the root call terminating
        if ecx.journaled_state.depth() == 0 {
            // If we already have a revert, we shouldn't run the below logic as it can obfuscate an
            // earlier error that happened first with unrelated information about
            // another error when using cheatcodes.
            if outcome.result.is_revert() {
                return outcome;
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
                        let but = if outcome.result.is_ok() {
                            let s = if *actual_count == 1 { "" } else { "s" };
                            format!("was called {actual_count} time{s}")
                        } else {
                            "the call reverted instead; \
                             ensure you're testing the happy path when using `expectCall`"
                                .to_string()
                        };
                        let s = if *count == 1 { "" } else { "s" };
                        let msg = format!(
                            "expected call to {address} with {expected_values} \
                             to be called {count} time{s}, but {but}"
                        );
                        outcome.result.result = InstructionResult::Revert;
                        outcome.result.output = Error::encode(msg);

                        return outcome;
                    }
                }
            }
            // Check if we have any leftover expected emits
            // First, if any emits were found at the root call, then we its ok and we remove them.
            self.expected_emits.retain(|(expected, _)| expected.count > 0 && !expected.found);
            // If not empty, we got mismatched emits
            if !self.expected_emits.is_empty() {
                let msg = if outcome.result.is_ok() {
                    "expected an emit, but no logs were emitted afterwards. \
                     you might have mismatched events or not enough events were emitted"
                } else {
                    "expected an emit, but the call reverted instead. \
                     ensure you're testing the happy path when using `expectEmit`"
                };
                outcome.result.result = InstructionResult::Revert;
                outcome.result.output = Error::encode(msg);
                return outcome;
            }
        }

        outcome
    }

    fn create(&mut self, ecx: Ecx, call: &mut CreateInputs) -> Option<CreateOutcome> {
        self.create_common(ecx, call)
    }

    fn create_end(
        &mut self,
        ecx: Ecx,
        _call: &CreateInputs,
        outcome: CreateOutcome,
    ) -> CreateOutcome {
        self.create_end_common(ecx, outcome)
    }

    fn eofcreate(&mut self, ecx: Ecx, call: &mut EOFCreateInputs) -> Option<CreateOutcome> {
        self.create_common(ecx, call)
    }

    fn eofcreate_end(
        &mut self,
        ecx: Ecx,
        _call: &EOFCreateInputs,
        outcome: CreateOutcome,
    ) -> CreateOutcome {
        self.create_end_common(ecx, outcome)
    }
}

impl InspectorExt for Cheatcodes {
    fn should_use_create2_factory(&mut self, ecx: Ecx, inputs: &mut CreateInputs) -> bool {
        if let CreateScheme::Create2 { .. } = inputs.scheme {
            let target_depth = if let Some(prank) = &self.prank {
                prank.depth
            } else if let Some(broadcast) = &self.broadcast {
                broadcast.depth
            } else {
                1
            };

            ecx.journaled_state.depth() == target_depth &&
                (self.broadcast.is_some() || self.config.always_use_create_2_factory)
        } else {
            false
        }
    }

    fn create2_deployer(&self) -> Address {
        self.config.evm_opts.create2_deployer
    }
}

impl Cheatcodes {
    #[cold]
    fn meter_gas(&mut self, interpreter: &mut Interpreter) {
        if let Some(paused_gas) = self.gas_metering.paused_frames.last() {
            // Keep gas constant if paused.
            interpreter.gas = *paused_gas;
        } else {
            // Record frame paused gas.
            self.gas_metering.paused_frames.push(interpreter.gas);
        }
    }

    #[cold]
    fn meter_gas_record(&mut self, interpreter: &mut Interpreter, ecx: Ecx) {
        if matches!(interpreter.instruction_result, InstructionResult::Continue) {
            self.gas_metering.gas_records.iter_mut().for_each(|record| {
                if ecx.journaled_state.depth() == record.depth {
                    // Skip the first opcode of the first call frame as it includes the gas cost of
                    // creating the snapshot.
                    if self.gas_metering.last_gas_used != 0 {
                        let gas_diff =
                            interpreter.gas.spent().saturating_sub(self.gas_metering.last_gas_used);
                        record.gas_used = record.gas_used.saturating_add(gas_diff);
                    }

                    // Update `last_gas_used` to the current spent gas for the next iteration to
                    // compare against.
                    self.gas_metering.last_gas_used = interpreter.gas.spent();
                }
            });
        }
    }

    #[cold]
    fn meter_gas_end(&mut self, interpreter: &mut Interpreter) {
        // Remove recorded gas if we exit frame.
        if will_exit(interpreter.instruction_result) {
            self.gas_metering.paused_frames.pop();
        }
    }

    #[cold]
    fn meter_gas_reset(&mut self, interpreter: &mut Interpreter) {
        interpreter.gas = Gas::new(interpreter.gas().limit());
        self.gas_metering.reset = false;
    }

    #[cold]
    fn meter_gas_check(&mut self, interpreter: &mut Interpreter) {
        if will_exit(interpreter.instruction_result) {
            // Reset gas if spent is less than refunded.
            // This can happen if gas was paused / resumed or reset.
            // https://github.com/foundry-rs/foundry/issues/4370
            if interpreter.gas.spent() <
                u64::try_from(interpreter.gas.refunded()).unwrap_or_default()
            {
                interpreter.gas = Gas::new(interpreter.gas.limit());
            }
        }
    }

    /// Generates or copies arbitrary values for storage slots.
    /// Invoked in inspector `step_end` (when the current opcode is not executed), if current opcode
    /// to execute is `SLOAD` and storage slot is cold.
    /// Ensures that in next step (when `SLOAD` opcode is executed) an arbitrary value is returned:
    /// - copies the existing arbitrary storage value (or the new generated one if no value in
    ///   cache) from mapped source address to the target address.
    /// - generates arbitrary value and saves it in target address storage.
    #[cold]
    fn arbitrary_storage_end(&mut self, interpreter: &mut Interpreter, ecx: Ecx) {
        let (key, target_address) = if interpreter.current_opcode() == op::SLOAD {
            (try_or_return!(interpreter.stack().peek(0)), interpreter.contract().target_address)
        } else {
            return
        };

        let Ok(value) = ecx.sload(target_address, key) else {
            return;
        };

        if value.is_cold && value.data.is_zero() {
            if self.has_arbitrary_storage(&target_address) {
                let arbitrary_value = self.rng().gen();
                self.arbitrary_storage.as_mut().unwrap().save(
                    &mut ecx.inner,
                    target_address,
                    key,
                    arbitrary_value,
                );
            } else if self.is_arbitrary_storage_copy(&target_address) {
                let arbitrary_value = self.rng().gen();
                self.arbitrary_storage.as_mut().unwrap().copy(
                    &mut ecx.inner,
                    target_address,
                    key,
                    arbitrary_value,
                );
            }
        }
    }

    /// Records storage slots reads and writes.
    #[cold]
    fn record_accesses(&mut self, interpreter: &mut Interpreter) {
        let Some(access) = &mut self.accesses else { return };
        match interpreter.current_opcode() {
            op::SLOAD => {
                let key = try_or_return!(interpreter.stack().peek(0));
                access.record_read(interpreter.contract().target_address, key);
            }
            op::SSTORE => {
                let key = try_or_return!(interpreter.stack().peek(0));
                access.record_write(interpreter.contract().target_address, key);
            }
            _ => {}
        }
    }

    #[cold]
    fn record_state_diffs(&mut self, interpreter: &mut Interpreter, ecx: Ecx) {
        let Some(account_accesses) = &mut self.recorded_account_diffs_stack else { return };
        match interpreter.current_opcode() {
            op::SELFDESTRUCT => {
                // Ensure that we're not selfdestructing a context recording was initiated on
                let Some(last) = account_accesses.last_mut() else { return };

                // get previous balance and initialized status of the target account
                let target = try_or_return!(interpreter.stack().peek(0));
                let target = Address::from_word(B256::from(target));
                let (initialized, old_balance) = ecx
                    .load_account(target)
                    .map(|account| (account.info.exists(), account.info.balance))
                    .unwrap_or_default();

                // load balance of this account
                let value = ecx
                    .balance(interpreter.contract().target_address)
                    .map(|b| b.data)
                    .unwrap_or(U256::ZERO);

                // register access for the target account
                last.push(crate::Vm::AccountAccess {
                    chainInfo: crate::Vm::ChainInfo {
                        forkId: ecx.db.active_fork_id().unwrap_or_default(),
                        chainId: U256::from(ecx.env.cfg.chain_id),
                    },
                    accessor: interpreter.contract().target_address,
                    account: target,
                    kind: crate::Vm::AccountAccessKind::SelfDestruct,
                    initialized,
                    oldBalance: old_balance,
                    newBalance: old_balance + value,
                    value,
                    data: Bytes::new(),
                    reverted: false,
                    deployedCode: Bytes::new(),
                    storageAccesses: vec![],
                    depth: ecx.journaled_state.depth(),
                });
            }

            op::SLOAD => {
                let Some(last) = account_accesses.last_mut() else { return };

                let key = try_or_return!(interpreter.stack().peek(0));
                let address = interpreter.contract().target_address;

                // Try to include present value for informational purposes, otherwise assume
                // it's not set (zero value)
                let mut present_value = U256::ZERO;
                // Try to load the account and the slot's present value
                if ecx.load_account(address).is_ok() {
                    if let Ok(previous) = ecx.sload(address, key) {
                        present_value = previous.data;
                    }
                }
                let access = crate::Vm::StorageAccess {
                    account: interpreter.contract().target_address,
                    slot: key.into(),
                    isWrite: false,
                    previousValue: present_value.into(),
                    newValue: present_value.into(),
                    reverted: false,
                };
                append_storage_access(last, access, ecx.journaled_state.depth());
            }
            op::SSTORE => {
                let Some(last) = account_accesses.last_mut() else { return };

                let key = try_or_return!(interpreter.stack().peek(0));
                let value = try_or_return!(interpreter.stack().peek(1));
                let address = interpreter.contract().target_address;
                // Try to load the account and the slot's previous value, otherwise, assume it's
                // not set (zero value)
                let mut previous_value = U256::ZERO;
                if ecx.load_account(address).is_ok() {
                    if let Ok(previous) = ecx.sload(address, key) {
                        previous_value = previous.data;
                    }
                }

                let access = crate::Vm::StorageAccess {
                    account: address,
                    slot: key.into(),
                    isWrite: true,
                    previousValue: previous_value.into(),
                    newValue: value.into(),
                    reverted: false,
                };
                append_storage_access(last, access, ecx.journaled_state.depth());
            }

            // Record account accesses via the EXT family of opcodes
            op::EXTCODECOPY | op::EXTCODESIZE | op::EXTCODEHASH | op::BALANCE => {
                let kind = match interpreter.current_opcode() {
                    op::EXTCODECOPY => crate::Vm::AccountAccessKind::Extcodecopy,
                    op::EXTCODESIZE => crate::Vm::AccountAccessKind::Extcodesize,
                    op::EXTCODEHASH => crate::Vm::AccountAccessKind::Extcodehash,
                    op::BALANCE => crate::Vm::AccountAccessKind::Balance,
                    _ => unreachable!(),
                };
                let address =
                    Address::from_word(B256::from(try_or_return!(interpreter.stack().peek(0))));
                let initialized;
                let balance;
                if let Ok(acc) = ecx.load_account(address) {
                    initialized = acc.info.exists();
                    balance = acc.info.balance;
                } else {
                    initialized = false;
                    balance = U256::ZERO;
                }
                let account_access = crate::Vm::AccountAccess {
                    chainInfo: crate::Vm::ChainInfo {
                        forkId: ecx.db.active_fork_id().unwrap_or_default(),
                        chainId: U256::from(ecx.env.cfg.chain_id),
                    },
                    accessor: interpreter.contract().target_address,
                    account: address,
                    kind,
                    initialized,
                    oldBalance: balance,
                    newBalance: balance,
                    value: U256::ZERO,
                    data: Bytes::new(),
                    reverted: false,
                    deployedCode: Bytes::new(),
                    storageAccesses: vec![],
                    depth: ecx.journaled_state.depth(),
                };
                // Record the EXT* call as an account access at the current depth
                // (future storage accesses will be recorded in a new "Resume" context)
                if let Some(last) = account_accesses.last_mut() {
                    last.push(account_access);
                } else {
                    account_accesses.push(vec![account_access]);
                }
            }
            _ => {}
        }
    }

    /// Checks to see if the current opcode can either mutate directly or expand memory.
    ///
    /// If the opcode at the current program counter is a match, check if the modified memory lies
    /// within the allowed ranges. If not, revert and fail the test.
    #[cold]
    fn check_mem_opcodes(&self, interpreter: &mut Interpreter, depth: u64) {
        let Some(ranges) = self.allowed_mem_writes.get(&depth) else {
            return;
        };

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

                    op::MSTORE => {
                        // The offset of the mstore operation is at the top of the stack.
                        let offset = try_or_return!(interpreter.stack().peek(0)).saturating_to::<u64>();

                        // If none of the allowed ranges contain [offset, offset + 32), memory has been
                        // unexpectedly mutated.
                        if !ranges.iter().any(|range| {
                            range.contains(&offset) && range.contains(&(offset + 31))
                        }) {
                            // SPECIAL CASE: When the compiler attempts to store the selector for
                            // `stopExpectSafeMemory`, this is allowed. It will do so at the current free memory
                            // pointer, which could have been updated to the exclusive upper bound during
                            // execution.
                            let value = try_or_return!(interpreter.stack().peek(1)).to_be_bytes::<32>();
                            if value[..SELECTOR_LEN] == stopExpectSafeMemoryCall::SELECTOR {
                                return
                            }

                            disallowed_mem_write(offset, 32, interpreter, ranges);
                            return
                        }
                    }
                    op::MSTORE8 => {
                        // The offset of the mstore8 operation is at the top of the stack.
                        let offset = try_or_return!(interpreter.stack().peek(0)).saturating_to::<u64>();

                        // If none of the allowed ranges contain the offset, memory has been
                        // unexpectedly mutated.
                        if !ranges.iter().any(|range| range.contains(&offset)) {
                            disallowed_mem_write(offset, 1, interpreter, ranges);
                            return
                        }
                    }

                    ////////////////////////////////////////////////////////////////
                    //        OPERATIONS THAT CAN EXPAND MEMORY BY READING        //
                    ////////////////////////////////////////////////////////////////

                    op::MLOAD => {
                        // The offset of the mload operation is at the top of the stack
                        let offset = try_or_return!(interpreter.stack().peek(0)).saturating_to::<u64>();

                        // If the offset being loaded is >= than the memory size, the
                        // memory is being expanded. If none of the allowed ranges contain
                        // [offset, offset + 32), memory has been unexpectedly mutated.
                        if offset >= interpreter.shared_memory.len() as u64 && !ranges.iter().any(|range| {
                            range.contains(&offset) && range.contains(&(offset + 31))
                        }) {
                            disallowed_mem_write(offset, 32, interpreter, ranges);
                            return
                        }
                    }

                    ////////////////////////////////////////////////////////////////
                    //          OPERATIONS WITH OFFSET AND SIZE ON STACK          //
                    ////////////////////////////////////////////////////////////////

                    op::CALL => {
                        // The destination offset of the operation is the fifth element on the stack.
                        let dest_offset = try_or_return!(interpreter.stack().peek(5)).saturating_to::<u64>();

                        // The size of the data that will be copied is the sixth element on the stack.
                        let size = try_or_return!(interpreter.stack().peek(6)).saturating_to::<u64>();

                        // If none of the allowed ranges contain [dest_offset, dest_offset + size),
                        // memory outside of the expected ranges has been touched. If the opcode
                        // only reads from memory, this is okay as long as the memory is not expanded.
                        let fail_cond = !ranges.iter().any(|range| {
                            range.contains(&dest_offset) &&
                                range.contains(&(dest_offset + size.saturating_sub(1)))
                        });

                        // If the failure condition is met, set the output buffer to a revert string
                        // that gives information about the allowed ranges and revert.
                        if fail_cond {
                            // SPECIAL CASE: When a call to `stopExpectSafeMemory` is performed, this is allowed.
                            // It allocated calldata at the current free memory pointer, and will attempt to read
                            // from this memory region to perform the call.
                            let to = Address::from_word(try_or_return!(interpreter.stack().peek(1)).to_be_bytes::<32>().into());
                            if to == CHEATCODE_ADDRESS {
                                let args_offset = try_or_return!(interpreter.stack().peek(3)).saturating_to::<usize>();
                                let args_size = try_or_return!(interpreter.stack().peek(4)).saturating_to::<usize>();
                                let memory_word = interpreter.shared_memory.slice(args_offset, args_size);
                                if memory_word[..SELECTOR_LEN] == stopExpectSafeMemoryCall::SELECTOR {
                                    return
                                }
                            }

                            disallowed_mem_write(dest_offset, size, interpreter, ranges);
                            return
                        }
                    }

                    $(op::$opcode => {
                        // The destination offset of the operation.
                        let dest_offset = try_or_return!(interpreter.stack().peek($offset_depth)).saturating_to::<u64>();

                        // The size of the data that will be copied.
                        let size = try_or_return!(interpreter.stack().peek($size_depth)).saturating_to::<u64>();

                        // If none of the allowed ranges contain [dest_offset, dest_offset + size),
                        // memory outside of the expected ranges has been touched. If the opcode
                        // only reads from memory, this is okay as long as the memory is not expanded.
                        let fail_cond = !ranges.iter().any(|range| {
                                range.contains(&dest_offset) &&
                                    range.contains(&(dest_offset + size.saturating_sub(1)))
                            }) && ($writes ||
                                [dest_offset, (dest_offset + size).saturating_sub(1)].into_iter().any(|offset| {
                                    offset >= interpreter.shared_memory.len() as u64
                                })
                            );

                        // If the failure condition is met, set the output buffer to a revert string
                        // that gives information about the allowed ranges and revert.
                        if fail_cond {
                            disallowed_mem_write(dest_offset, size, interpreter, ranges);
                            return
                        }
                    })*

                    _ => {}
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
        );
    }
}

/// Helper that expands memory, stores a revert string pertaining to a disallowed memory write,
/// and sets the return range to the revert string's location in memory.
///
/// This will set the interpreter's next action to a return with the revert string as the output.
/// And trigger a revert.
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
        ranges.iter().map(|r| format!("(0x{:02X}, 0x{:02X}]", r.start, r.end)).join(" U ")
    );

    interpreter.instruction_result = InstructionResult::Revert;
    interpreter.next_action = InterpreterAction::Return {
        result: InterpreterResult {
            output: Error::encode(revert_string),
            gas: interpreter.gas,
            result: InstructionResult::Revert,
        },
    };
}

// Determines if the gas limit on a given call was manually set in the script and should therefore
// not be overwritten by later estimations
fn check_if_fixed_gas_limit(ecx: InnerEcx, call_gas_limit: u64) -> bool {
    // If the gas limit was not set in the source code it is set to the estimated gas left at the
    // time of the call, which should be rather close to configured gas limit.
    // TODO: Find a way to reliably make this determination.
    // For example by generating it in the compilation or EVM simulation process
    U256::from(ecx.env.tx.gas_limit) > ecx.env.block.gas_limit &&
        U256::from(call_gas_limit) <= ecx.env.block.gas_limit
        // Transfers in forge scripts seem to be estimated at 2300 by revm leading to "Intrinsic
        // gas too low" failure when simulated on chain
        && call_gas_limit > 2300
}

/// Returns true if the kind of account access is a call.
fn access_is_call(kind: crate::Vm::AccountAccessKind) -> bool {
    matches!(
        kind,
        crate::Vm::AccountAccessKind::Call |
            crate::Vm::AccountAccessKind::StaticCall |
            crate::Vm::AccountAccessKind::CallCode |
            crate::Vm::AccountAccessKind::DelegateCall
    )
}

/// Appends an AccountAccess that resumes the recording of the current context.
fn append_storage_access(
    last: &mut Vec<AccountAccess>,
    storage_access: crate::Vm::StorageAccess,
    storage_depth: u64,
) {
    // Assert that there's an existing record for the current context.
    if !last.is_empty() && last.first().unwrap().depth < storage_depth {
        // Three cases to consider:
        // 1. If there hasn't been a context switch since the start of this context, then add the
        //    storage access to the current context record.
        // 2. If there's an existing Resume record, then add the storage access to it.
        // 3. Otherwise, create a new Resume record based on the current context.
        if last.len() == 1 {
            last.first_mut().unwrap().storageAccesses.push(storage_access);
        } else {
            let last_record = last.last_mut().unwrap();
            if last_record.kind as u8 == crate::Vm::AccountAccessKind::Resume as u8 {
                last_record.storageAccesses.push(storage_access);
            } else {
                let entry = last.first().unwrap();
                let resume_record = crate::Vm::AccountAccess {
                    chainInfo: crate::Vm::ChainInfo {
                        forkId: entry.chainInfo.forkId,
                        chainId: entry.chainInfo.chainId,
                    },
                    accessor: entry.accessor,
                    account: entry.account,
                    kind: crate::Vm::AccountAccessKind::Resume,
                    initialized: entry.initialized,
                    storageAccesses: vec![storage_access],
                    reverted: entry.reverted,
                    // The remaining fields are defaults
                    oldBalance: U256::ZERO,
                    newBalance: U256::ZERO,
                    value: U256::ZERO,
                    data: Bytes::new(),
                    deployedCode: Bytes::new(),
                    depth: entry.depth,
                };
                last.push(resume_record);
            }
        }
    }
}

/// Dispatches the cheatcode call to the appropriate function.
fn apply_dispatch(
    calls: &Vm::VmCalls,
    ccx: &mut CheatsCtxt,
    executor: &mut dyn CheatcodesExecutor,
) -> Result {
    let cheat = calls_as_dyn_cheatcode(calls);

    let _guard = debug_span!(target: "cheatcodes", "apply", id = %cheat.id()).entered();
    trace!(target: "cheatcodes", cheat = ?cheat.as_debug(), "applying");

    if let spec::Status::Deprecated(replacement) = *cheat.status() {
        ccx.state.deprecated_cheatcodes.insert(cheat.signature(), replacement);
    }

    if !ccx.state.config.evm_opts.isolate && *cheat.requires() == spec::Requires::Isolation {
        ccx.state.isolated_cheatcodes.push(cheat.signature());
    }

    // Apply the cheatcode.
    let mut result = cheat.dyn_apply(ccx, executor);

    // Format the error message to include the cheatcode name.
    if let Err(e) = &mut result {
        if e.is_str() {
            let name = cheat.name();
            // Skip showing the cheatcode name for:
            // - assertions: too verbose, and can already be inferred from the error message
            // - `rpcUrl`: forge-std relies on it in `getChainWithUpdatedRpcUrl`
            if !name.contains("assert") && name != "rpcUrl" {
                *e = fmt_err!("vm.{name}: {e}");
            }
        }
    }

    trace!(
        target: "cheatcodes",
        return = %match &result {
            Ok(b) => hex::encode(b),
            Err(e) => e.to_string(),
        }
    );

    result
}

fn calls_as_dyn_cheatcode(calls: &Vm::VmCalls) -> &dyn DynCheatcode {
    macro_rules! as_dyn {
        ($($variant:ident),*) => {
            match calls {
                $(Vm::VmCalls::$variant(cheat) => cheat,)*
            }
        };
    }
    vm_calls!(as_dyn)
}

/// Helper function to check if frame execution will exit.
fn will_exit(ir: InstructionResult) -> bool {
    !matches!(ir, InstructionResult::Continue | InstructionResult::CallOrCreate)
}
