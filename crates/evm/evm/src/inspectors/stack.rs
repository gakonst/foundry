use super::{
    Cheatcodes, CheatsConfig, ChiselState, CoverageCollector, Debugger, Fuzzer, LogCollector,
    StackSnapshotType, TracePrinter, TracingInspector, TracingInspectorConfig,
};
use alloy_primitives::{Address, Bytes, Log, B256, U256};
use alloy_signer::LocalWallet;
use foundry_evm_core::{
    backend::DatabaseExt,
    debug::DebugArena,
    utils::{eval_to_instruction_result, halt_to_instruction_result},
};
use foundry_evm_coverage::HitMaps;
use foundry_evm_traces::CallTraceArena;
use revm::{
    evm_inner,
    interpreter::{
        return_revert, CallInputs, CreateInputs, Gas, InstructionResult, Interpreter, Stack,
    },
    primitives::{BlockEnv, Env, ExecutionResult, Output, State, TxEnv},
    DatabaseCommit, EVMData, Inspector,
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
            if let Some(result) = $call {
                return result;
            }
        }
    )+}};
    ([$($inspector:expr),+ $(,)?], |$id:ident $(,)?| $call:expr, $self:ident, $data:ident $(,)?) => {
        if $self.in_inner_context {
            $data.journaled_state.depth += 1;
        }
        call_inspectors!([$($inspector),+], |$id| {
            $call.map(|result| {
                if $self.in_inner_context {
                    $data.journaled_state.depth -= 1;
                }
                result
            })
        });
        if $self.in_inner_context {
            $data.journaled_state.depth -= 1;
        }
    }
}

fn merge_states(main_state: &mut State, new_state: State) {
    for (addr, acc) in new_state {
        if main_state.contains_key(&addr) {
            let acc_mut = main_state.get_mut(&addr).unwrap();
            acc_mut.status |= acc.status;
            acc_mut.info = acc.info;
            acc_mut.storage.extend(acc.storage);
        } else {
            main_state.insert(addr, acc);
        }
    }
}

/// The collected results of [`InspectorStack`].
pub struct InspectorData {
    pub logs: Vec<Log>,
    pub labels: HashMap<Address, String>,
    pub traces: Option<CallTraceArena>,
    pub debug: Option<DebugArena>,
    pub coverage: Option<HitMaps>,
    pub cheatcodes: Option<Cheatcodes>,
    pub script_wallets: Vec<LocalWallet>,
    pub chisel_state: Option<(Stack, Vec<u8>, InstructionResult)>,
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
    pub printer: Option<TracePrinter>,
    pub tracer: Option<TracingInspector>,
    /// Flag marking if we are in the inner EVM context.
    pub in_inner_context: bool,
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
                record_call_return_data: true,
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
            #[allow(clippy::useless_asref)] // https://github.com/rust-lang/rust-clippy/issues/12135
            script_wallets: self
                .cheatcodes
                .as_ref()
                .map(|cheatcodes| cheatcodes.script_wallets.clone())
                .unwrap_or_default(),
            cheatcodes: self.cheatcodes,
            chisel_state: self.chisel_state.and_then(|state| state.state),
        }
    }

    fn do_call_end<DB: DatabaseExt>(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: &CallInputs,
        remaining_gas: Gas,
        status: InstructionResult,
        retdata: Bytes,
    ) -> (InstructionResult, Gas, Bytes) {
        call_inspectors!(
            [
                &mut self.fuzzer,
                &mut self.debugger,
                &mut self.tracer,
                &mut self.coverage,
                &mut self.log_collector,
                &mut self.cheatcodes,
                &mut self.printer
            ],
            |inspector| {
                let (new_status, new_gas, new_retdata) =
                    inspector.call_end(data, call, remaining_gas, status, retdata.clone());

                // If the inspector returns a different status or a revert with a non-empty message,
                // we assume it wants to tell us something
                if new_status != status ||
                    (new_status == InstructionResult::Revert && new_retdata != retdata)
                {
                    Some((new_status, new_gas, new_retdata))
                } else {
                    None
                }
            },
            self,
            data
        );
        (status, remaining_gas, retdata)
    }
}

impl<DB: DatabaseExt + DatabaseCommit + Clone> Inspector<DB> for InspectorStack {
    fn initialize_interp(&mut self, interpreter: &mut Interpreter<'_>, data: &mut EVMData<'_, DB>) {
        let res = interpreter.instruction_result;
        call_inspectors!(
            [
                &mut self.debugger,
                &mut self.coverage,
                &mut self.tracer,
                &mut self.log_collector,
                &mut self.cheatcodes,
                &mut self.printer
            ],
            |inspector| {
                inspector.initialize_interp(interpreter, data);

                // Allow inspectors to exit early
                if interpreter.instruction_result != res {
                    Some(())
                } else {
                    None
                }
            },
            self,
            data
        );
    }

