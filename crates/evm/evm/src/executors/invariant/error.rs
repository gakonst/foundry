use super::{BasicTxDetails, InvariantContract};
use crate::executors::{Executor, RawCallResult};
use alloy_json_abi::Function;
use alloy_primitives::{Address, Bytes, Log};
use eyre::Result;
use foundry_common::contracts::{ContractsByAddress, ContractsByArtifact};
use foundry_evm_core::{constants::CALLER, decode::RevertDecoder};
use foundry_evm_fuzz::{
    invariant::FuzzRunIdentifiedContracts, BaseCounterExample, CounterExample, FuzzedCases, Reason,
};
use foundry_evm_traces::{load_contracts, CallTraceArena, TraceKind, Traces};
use itertools::Itertools;
use parking_lot::RwLock;
use proptest::test_runner::TestError;
use rand::{seq, thread_rng, Rng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use revm::primitives::U256;
use std::{borrow::Cow, sync::Arc};

/// Stores information about failures and reverts of the invariant tests.
#[derive(Clone, Default)]
pub struct InvariantFailures {
    /// Total number of reverts.
    pub reverts: usize,
    /// How many different invariants have been broken.
    pub broken_invariants_count: usize,
    /// The latest revert reason of a run.
    pub revert_reason: Option<String>,
    /// Maps a broken invariant to its specific error.
    pub error: Option<InvariantFuzzError>,
}

impl InvariantFailures {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_inner(self) -> (usize, Option<InvariantFuzzError>) {
        (self.reverts, self.error)
    }
}

/// The outcome of an invariant fuzz test
#[derive(Debug)]
pub struct InvariantFuzzTestResult {
    pub error: Option<InvariantFuzzError>,
    /// Every successful fuzz test case
    pub cases: Vec<FuzzedCases>,
    /// Number of reverted fuzz calls
    pub reverts: usize,

    /// The entire inputs of the last run of the invariant campaign, used for
    /// replaying the run for collecting traces.
    pub last_run_inputs: Vec<BasicTxDetails>,

    /// Additional traces used for gas report construction.
    pub gas_report_traces: Vec<Vec<CallTraceArena>>,
}

#[derive(Clone, Debug)]
pub enum InvariantFuzzError {
    Revert(FailedInvariantCaseData),
    BrokenInvariant(FailedInvariantCaseData),
    MaxAssumeRejects(u32),
}

impl InvariantFuzzError {
    pub fn revert_reason(&self) -> Option<String> {
        match self {
            Self::BrokenInvariant(case_data) | Self::Revert(case_data) => {
                (!case_data.revert_reason.is_empty()).then(|| case_data.revert_reason.clone())
            }
            Self::MaxAssumeRejects(allowed) => Some(format!(
                "The `vm.assume` cheatcode rejected too many inputs ({allowed} allowed)"
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FailedInvariantCaseData {
    pub logs: Vec<Log>,
    pub traces: Option<CallTraceArena>,
    /// The proptest error occurred as a result of a test case.
    pub test_error: TestError<Vec<BasicTxDetails>>,
    /// The return reason of the offending call.
    pub return_reason: Reason,
    /// The revert string of the offending call.
    pub revert_reason: String,
    /// Address of the invariant asserter.
    pub addr: Address,
    /// Function data for invariant check.
    pub func: Option<Bytes>,
    /// Inner fuzzing Sequence coming from overriding calls.
    pub inner_sequence: Vec<Option<BasicTxDetails>>,
    /// Shrink the failed test case to the smallest sequence.
    pub shrink: bool,
    /// Shrink run limit
    pub shrink_run_limit: usize,
}

impl FailedInvariantCaseData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        invariant_contract: &InvariantContract<'_>,
        targeted_contracts: &FuzzRunIdentifiedContracts,
        error_func: Option<&Function>,
        calldata: &[BasicTxDetails],
        call_result: RawCallResult,
        inner_sequence: &[Option<BasicTxDetails>],
        shrink: bool,
        shrink_run_limit: usize,
    ) -> Self {
        let (func, origin) = if let Some(f) = error_func {
            (Some(f.selector().to_vec().into()), f.name.as_str())
        } else {
            (None, "Revert")
        };

        // Collect abis of fuzzed and invariant contracts to decode custom error.
        let targets = targeted_contracts.targets.lock();
        let abis = targets
            .iter()
            .map(|contract| &contract.1 .1)
            .chain(std::iter::once(invariant_contract.abi));

        let revert_reason = RevertDecoder::new()
            .with_abis(abis)
            .decode(call_result.result.as_ref(), Some(call_result.exit_reason));

        Self {
            logs: call_result.logs,
            traces: call_result.traces,
            test_error: proptest::test_runner::TestError::Fail(
                format!("{origin}, reason: {revert_reason}").into(),
                calldata.to_vec(),
            ),
            return_reason: "".into(),
            revert_reason,
            addr: invariant_contract.address,
            func,
            inner_sequence: inner_sequence.to_vec(),
            shrink,
            shrink_run_limit,
        }
    }

    /// Replays the error case and collects all necessary traces.
    pub fn replay(
        &self,
        mut executor: Executor,
        known_contracts: Option<&ContractsByArtifact>,
        mut ided_contracts: ContractsByAddress,
        logs: &mut Vec<Log>,
        traces: &mut Traces,
    ) -> Result<Option<CounterExample>> {
        let mut counterexample_sequence = vec![];
        let mut calls = match self.test_error {
            // Don't use at the moment.
            TestError::Abort(_) => return Ok(None),
            TestError::Fail(_, ref calls) => calls.clone(),
        };

        if self.shrink {
            calls = self.try_shrinking(&calls, &executor)?.into_iter().cloned().collect();
        } else {
            trace!(target: "forge::test", "Shrinking disabled.");
        }

        // We want traces for a failed case.
        executor.set_tracing(true);

        set_up_inner_replay(&mut executor, &self.inner_sequence);

        // Replay each call from the sequence until we break the invariant.
        for (sender, (addr, bytes, _, _)) in calls.iter() {
            let call_result =
                executor.call_raw_committing(*sender, *addr, bytes.clone(), U256::ZERO)?;

            logs.extend(call_result.logs);
            traces.push((TraceKind::Execution, call_result.traces.clone().unwrap()));

            // Identify newly generated contracts, if they exist.
            ided_contracts.extend(load_contracts(
                vec![(TraceKind::Execution, call_result.traces.clone().unwrap())],
                known_contracts,
            ));

            counterexample_sequence.push(BaseCounterExample::create(
                *sender,
                *addr,
                bytes,
                &ided_contracts,
                call_result.traces,
            ));

            // Checks the invariant.
            if let Some(func) = &self.func {
                let error_call_result =
                    executor.call_raw(CALLER, self.addr, func.clone(), U256::ZERO)?;

                traces.push((TraceKind::Execution, error_call_result.traces.clone().unwrap()));

                logs.extend(error_call_result.logs);
                if error_call_result.reverted {
                    break
                }
            }
        }

        Ok((!counterexample_sequence.is_empty())
            .then_some(CounterExample::Sequence(counterexample_sequence)))
    }

    /// Checks that a subsequence of the provided calls fails the provided invariant test
    /// and updates an Arc Mutex of the indices of the shortest sequence
    fn set_fails_successfully(
        &self,
        mut executor: Executor,
        calls: &[BasicTxDetails],
        use_calls: &[usize],
        curr_seq: Arc<RwLock<Vec<usize>>>,
    ) -> eyre::Result<()> {
        if curr_seq.read().len() == 1 {
            // if current sequence is already the smallest possible, just return
            return Ok(());
        }

        let mut new_sequence = Vec::with_capacity(calls.len());
        for index in 0..calls.len() {
            if !use_calls.contains(&index) {
                continue
            }

            new_sequence.push(index);

            // If the new sequence is already longer than the known best, skip execution
            if new_sequence.len() >= curr_seq.read().len() {
                return Ok(());
            }
        }

        for (seq_idx, call_index) in new_sequence.iter().enumerate() {
            let (sender, (addr, bytes, _, _)) = &calls[*call_index];

            executor.call_raw_committing(*sender, *addr, bytes.clone(), U256::ZERO)?;

            // Checks the invariant. If we revert or fail before the last call, all the better.
            if let Some(func) = &self.func {
                let mut call_result =
                    executor.call_raw(CALLER, self.addr, func.clone(), U256::ZERO)?;
                let is_success = executor.is_raw_call_success(
                    self.addr,
                    Cow::Owned(call_result.state_changeset.take().unwrap()),
                    &call_result,
                    false,
                );
                if !is_success {
                    let mut locked = curr_seq.write();
                    if new_sequence[..=seq_idx].len() < locked.len() {
                        // update the curr_sequence if the new sequence is lower than
                        *locked = new_sequence[..=seq_idx].to_vec();
                    }
                }
            }
        }
        Ok(())
    }

    /// Tries to shrink the failure case to its smallest sequence of calls.
    ///
    /// If the number of calls is small enough, we can guarantee maximal shrinkage
    fn try_shrinking<'a>(
        &self,
        calls: &'a [BasicTxDetails],
        executor: &Executor,
    ) -> eyre::Result<Vec<&'a BasicTxDetails>> {
        trace!(target: "forge::test", "Shrinking.");

        // Special case test: the invariant is *unsatisfiable* - it took 0 calls to
        // break the invariant -- consider emitting a warning.
        if let Some(func) = &self.func {
            let error_call_result =
                executor.call_raw(CALLER, self.addr, func.clone(), U256::ZERO)?;
            if error_call_result.reverted {
                return Ok(vec![]);
            }
        }

        let shrunk_call_indices = self.try_shrinking_recurse(calls, executor, 0, 0)?;

        // We recreate the call sequence in the same order as they reproduce the failure,
        // otherwise we could end up with inverted sequence.
        // E.g. in a sequence of:
        // 1. Alice calls acceptOwnership and reverts
        // 2. Bob calls transferOwnership to Alice
        // 3. Alice calls acceptOwnership and test fails
        // we shrink to indices of [2, 1] and we recreate call sequence in same order.
        Ok(shrunk_call_indices.iter().map(|idx| &calls[*idx]).collect())
    }

    /// We try to construct a [powerset](https://en.wikipedia.org/wiki/Power_set) of the sequence if
    /// the configuration allows for it and the length of the sequence is small enough. If we
    /// do construct the powerset, we run all of the sequences in parallel, finding the smallest
    /// one. If we have ran the powerset, we are guaranteed to find the smallest sequence for a
    /// given set of calls (however, not guaranteed *global* smallest if a random sequence was
    /// used at any point during recursion).
    ///
    /// If we were unable to construct a powerset, we construct a random sample over the sequence
    /// and run these sequences in parallel instead.
    ///
    /// After running either the powerset or the random sequences, we check if we successfully
    /// shrunk the call sequence.
    fn try_shrinking_recurse(
        &self,
        calls: &[BasicTxDetails],
        executor: &Executor,
        runs: usize,
        retries: usize,
    ) -> eyre::Result<Vec<usize>> {
        // Construct a ArcRwLock vector of indices of `calls`
        let shrunk_call_indices = Arc::new(RwLock::new((0..calls.len()).collect()));
        let shrink_limit = self.shrink_run_limit - runs;

        let upper_bound = calls.len().saturating_sub(1);
        // We construct either a full powerset (this guarantees we maximally shrunk for the given
        // calls) or a random subset
        let (set_of_indices, is_powerset): (Vec<_>, bool) = if calls.len() <= 64 &&
            (1 << calls.len() as u32) <= shrink_limit
        {
            // We add the last tx always because thats ultimately what broke the invariant
            let powerset = (0..upper_bound)
                .powerset()
                .map(|mut subset| {
                    subset.push(upper_bound);
                    subset
                })
                .collect();
            (powerset, true)
        } else {
            // construct a random set of subsequences
            let mut rng = thread_rng();
            (
                (0..shrink_limit / 3)
                    .map(|_| {
                        // Select between 1 and calls.len() - 2 number of indices
                        let amt: usize = rng.gen_range(1..upper_bound);
                        // Construct a random sequence of indices, up to calls.len() - 1 (sample is
                        // exclusive range and we dont include the last tx
                        // because its always included), and amt number of indices
                        let mut seq = seq::index::sample(&mut rng, upper_bound, amt).into_vec();
                        // Sort the indices because seq::index::sample is unordered
                        seq.sort();
                        // We add the last tx always because thats what ultimately broke the
                        // invariant
                        seq.push(upper_bound);
                        seq
                    })
                    .collect(),
                false,
            )
        };

        let new_runs = set_of_indices.len();

        // just try all of them in parallel
        set_of_indices
            .par_iter()
            .map(|use_calls| {
                self.set_fails_successfully(
                    executor.clone(),
                    calls,
                    use_calls,
                    Arc::clone(&shrunk_call_indices),
                )
            })
            .collect::<eyre::Result<()>>()?;

        // There are no more live references to shrunk_call_indices as the parallel execution is
        // finished, so it is fine to get the inner value via `Arc::unwrap`.
        let shrunk_call_indices = Arc::try_unwrap(shrunk_call_indices).unwrap().into_inner();

        if is_powerset {
            // A powerset is guaranteed to be smallest local subset, so we return early.
            return Ok(shrunk_call_indices);
        }

        let computation_budget_not_hit = new_runs + runs < self.shrink_run_limit;
        // If the new shrunk_call_indices is less than the input calls length,
        // we found a subsequence that is shorter. So we can measure if we made progress by
        // comparing them
        let made_progress = shrunk_call_indices.len() < calls.len();
        // We limit the number of times we can iterate without making progress
        let has_remaining_retries = retries <= 3;

        match (computation_budget_not_hit, made_progress) {
            (true, false) if has_remaining_retries => {
                // we havent hit the computation budget and we have retries remaining
                //
                // use the same call set but increase retries which should select a different random
                // subset we dont need to do the mapping stuff like above because we dont
                // take a subset of the input
                self.try_shrinking_recurse(calls, executor, runs + new_runs, retries + 1)
            }
            (true, true) => {
                // We construct a *new* subset of calls using the `shrunk_call_indices` of the
                // passed in calls i.e. if shrunk_call_indices == [1, 3], and calls
                // is: [call0, call1, call2, call3] then new_calls == [call1, call3]
                let new_calls: Vec<_> = calls
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| shrunk_call_indices.contains(i))
                    .map(|(_, call)| call.clone())
                    .collect();

                // We rerun this algorithm as if the new smaller subset above were the original
                // calls. i.e. if [call0, call1, call2, call3] got reduced to
                // [call1, call3] (in the above line) and we still have progress
                // to make, we recall this function with [call1, call3]. Then after this call say it
                // returns [1]. This means `call3` is all that is required to break
                // the invariant.
                let new_calls_idxs =
                    self.try_shrinking_recurse(&new_calls, executor, runs + new_runs, 0)?;

                // Notably, the indices returned above are relative to `new_calls`, *not* the
                // originally passed in `calls`. So we map back by filtering
                // `new_calls` by index if the index was returned above, and finding the position
                // of the `new_call` in the passed in `call`
                Ok(new_calls
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, new_call)| {
                        if !new_calls_idxs.contains(&idx) {
                            None
                        } else {
                            calls.iter().position(|r| r == new_call)
                        }
                    })
                    .collect())
            }
            _ => {
                // The computation budget has been hit or no retries remaining, stop trying to make
                // progress
                Ok(shrunk_call_indices)
            }
        }
    }
}

/// Sets up the calls generated by the internal fuzzer, if they exist.
fn set_up_inner_replay(executor: &mut Executor, inner_sequence: &[Option<BasicTxDetails>]) {
    if let Some(fuzzer) = &mut executor.inspector.fuzzer {
        if let Some(call_generator) = &mut fuzzer.call_generator {
            call_generator.last_sequence = Arc::new(RwLock::new(inner_sequence.to_owned()));
            call_generator.set_replay(true);
        }
    }
}
