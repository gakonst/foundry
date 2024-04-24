use super::{
    Cheatcodes, CheatsConfig, ChiselState, CoverageCollector, Debugger, Fuzzer, LogCollector,
    StackSnapshotType, TracingInspector, TracingInspectorConfig,
};
use alloy_primitives::{Address, Bytes, Log, U256};
use foundry_evm_core::{
    backend::{update_state, DatabaseExt},
    debug::DebugArena,
    InspectorExt,
};
use foundry_evm_coverage::HitMaps;
use foundry_evm_traces::CallTraceArena;
use revm::{
    inspectors::CustomPrintTracer,
    interpreter::{
        CallInputs, CallOutcome, CallScheme, CreateInputs, CreateOutcome, Gas, InstructionResult,
        Interpreter, InterpreterResult,
    },
    primitives::{BlockEnv, Env, EnvWithHandlerCfg, ExecutionResult, Output, TransactTo},
    DatabaseCommit, EvmContext, Inspector,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Debug, Default)]
#[must_use = "builders do nothing unless you call `build` on them"]
pub struct InspectorStackBuilder {
    /// The block environment.
    ///
    /// Used in the cheatcode handler to overwrite the block environment separately from the
    /// execution block environment.
    pub block: Option<BlockEnv>,
    /// The gas price.
    ///
    /// Used in the cheatcode handler to overwrite the gas price separately from the gas price
    /// in the execution environment.
    pub gas_price: Option<U256>,
    /// The cheatcodes config.
    pub cheatcodes: Option<Arc<CheatsConfig>>,
    /// The fuzzer inspector and its state, if it exists.
    pub fuzzer: Option<Fuzzer>,
    /// Whether to enable tracing.
    pub trace: Option<bool>,
    /// Whether to enable the debugger.
    pub debug: Option<bool>,
    /// Whether logs should be collected.
    pub logs: Option<bool>,
    /// Whether coverage info should be collected.
    pub coverage: Option<bool>,
    /// Whether to print all opcode traces into the console. Useful for debugging the EVM.
    pub print: Option<bool>,
    /// The chisel state inspector.
    pub chisel_state: Option<usize>,
    /// Whether to enable call isolation.
    /// In isolation mode all top-level calls are executed as a separate transaction in a separate
    /// EVM context, enabling more precise gas accounting and transaction state changes.
    pub enable_isolation: bool,
}

impl InspectorStackBuilder {
    /// Create a new inspector stack builder.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the block environment.
    #[inline]
    pub fn block(mut self, block: BlockEnv) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the gas price.
    #[inline]
    pub fn gas_price(mut self, gas_price: U256) -> Self {
        self.gas_price = Some(gas_price);
        self
    }

    /// Enable cheatcodes with the given config.
    #[inline]
    pub fn cheatcodes(mut self, config: Arc<CheatsConfig>) -> Self {
        self.cheatcodes = Some(config);
        self
    }

    /// Set the fuzzer inspector.
    #[inline]
    pub fn fuzzer(mut self, fuzzer: Fuzzer) -> Self {
        self.fuzzer = Some(fuzzer);
        self
    }

    /// Set the Chisel inspector.
    #[inline]
    pub fn chisel_state(mut self, final_pc: usize) -> Self {
        self.chisel_state = Some(final_pc);
        self
    }

    /// Set whether to collect logs.
    #[inline]
    pub fn logs(mut self, yes: bool) -> Self {
        self.logs = Some(yes);
        self
    }

    /// Set whether to collect coverage information.
    #[inline]
    pub fn coverage(mut self, yes: bool) -> Self {
        self.coverage = Some(yes);
        self
    }

    /// Set whether to enable the debugger.
    #[inline]
    pub fn debug(mut self, yes: bool) -> Self {
        self.debug = Some(yes);
        self
    }

    /// Set whether to enable the trace printer.
    #[inline]
    pub fn print(mut self, yes: bool) -> Self {
        self.print = Some(yes);
        self
    }

    /// Set whether to enable the tracer.
    #[inline]
    pub fn trace(mut self, yes: bool) -> Self {
        self.trace = Some(yes);
        self
    }

    /// Set whether to enable the call isolation.
    /// For description of call isolation, see [`InspectorStack::enable_isolation`].
    #[inline]
    pub fn enable_isolation(mut self, yes: bool) -> Self {
        self.enable_isolation = yes;
        self
    }