    fn step(&mut self, interpreter: &mut Interpreter<'_>, data: &mut EVMData<'_, DB>) {
        let res = interpreter.instruction_result;
        call_inspectors!(
            [
                &mut self.fuzzer,
                &mut self.debugger,
                &mut self.tracer,
                &mut self.coverage,
                &mut self.log_collector,
                &mut self.cheatcodes,
                &mut self.printer
            ],
            |inspector| {
                inspector.step(interpreter, data);

                // Allow inspectors to exit early
                if interpreter.instruction_result != res {
                    Some(())
                } else {
                    None
                }
            },
            self,
            data
        );
    }

    fn log(
        &mut self,
        evm_data: &mut EVMData<'_, DB>,
        address: &Address,
        topics: &[B256],
        data: &Bytes,
    ) {
        call_inspectors!(
            [&mut self.tracer, &mut self.log_collector, &mut self.cheatcodes, &mut self.printer],
            |inspector| {
                inspector.log(evm_data, address, topics, data);
                None
            },
            self,
            evm_data
        );
    }

    fn step_end(&mut self, interpreter: &mut Interpreter<'_>, data: &mut EVMData<'_, DB>) {
        let res = interpreter.instruction_result;
        call_inspectors!(
            [
                &mut self.debugger,
                &mut self.tracer,
                &mut self.log_collector,
                &mut self.cheatcodes,
                &mut self.printer,
                &mut self.chisel_state
            ],
            |inspector| {
                inspector.step_end(interpreter, data);

                // Allow inspectors to exit early
                if interpreter.instruction_result != res {
                    Some(())
                } else {
                    None
                }
            },
            self,
            data
        );
    }

    fn call(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: &mut CallInputs,
    ) -> (InstructionResult, Gas, Bytes) {
        // Inner context calls with depth 0 are being dispatched as top-level calls with depth 1.
        // Avoid processing twice.
        if self.in_inner_context && data.journaled_state.depth == 0 {
            return (InstructionResult::Continue, Gas::new(call.gas_limit), Bytes::new());
        }
        call_inspectors!(
            [
                &mut self.fuzzer,
                &mut self.debugger,
                &mut self.tracer,
                &mut self.coverage,
                &mut self.log_collector,
                &mut self.cheatcodes,
                &mut self.printer
            ],
            |inspector| {
                let (status, gas, retdata) = inspector.call(data, call);

                // Allow inspectors to exit early
                if status != InstructionResult::Continue {
                    Some((status, gas, retdata))
                } else {
                    None
                }
            },
            self,
            data
        );
        if !call.is_static && !self.in_inner_context && data.journaled_state.depth == 1 {
            data.db.commit(data.journaled_state.state.clone());
            let nonce = data
                .journaled_state
                .load_account(call.context.caller, data.db)
                .expect("failed to load caller")
                .0
                .info
                .nonce;
            let mut env = Env {
                cfg: data.env.cfg.clone(),
                block: BlockEnv {
                    basefee: U256::ZERO,
                    gas_limit: U256::MAX,
                    ..data.env.block.clone()
                },
                tx: TxEnv {
                    caller: call.context.caller,
                    transact_to: revm::primitives::TransactTo::Call(call.contract),
                    data: call.input.clone(),
                    value: call.transfer.value,
                    gas_limit: call.gas_limit,
                    nonce: Some(nonce),
                    gas_price: U256::ZERO,
                    ..data.env.tx.clone()
                },
            };
            self.in_inner_context = true;
            let res = evm_inner(&mut env, data.db, Some(self))
                .transact()
                .expect("error while transacting");
            self.in_inner_context = false;
            let mut gas = Gas::new(call.gas_limit);

            merge_states(&mut data.journaled_state.state, res.state);

            match res.result {
                ExecutionResult::Success { reason, gas_used, gas_refunded, logs: _, output } => {
                    gas.set_refund(gas_refunded as i64);
                    gas.record_cost(gas_used);
                    return (eval_to_instruction_result(reason), gas, output.into_data());
                }
                ExecutionResult::Halt { reason, gas_used } => {
                    gas.record_cost(gas_used);
                    return (halt_to_instruction_result(reason), gas, Bytes::new());
                }
                ExecutionResult::Revert { gas_used, output } => {
                    gas.record_cost(gas_used);
                    return (InstructionResult::Revert, gas, output);
                }
            }
        }

        (InstructionResult::Continue, Gas::new(call.gas_limit), Bytes::new())
    }

