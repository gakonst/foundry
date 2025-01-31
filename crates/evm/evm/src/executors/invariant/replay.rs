use super::{
    call_after_invariant_function, call_invariant_function, error::FailedInvariantCaseData,
    shrink_sequence,
};
use crate::executors::Executor;
use alloy_dyn_abi::JsonAbiExt;
use alloy_primitives::{map::HashMap, Log};
use eyre::Result;
use foundry_common::{ContractsByAddress, ContractsByArtifact};
use foundry_evm_coverage::HitMaps;
use foundry_evm_fuzz::{
    invariant::{BasicTxDetails, InvariantContract},
    BaseCounterExample,
};
use foundry_evm_traces::{load_contracts, TraceKind, TraceMode, Traces};
use indicatif::ProgressBar;
use parking_lot::RwLock;
use proptest::test_runner::TestError;
use revm::primitives::U256;
use std::sync::Arc;

/// Replays a call sequence for collecting logs and traces.
/// Returns counterexample to be used when the call sequence is a failed scenario.
#[allow(clippy::too_many_arguments)]
pub fn replay_run(
    invariant_contract: &InvariantContract<'_>,
    mut executor: Executor,
    known_contracts: &ContractsByArtifact,
    mut ided_contracts: ContractsByAddress,
    logs: &mut Vec<Log>,
    traces: &mut Traces,
    coverage: &mut Option<HitMaps>,
    deprecated_cheatcodes: &mut HashMap<&'static str, Option<&'static str>>,
    isolated_cheatcodes: &mut Vec<&'static str>,
    inputs: &[BasicTxDetails],
) -> Result<Vec<BaseCounterExample>> {
    // We want traces for a failed case.
    if executor.inspector().tracer.is_none() {
        executor.set_tracing(TraceMode::Call);
    }

    let mut counterexample_sequence = vec![];

    // Replay each call from the sequence, collect logs, traces and coverage.
    for tx in inputs {
        let call_result = executor.transact_raw(
            tx.sender,
            tx.call_details.target,
            tx.call_details.calldata.clone(),
            U256::ZERO,
        )?;

        logs.extend(call_result.logs);
        traces.push((TraceKind::Execution, call_result.traces.clone().unwrap()));
        HitMaps::merge_opt(coverage, call_result.coverage);

        // Identify newly generated contracts, if they exist.
        ided_contracts
            .extend(load_contracts(call_result.traces.iter().map(|a| &a.arena), known_contracts));

        // Create counter example to be used in failed case.
        counterexample_sequence.push(BaseCounterExample::from_invariant_call(
            tx.sender,
            tx.call_details.target,
            &tx.call_details.calldata,
            &ided_contracts,
            call_result.traces,
        ));
    }

    // Replay invariant to collect logs and traces.
    // We do this only once at the end of the replayed sequence.
    // Checking after each call doesn't add valuable info for passing scenario
    // (invariant call result is always success) nor for failed scenarios
    // (invariant call result is always success until the last call that breaks it).
    let (invariant_result, invariant_success) = call_invariant_function(
        &executor,
        invariant_contract.address,
        invariant_contract.invariant_function.abi_encode_input(&[])?.into(),
    )?;
    traces.push((TraceKind::Execution, invariant_result.traces.clone().unwrap()));
    logs.extend(invariant_result.logs);
    deprecated_cheatcodes.extend(
        invariant_result
            .cheatcodes
            .as_ref()
            .map_or_else(Default::default, |cheats| cheats.deprecated_cheatcodes.clone()),
    );
    isolated_cheatcodes.extend(
        invariant_result
            .cheatcodes
            .as_ref()
            .map_or_else(Default::default, |cheats| cheats.isolated_cheatcodes.clone()),
    );

    // Collect after invariant logs and traces.
    if invariant_contract.call_after_invariant && invariant_success {
        let (after_invariant_result, _) =
            call_after_invariant_function(&executor, invariant_contract.address)?;
        traces.push((TraceKind::Execution, after_invariant_result.traces.clone().unwrap()));
        logs.extend(after_invariant_result.logs);
    }

    Ok(counterexample_sequence)
}

/// Replays the error case, shrinks the failing sequence and collects all necessary traces.
#[allow(clippy::too_many_arguments)]
pub fn replay_error(
    failed_case: &FailedInvariantCaseData,
    invariant_contract: &InvariantContract<'_>,
    mut executor: Executor,
    known_contracts: &ContractsByArtifact,
    ided_contracts: ContractsByAddress,
    logs: &mut Vec<Log>,
    traces: &mut Traces,
    coverage: &mut Option<HitMaps>,
    deprecated_cheatcodes: &mut HashMap<&'static str, Option<&'static str>>,
    isolated_cheatcodes: &mut Vec<&'static str>,
    progress: Option<&ProgressBar>,
) -> Result<Vec<BaseCounterExample>> {
    match failed_case.test_error {
        // Don't use at the moment.
        TestError::Abort(_) => Ok(vec![]),
        TestError::Fail(_, ref calls) => {
            // Shrink sequence of failed calls.
            let calls = shrink_sequence(
                failed_case,
                calls,
                &executor,
                invariant_contract.call_after_invariant,
                progress,
            )?;

            set_up_inner_replay(&mut executor, &failed_case.inner_sequence);

            // Replay calls to get the counterexample and to collect logs, traces and coverage.
            replay_run(
                invariant_contract,
                executor,
                known_contracts,
                ided_contracts,
                logs,
                traces,
                coverage,
                deprecated_cheatcodes,
                isolated_cheatcodes,
                &calls,
            )
        }
    }
}

/// Sets up the calls generated by the internal fuzzer, if they exist.
fn set_up_inner_replay(executor: &mut Executor, inner_sequence: &[Option<BasicTxDetails>]) {
    if let Some(fuzzer) = &mut executor.inspector_mut().fuzzer {
        if let Some(call_generator) = &mut fuzzer.call_generator {
            call_generator.last_sequence = Arc::new(RwLock::new(inner_sequence.to_owned()));
            call_generator.set_replay(true);
        }
    }
}