    /// Builds the stack of inspectors to use when transacting/committing on the EVM.
    ///
    /// See also [`revm::Evm::inspect_ref`] and [`revm::Evm::commit_ref`].
    pub fn build(self) -> InspectorStack {
        let Self {
            block,
            gas_price,
            cheatcodes,
            fuzzer,
            trace,
            debug,
            logs,
            coverage,
            print,
            chisel_state,
            enable_isolation,
        } = self;
        let mut stack = InspectorStack::new();

        // inspectors
        if let Some(config) = cheatcodes {
            stack.set_cheatcodes(Cheatcodes::new(config));
        }
        if let Some(fuzzer) = fuzzer {
            stack.set_fuzzer(fuzzer);
        }
        if let Some(chisel_state) = chisel_state {
            stack.set_chisel(chisel_state);
        }
        stack.collect_coverage(coverage.unwrap_or(false));
        stack.collect_logs(logs.unwrap_or(true));
        stack.enable_debugger(debug.unwrap_or(false));
        stack.print(print.unwrap_or(false));
        stack.tracing(trace.unwrap_or(false));

        stack.enable_isolation(enable_isolation);

        // environment, must come after all of the inspectors
        if let Some(block) = block {
            stack.set_block(&block);
        }
        if let Some(gas_price) = gas_price {
            stack.set_gas_price(gas_price);
        }

        stack
    }
}

/// Helper macro to call the same method on multiple inspectors without resorting to dynamic
/// dispatch.
#[macro_export]
macro_rules! call_inspectors {
    ([$($inspector:expr),+ $(,)?], |$id:ident $(,)?| $call:expr $(,)?) => {{$(
        if let Some($id) = $inspector {
            $call
        }
    )+}}
}

/// Same as [call_inspectors] macro, but with depth adjustment for isolated execution.
macro_rules! call_inspectors_adjust_depth {
    (#[no_ret] [$($inspector:expr),+ $(,)?], |$id:ident $(,)?| $call:expr, $self:ident, $data:ident $(,)?) => {
        if $self.in_inner_context {
            $data.journaled_state.depth += 1;
            $(
                if let Some($id) = $inspector {
                    $call
                }
            )+
            $data.journaled_state.depth -= 1;
        } else {
            $(
                if let Some($id) = $inspector {
                    $call
                }
            )+
        }
    };
    ([$($inspector:expr),+ $(,)?], |$id:ident $(,)?| $call:expr, $self:ident, $data:ident $(,)?) => {
        if $self.in_inner_context {
            $data.journaled_state.depth += 1;
            $(
                if let Some($id) = $inspector {
                    if let Some(result) = $call {
                        $data.journaled_state.depth -= 1;
                        return result;
                    }
                }
            )+
            $data.journaled_state.depth -= 1;
        } else {
            $(
                if let Some($id) = $inspector {
                    if let Some(result) = $call {
                        return result;
                    }
                }
            )+
        }
    };
}

/// The collected results of [`InspectorStack`].
pub struct InspectorData {
    pub logs: Vec<Log>,
    pub labels: HashMap<Address, String>,
    pub traces: Option<CallTraceArena>,
    pub debug: Option<DebugArena>,
    pub coverage: Option<HitMaps>,
    pub cheatcodes: Option<Cheatcodes>,
    pub chisel_state: Option<(Vec<U256>, Vec<u8>, InstructionResult)>,
}

/// Contains data about the state of outer/main EVM which created and invoked the inner EVM context.
/// Used to adjust EVM state while in inner context.
///
/// We need this to avoid breaking changes due to EVM behavior differences in isolated vs
/// non-isolated mode. For descriptions and workarounds for those changes see: https://github.com/foundry-rs/foundry/pull/7186#issuecomment-1959102195
#[derive(Debug, Clone)]
pub struct InnerContextData {
    /// The sender of the inner EVM context.
    /// It is also an origin of the transaction that created the inner EVM context.
    sender: Address,
    /// Nonce of the sender before invocation of the inner EVM context.
    original_sender_nonce: u64,
    /// Origin of the transaction in the outer EVM context.
    original_origin: Address,
    /// Whether the inner context was created by a CREATE transaction.
    is_create: bool,
}