    fn call_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: &CallInputs,
        remaining_gas: Gas,
        status: InstructionResult,
        retdata: Bytes,
    ) -> (InstructionResult, Gas, Bytes) {
        // Inner context calls with depth 0 are being dispatched as top-level calls with depth 1.
        // Avoid processing twice.
        if self.in_inner_context && data.journaled_state.depth == 0 {
            return (status, remaining_gas, retdata);
        }

        let res = self.do_call_end(data, call, remaining_gas, status, retdata);
        if matches!(res.0, return_revert!()) {
            // Encountered a revert, since cheatcodes may have altered the evm state in such a way
            // that violates some constraints, e.g. `deal`, we need to manually roll back on revert
            // before revm reverts the state itself
            if let Some(cheats) = self.cheatcodes.as_mut() {
                cheats.on_revert(data);
            }
        }

        res
    }

    fn create(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: &mut CreateInputs,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        // Inner context calls with depth 0 are being dispatched as top-level calls with depth 1.
        // Avoid processing twice.
        if self.in_inner_context && data.journaled_state.depth == 0 {
            return (InstructionResult::Continue, None, Gas::new(call.gas_limit), Bytes::new());
        }
        call_inspectors!(
            [
                &mut self.debugger,
                &mut self.tracer,
                &mut self.coverage,
                &mut self.log_collector,
                &mut self.cheatcodes,
                &mut self.printer
            ],
            |inspector| {
                let (status, addr, gas, retdata) = inspector.create(data, call);

                // Allow inspectors to exit early
                if status != InstructionResult::Continue {
                    Some((status, addr, gas, retdata))
                } else {
                    None
                }
            },
            self,
            data
        );
        if !self.in_inner_context && data.journaled_state.depth == 1 {
            data.db.commit(data.journaled_state.state.clone());
            let nonce = data
                .journaled_state
                .load_account(call.caller, data.db)
                .expect("failed to load caller")
                .0
                .info
                .nonce;
            let mut env = Env {
                cfg: data.env.cfg.clone(),
                block: BlockEnv {
                    basefee: U256::ZERO,
                    gas_limit: U256::MAX,
                    ..data.env.block.clone()
                },
                tx: TxEnv {
                    caller: call.caller,
                    transact_to: revm::primitives::TransactTo::Create(call.scheme),
                    data: call.init_code.clone(),
                    value: call.value,
                    gas_limit: call.gas_limit,
                    nonce: Some(nonce),
                    gas_price: U256::ZERO,
                    ..data.env.tx.clone()
                },
            };

            self.in_inner_context = true;
            let res = evm_inner(&mut env, data.db, Some(self))
                .transact()
                .expect("error while transacting");
            self.in_inner_context = false;
            let mut gas = Gas::new(call.gas_limit);

            merge_states(&mut data.journaled_state.state, res.state);

            match res.result {
                ExecutionResult::Success { reason, gas_used, gas_refunded, logs: _, output } => {
                    gas.set_refund(gas_refunded as i64);
                    gas.record_cost(gas_used);
                    let address = match output {
                        Output::Create(_, address) => address,
                        _ => unreachable!(),
                    };
                    return (eval_to_instruction_result(reason), address, gas, output.into_data());
                }
                ExecutionResult::Halt { reason, gas_used } => {
                    gas.record_cost(gas_used);
                    return (halt_to_instruction_result(reason), None, gas, Bytes::new());
                }
                ExecutionResult::Revert { gas_used, output } => {
                    gas.record_cost(gas_used);
                    return (InstructionResult::Revert, None, gas, output);
                }
            }
        }

        (InstructionResult::Continue, None, Gas::new(call.gas_limit), Bytes::new())
    }

    fn create_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        call: &CreateInputs,
        status: InstructionResult,
        address: Option<Address>,
        remaining_gas: Gas,
        retdata: Bytes,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        // Inner context calls with depth 0 are being dispatched as top-level calls with depth 1.
        // Avoid processing twice.
        if self.in_inner_context && data.journaled_state.depth == 0 {
            return (status, address, remaining_gas, retdata);
        }
        call_inspectors!(
            [
                &mut self.debugger,
                &mut self.tracer,
                &mut self.coverage,
                &mut self.log_collector,
                &mut self.cheatcodes,
                &mut self.printer
            ],
            |inspector| {
                let (new_status, new_address, new_gas, new_retdata) = inspector.create_end(
                    data,
                    call,
                    status,
                    address,
                    remaining_gas,
                    retdata.clone(),
                );

                if new_status != status {
                    Some((new_status, new_address, new_gas, new_retdata))
                } else {
                    None
                }
            },
            self,
            data
        );

        (status, address, remaining_gas, retdata)
    }

    fn selfdestruct(&mut self, contract: Address, target: Address, value: U256) {
        call_inspectors!(
            [
                &mut self.debugger,
                &mut self.tracer,
                &mut self.log_collector,
                &mut self.cheatcodes,
                &mut self.printer,
                &mut self.chisel_state
            ],
            |inspector| {
                Inspector::<DB>::selfdestruct(inspector, contract, target, value);
                None
            }
        );
    }
}
