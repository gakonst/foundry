use super::{BasicTxDetails, CallDetails};
use alloy_primitives::{Address, U256};
use parking_lot::{Mutex, RwLock};
use proptest::{
    option::weighted,
    strategy::{SBoxedStrategy, Strategy, ValueTree},
    test_runner::TestRunner,
};
use std::sync::Arc;

/// Given a TestRunner and a strategy, it generates calls. Used inside the Fuzzer inspector to
/// override external calls to test for potential reentrancy vulnerabilities..
#[derive(Clone, Debug)]
pub struct RandomCallGenerator {
    /// Address of the test contract.
    pub test_address: Address,
    /// Runner that will generate the call from the strategy.
    pub runner: Arc<Mutex<TestRunner>>,
    /// Strategy to be used to generate calls from `target_reference`.
    pub strategy: SBoxedStrategy<Option<CallDetails>>,
    /// Reference to which contract we want a fuzzed calldata from.
    pub target_reference: Arc<RwLock<Address>>,
    /// Flag to know if a call has been overridden. Don't allow nesting for now.
    pub used: bool,
    /// If set to `true`, consumes the next call from `last_sequence`, otherwise queries it from
    /// the strategy.
    pub replay: bool,
    /// Saves the sequence of generated calls that can be replayed later on.
    pub last_sequence: Arc<RwLock<Vec<Option<BasicTxDetails>>>>,
}

impl RandomCallGenerator {
    pub fn new(
        test_address: Address,
        runner: TestRunner,
        strategy: impl Strategy<Value = CallDetails> + Send + Sync + 'static,
        target_reference: Arc<RwLock<Address>>,
    ) -> Self {
        Self {
            test_address,
            runner: Arc::new(Mutex::new(runner)),
            strategy: weighted(0.9, strategy).sboxed(),
            target_reference,
            last_sequence: Arc::default(),
            replay: false,
            used: false,
        }
    }

    /// All `self.next()` calls will now pop `self.last_sequence`. Used to replay an invariant
    /// failure.
    pub fn set_replay(&mut self, status: bool) {
        self.replay = status;
        if status {
            // So it can later be popped.
            self.last_sequence.write().reverse();
        }
    }

    /// Gets the next call. Random if replay is not set. Otherwise, it pops from `last_sequence`.
    pub fn next(
        &mut self,
        original_caller: Address,
        original_target: Address,
    ) -> Option<BasicTxDetails> {
        if self.replay {
            self.last_sequence.write().pop().expect(
                "to have same size as the number of (unsafe) external calls of the sequence.",
            )
        } else {
            // TODO: Do we want it to be 80% chance only too ?
            let sender = original_target;

            // Set which contract we mostly (80% chance) want to generate calldata from.
            *self.target_reference.write() = original_caller;

            // `original_caller` has a 80% chance of being the `new_target`.
            // TODO: Support msg.value > 0 for call_override
            let choice = self
                .strategy
                .new_tree(&mut self.runner.lock())
                .unwrap()
                .current()
                .map(|call_details| BasicTxDetails { sender, call_details, value: U256::ZERO });

            self.last_sequence.write().push(choice.clone());
            choice
        }
    }
}