/// An inspector that calls multiple inspectors in sequence.
///
/// If a call to an inspector returns a value other than [InstructionResult::Continue] (or
/// equivalent) the remaining inspectors are not called.
#[derive(Clone, Debug, Default)]
pub struct InspectorStack {
    pub cheatcodes: Option<Cheatcodes>,
    pub chisel_state: Option<ChiselState>,
    pub coverage: Option<CoverageCollector>,
    pub debugger: Option<Debugger>,
    pub fuzzer: Option<Fuzzer>,
    pub log_collector: Option<LogCollector>,
    pub printer: Option<CustomPrintTracer>,
    pub tracer: Option<TracingInspector>,
    pub enable_isolation: bool,

    /// Flag marking if we are in the inner EVM context.
    pub in_inner_context: bool,
    pub inner_context_data: Option<InnerContextData>,
}

impl InspectorStack {
    /// Creates a new inspector stack.
    ///
    /// Note that the stack is empty by default, and you must add inspectors to it.
    /// This is done by calling the `set_*` methods on the stack directly, or by building the stack
    /// with [`InspectorStack`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set variables from an environment for the relevant inspectors.
    #[inline]
    pub fn set_env(&mut self, env: &Env) {
        self.set_block(&env.block);
        self.set_gas_price(env.tx.gas_price);
    }

    /// Sets the block for the relevant inspectors.
    #[inline]
    pub fn set_block(&mut self, block: &BlockEnv) {
        if let Some(cheatcodes) = &mut self.cheatcodes {
            cheatcodes.block = Some(block.clone());
        }
    }

    /// Sets the gas price for the relevant inspectors.
    #[inline]
    pub fn set_gas_price(&mut self, gas_price: U256) {
        if let Some(cheatcodes) = &mut self.cheatcodes {
            cheatcodes.gas_price = Some(gas_price);
        }
    }

    /// Set the cheatcodes inspector.
    #[inline]
    pub fn set_cheatcodes(&mut self, cheatcodes: Cheatcodes) {
        self.cheatcodes = Some(cheatcodes);
    }

    /// Set the fuzzer inspector.
    #[inline]
    pub fn set_fuzzer(&mut self, fuzzer: Fuzzer) {
        self.fuzzer = Some(fuzzer);
    }

    /// Set the Chisel inspector.
    #[inline]
    pub fn set_chisel(&mut self, final_pc: usize) {
        self.chisel_state = Some(ChiselState::new(final_pc));
    }

    /// Set whether to enable the coverage collector.
    #[inline]
    pub fn collect_coverage(&mut self, yes: bool) {
        self.coverage = yes.then(Default::default);
    }

    /// Set whether to enable the debugger.
    #[inline]
    pub fn enable_debugger(&mut self, yes: bool) {
        self.debugger = yes.then(Default::default);
    }

    /// Set whether to enable call isolation.
    #[inline]
    pub fn enable_isolation(&mut self, yes: bool) {
        self.enable_isolation = yes;
    }

    /// Set whether to enable the log collector.
    #[inline]
    pub fn collect_logs(&mut self, yes: bool) {
        self.log_collector = yes.then(Default::default);
    }

    /// Set whether to enable the trace printer.
    #[inline]
    pub fn print(&mut self, yes: bool) {
        self.printer = yes.then(Default::default);
    }

    /// Set whether to enable the tracer.
    #[inline]
    pub fn tracing(&mut self, yes: bool) {
        self.tracer = yes.then(|| {
            TracingInspector::new(TracingInspectorConfig {
                record_steps: false,
                record_memory_snapshots: false,
                record_stack_snapshots: StackSnapshotType::None,
                record_state_diff: false,
                exclude_precompile_calls: false,
                record_logs: true,
            })
        });
    }

    /// Collects all the data gathered during inspection into a single struct.
    #[inline]
    pub fn collect(self) -> InspectorData {
        InspectorData {
            logs: self.log_collector.map(|logs| logs.logs).unwrap_or_default(),
            labels: self
                .cheatcodes
                .as_ref()
                .map(|cheatcodes| {
                    cheatcodes.labels.clone().into_iter().map(|l| (l.0, l.1)).collect()
                })
                .unwrap_or_default(),
            traces: self.tracer.map(|tracer| tracer.get_traces().clone()),
            debug: self.debugger.map(|debugger| debugger.arena),
            coverage: self.coverage.map(|coverage| coverage.maps),
            cheatcodes: self.cheatcodes,
            chisel_state: self.chisel_state.and_then(|state| state.state),
        }
    }

    fn do_call_end<DB: DatabaseExt>(
        &mut self,
        ecx: &mut EvmContext<&mut DB>,
        inputs: &CallInputs,
        outcome: CallOutcome,
    ) -> CallOutcome {
        let result = outcome.result.result;
        call_inspectors_adjust_depth!(
            [&mut self.fuzzer, &mut self.debugger, &mut self.tracer, &mut self.cheatcodes],
            |inspector| {
                let new_outcome = inspector.call_end(ecx, inputs, outcome.clone());

                // If the inspector returns a different status or a revert with a non-empty message,
                // we assume it wants to tell us something
                let different = new_outcome.result.result != result ||
                    (new_outcome.result.result == InstructionResult::Revert &&
                        new_outcome.output() != outcome.output());
                different.then_some(new_outcome)
            },
            self,
            ecx
        );

        outcome
    }

    fn transact_inner<DB: DatabaseExt + DatabaseCommit>(
        &mut self,
        ecx: &mut EvmContext<&mut DB>,
        transact_to: TransactTo,
        caller: Address,
        input: Bytes,
        gas_limit: u64,
        value: U256,
    ) -> (InterpreterResult, Option<Address>) {
        let ecx = &mut ecx.inner;

        ecx.db.commit(ecx.journaled_state.state.clone());

        let nonce = ecx
            .journaled_state
            .load_account(caller, &mut ecx.db)
            .expect("failed to load caller")
            .0
            .info
            .nonce;

        let cached_env = ecx.env.clone();

        ecx.env.block.basefee = U256::ZERO;
        ecx.env.tx.caller = caller;
        ecx.env.tx.transact_to = transact_to.clone();
        ecx.env.tx.data = input;
        ecx.env.tx.value = value;
        ecx.env.tx.nonce = Some(nonce);
        // Add 21000 to the gas limit to account for the base cost of transaction.
        ecx.env.tx.gas_limit = gas_limit + 21000;
        // If we haven't disabled gas limit checks, ensure that transaction gas limit will not
        // exceed block gas limit.
        if !ecx.env.cfg.disable_block_gas_limit {
            ecx.env.tx.gas_limit =
                std::cmp::min(ecx.env.tx.gas_limit, ecx.env.block.gas_limit.to());
        }
        ecx.env.tx.gas_price = U256::ZERO;

        self.inner_context_data = Some(InnerContextData {
            sender: ecx.env.tx.caller,
            original_origin: cached_env.tx.caller,
            original_sender_nonce: nonce,
            is_create: matches!(transact_to, TransactTo::Create(_)),
        });
        self.in_inner_context = true;

        let env = EnvWithHandlerCfg::new_with_spec_id(ecx.env.clone(), ecx.spec_id());
        let res = {
            let mut evm =
                crate::utils::new_extended_evm_with_inspector(&mut *ecx.db, env, &mut *self);
            let res = evm.transact();

            // need to reset the env in case it was modified via cheatcodes during execution
            ecx.env = evm.context.evm.inner.env;
            res
        };

        self.in_inner_context = false;
        self.inner_context_data = None;

        ecx.env.tx = cached_env.tx;
        ecx.env.block.basefee = cached_env.block.basefee;

        let mut gas = Gas::new(gas_limit);

        let Ok(mut res) = res else {
            // Should we match, encode and propagate error as a revert reason?
            let result =
                InterpreterResult { result: InstructionResult::Revert, output: Bytes::new(), gas };
            return (result, None)
        };

        // Commit changes after transaction
        ecx.db.commit(res.state.clone());

        // Update both states with new DB data after commit.
        if let Err(e) = update_state(&mut ecx.journaled_state.state, &mut ecx.db) {
            let res = InterpreterResult {
                result: InstructionResult::Revert,
                output: Bytes::from(e.to_string()),
                gas,
            };
            return (res, None)
        }
        if let Err(e) = update_state(&mut res.state, &mut ecx.db) {
            let res = InterpreterResult {
                result: InstructionResult::Revert,
                output: Bytes::from(e.to_string()),
                gas,
            };
            return (res, None)
        }

        // Merge transaction journal into the active journal.
        for (addr, acc) in res.state {
            if let Some(acc_mut) = ecx.journaled_state.state.get_mut(&addr) {
                acc_mut.status |= acc.status;
                for (key, val) in acc.storage {
                    acc_mut.storage.entry(key).or_insert(val);
                }
            } else {
                ecx.journaled_state.state.insert(addr, acc);
            }
        }

        let (result, address, output) = match res.result {
            ExecutionResult::Success { reason, gas_used, gas_refunded, logs: _, output } => {
                gas.set_refund(gas_refunded as i64);
                gas.record_cost(gas_used);
                let address = match output {
                    Output::Create(_, address) => address,
                    Output::Call(_) => None,
                };
                (reason.into(), address, output.into_data())
            }
            ExecutionResult::Halt { reason, gas_used } => {
                gas.record_cost(gas_used);
                (reason.into(), None, Bytes::new())
            }
            ExecutionResult::Revert { gas_used, output } => {
                gas.record_cost(gas_used);
                (InstructionResult::Revert, None, output)
            }
        };
        (InterpreterResult { result, output, gas }, address)
    }

    /// Adjusts the EVM data for the inner EVM context.
    /// Should be called on the top-level call of inner context (depth == 0 &&
    /// self.in_inner_context) Decreases sender nonce for CALLs to keep backwards compatibility
    /// Updates tx.origin to the value before entering inner context
    fn adjust_evm_data_for_inner_context<DB: DatabaseExt>(
        &mut self,
        ecx: &mut EvmContext<&mut DB>,
    ) {
        let inner_context_data =
            self.inner_context_data.as_ref().expect("should be called in inner context");
        let sender_acc = ecx
            .journaled_state
            .state
            .get_mut(&inner_context_data.sender)
            .expect("failed to load sender");
        if !inner_context_data.is_create {
            sender_acc.info.nonce = inner_context_data.original_sender_nonce;
        }
        ecx.env.tx.caller = inner_context_data.original_origin;
    }
}

// NOTE: `&mut DB` is required because we recurse inside of `transact_inner` and we need to use the
// same reference to the DB, otherwise there's infinite recursion and Rust fails to instatiate this
// implementation. This currently works because internally we only use `&mut DB` anyways, but if
// this ever needs to be changed, this can be reverted back to using just `DB`, and instead using
// dynamic dispatch (`&mut dyn ...`) in `transact_inner`.
impl<DB: DatabaseExt + DatabaseCommit> Inspector<&mut DB> for InspectorStack {
    fn initialize_interp(&mut self, interpreter: &mut Interpreter, ecx: &mut EvmContext<&mut DB>) {
        call_inspectors_adjust_depth!(
            #[no_ret]
            [&mut self.coverage, &mut self.tracer, &mut self.cheatcodes],
            |inspector| inspector.initialize_interp(interpreter, ecx),
            self,
            ecx
        );
    }

    fn step(&mut self, interpreter: &mut Interpreter, ecx: &mut EvmContext<&mut DB>) {
        call_inspectors_adjust_depth!(
            #[no_ret]
            [
                &mut self.fuzzer,
                &mut self.debugger,
                &mut self.tracer,
                &mut self.coverage,
                &mut self.cheatcodes,
            ],
            |inspector| inspector.step(interpreter, ecx),
            self,
            ecx
        );
    }

    fn step_end(&mut self, interpreter: &mut Interpreter, ecx: &mut EvmContext<&mut DB>) {
        call_inspectors_adjust_depth!(
            #[no_ret]
            [&mut self.tracer, &mut self.cheatcodes, &mut self.chisel_state],
            |inspector| inspector.step_end(interpreter, ecx),
            self,
            ecx
        );
    }

    fn log(&mut self, ecx: &mut EvmContext<&mut DB>, log: &Log) {
        call_inspectors_adjust_depth!(
            #[no_ret]
            [&mut self.tracer, &mut self.log_collector, &mut self.cheatcodes],
            |inspector| inspector.log(ecx, log),
            self,
            ecx
        );
    }

    fn call(
        &mut self,
        ecx: &mut EvmContext<&mut DB>,
        call: &mut CallInputs,
    ) -> Option<CallOutcome> {
        if self.in_inner_context && ecx.journaled_state.depth == 0 {
            self.adjust_evm_data_for_inner_context(ecx);
            return None;
        }

        call_inspectors_adjust_depth!(
            [
                &mut self.fuzzer,
                &mut self.debugger,
                &mut self.tracer,
                &mut self.log_collector,
                &mut self.cheatcodes,
            ],
            |inspector| {
                let mut out = None;
                if let Some(output) = inspector.call(ecx, call) {
                    if output.result.result != InstructionResult::Continue {
                        out = Some(Some(output));
                    }
                }
                out
            },
            self,
            ecx
        );

        if self.enable_isolation &&
            call.context.scheme == CallScheme::Call &&
            !self.in_inner_context &&
            ecx.journaled_state.depth == 1
        {
            let (result, _) = self.transact_inner(
                ecx,
                TransactTo::Call(call.contract),
                call.context.caller,
                call.input.clone(),
                call.gas_limit,
                call.transfer.value,
            );
            return Some(CallOutcome { result, memory_offset: call.return_memory_offset.clone() })
        }

        None
    }

    fn call_end(
        &mut self,
        ecx: &mut EvmContext<&mut DB>,
        inputs: &CallInputs,
        outcome: CallOutcome,
    ) -> CallOutcome {
        // Inner context calls with depth 0 are being dispatched as top-level calls with depth 1.
        // Avoid processing twice.
        if self.in_inner_context && ecx.journaled_state.depth == 0 {
            return outcome
        }

        let outcome = self.do_call_end(ecx, inputs, outcome);
        if outcome.result.is_revert() {
            // Encountered a revert, since cheatcodes may have altered the evm state in such a way
            // that violates some constraints, e.g. `deal`, we need to manually roll back on revert
            // before revm reverts the state itself
            if let Some(cheats) = self.cheatcodes.as_mut() {
                cheats.on_revert(ecx);
            }
        }

        outcome
    }

    fn create(
        &mut self,
        ecx: &mut EvmContext<&mut DB>,
        create: &mut CreateInputs,
    ) -> Option<CreateOutcome> {
        if self.in_inner_context && ecx.journaled_state.depth == 0 {
            self.adjust_evm_data_for_inner_context(ecx);
            return None;
        }

        call_inspectors_adjust_depth!(
            [&mut self.debugger, &mut self.tracer, &mut self.coverage, &mut self.cheatcodes],
            |inspector| inspector.create(ecx, create).map(Some),
            self,
            ecx
        );

        if self.enable_isolation && !self.in_inner_context && ecx.journaled_state.depth == 1 {
            let (result, address) = self.transact_inner(
                ecx,
                TransactTo::Create(create.scheme),
                create.caller,
                create.init_code.clone(),
                create.gas_limit,
                create.value,
            );
            return Some(CreateOutcome { result, address })
        }

        None
    }

    fn create_end(
        &mut self,
        ecx: &mut EvmContext<&mut DB>,
        call: &CreateInputs,
        outcome: CreateOutcome,
    ) -> CreateOutcome {
        // Inner context calls with depth 0 are being dispatched as top-level calls with depth 1.
        // Avoid processing twice.
        if self.in_inner_context && ecx.journaled_state.depth == 0 {
            return outcome
        }

        let result = outcome.result.result;

        call_inspectors_adjust_depth!(
            [&mut self.debugger, &mut self.tracer, &mut self.cheatcodes],
            |inspector| {
                let new_outcome = inspector.create_end(ecx, call, outcome.clone());

                // If the inspector returns a different status or a revert with a non-empty message,
                // we assume it wants to tell us something
                let different = new_outcome.result.result != result ||
                    (new_outcome.result.result == InstructionResult::Revert &&
                        new_outcome.output() != outcome.output());
                different.then_some(new_outcome)
            },
            self,
            ecx
        );

        outcome
    }

    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        call_inspectors!([&mut self.tracer], |inspector| Inspector::<DB>::selfdestruct(
            inspector, contract, target, value
        ));
    }
}

impl<DB: DatabaseExt + DatabaseCommit> InspectorExt<&mut DB> for InspectorStack {
    fn should_use_create2_factory(
        &mut self,
        ecx: &mut EvmContext<&mut DB>,
        inputs: &mut CreateInputs,
    ) -> bool {
        call_inspectors_adjust_depth!(
            [&mut self.cheatcodes],
            |inspector| { inspector.should_use_create2_factory(ecx, inputs).then_some(true) },
            self,
            ecx
        );

        false
    }
}
