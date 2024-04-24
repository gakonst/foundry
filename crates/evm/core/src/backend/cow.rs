//! A wrapper around `Backend` that is clone-on-write used for fuzzing.

use crate::{
    backend::{
        diagnostic::RevertDiagnostic, error::DatabaseError, Backend, DatabaseExt, LocalForkId,
        RevertSnapshotAction,
    },
    fork::{CreateFork, ForkId},
    InspectorExt,
};
use alloy_genesis::GenesisAccount;
use alloy_primitives::{Address, B256, U256};
use eyre::WrapErr;
use revm::{
    db::DatabaseRef,
    primitives::{
        Account, AccountInfo, Bytecode, Env, EnvWithHandlerCfg, HashMap as Map, ResultAndState,
        SpecId,
    },
    ContextPrecompiles, Database, DatabaseCommit, JournaledState,
};
use std::{borrow::Cow, collections::BTreeMap};

/// A wrapper around `Backend` that ensures only `revm::DatabaseRef` functions are called.
///
/// Any changes made during its existence that affect the caching layer of the underlying Database
/// will result in a clone of the initial Database. Therefore, this backend type is basically
/// a clone-on-write `Backend`, where cloning is only necessary if cheatcodes will modify the
/// `Backend`
///
/// Entire purpose of this type is for fuzzing. A test function fuzzer will repeatedly execute the
/// function via immutable raw (no state changes) calls.
///
/// **N.B.**: we're assuming cheatcodes that alter the state (like multi fork swapping) are niche.
/// If they executed, it will require a clone of the initial input database.
/// This way we can support these cheatcodes cheaply without adding overhead for tests that
/// don't make use of them. Alternatively each test case would require its own `Backend` clone,
/// which would add significant overhead for large fuzz sets even if the Database is not big after
/// setup.
#[derive(Clone, Debug)]
pub struct CowBackend<'a> {
    /// The underlying `Backend`.
    ///
    /// No calls on the `CowBackend` will ever persistently modify the `backend`'s state.
    pub backend: Cow<'a, Backend>,
    /// Keeps track of whether the backed is already initialized
    is_initialized: bool,
    /// The [SpecId] of the current backend.
    spec_id: SpecId,
}

impl<'a> CowBackend<'a> {
    /// Creates a new `CowBackend` with the given `Backend`.
    pub fn new(backend: &'a Backend) -> Self {
        Self { backend: Cow::Borrowed(backend), is_initialized: false, spec_id: SpecId::LATEST }
    }

    /// Executes the configured transaction of the `env` without committing state changes
    ///
    /// Note: in case there are any cheatcodes executed that modify the environment, this will
    /// update the given `env` with the new values.
    pub fn inspect<'b, I: InspectorExt<&'b mut Self>>(
        &'b mut self,
        env: &mut EnvWithHandlerCfg,
        inspector: I,
    ) -> eyre::Result<ResultAndState> {
        // this is a new call to inspect with a new env, so even if we've cloned the backend
        // already, we reset the initialized state
        self.is_initialized = false;
        self.spec_id = env.handler_cfg.spec_id;
        let mut evm = crate::utils::new_extended_evm_with_inspector(self, env.clone(), inspector);

        let res = evm.transact().wrap_err("backend: failed while inspecting")?;

        env.env = evm.context.evm.inner.env;

        Ok(res)
    }

    /// Returns whether there was a snapshot failure in the backend.
    ///
    /// This is bubbled up from the underlying Copy-On-Write backend when a revert occurs.
    pub fn has_snapshot_failure(&self) -> bool {
        self.backend.has_snapshot_failure()
    }

    /// Returns a mutable instance of the Backend.
    ///
    /// If this is the first time this is called, the backed is cloned and initialized.
    fn backend_mut(&mut self, env: &Env) -> &mut Backend {
        if !self.is_initialized {
            let backend = self.backend.to_mut();
            let env = EnvWithHandlerCfg::new_with_spec_id(Box::new(env.clone()), self.spec_id);
            backend.initialize(&env);
            self.is_initialized = true;
            return backend
        }
        self.backend.to_mut()
    }

    /// Returns a mutable instance of the Backend if it is initialized.
    fn initialized_backend_mut(&mut self) -> Option<&mut Backend> {
        if self.is_initialized {
            return Some(self.backend.to_mut())
        }
        None
    }
}

impl<'a> DatabaseExt for CowBackend<'a> {
    fn snapshot(&mut self, journaled_state: &JournaledState, env: &Env) -> U256 {
        self.backend_mut(env).snapshot(journaled_state, env)
    }

    fn revert(
        &mut self,
        id: U256,
        journaled_state: &JournaledState,
        current: &mut Env,
        action: RevertSnapshotAction,
    ) -> Option<JournaledState> {
        self.backend_mut(current).revert(id, journaled_state, current, action)
    }

    fn delete_snapshot(&mut self, id: U256) -> bool {
        // delete snapshot requires a previous snapshot to be initialized
        if let Some(backend) = self.initialized_backend_mut() {
            return backend.delete_snapshot(id)
        }
        false
    }

    fn delete_snapshots(&mut self) {
        if let Some(backend) = self.initialized_backend_mut() {
            backend.delete_snapshots()
        }
    }

    fn create_fork(&mut self, fork: CreateFork) -> eyre::Result<LocalForkId> {
        self.backend.to_mut().create_fork(fork)
    }

    fn create_fork_at_transaction(
        &mut self,
        fork: CreateFork,
        transaction: B256,
    ) -> eyre::Result<LocalForkId> {
        self.backend.to_mut().create_fork_at_transaction(fork, transaction)
    }

    fn select_fork<DB: DatabaseExt>(
        &mut self,
        id: LocalForkId,
        env: &mut Env,
        precompiles: &mut ContextPrecompiles<DB>,
        journaled_state: &mut JournaledState,
    ) -> eyre::Result<()> {
        self.backend_mut(env).select_fork(id, env, precompiles, journaled_state)
    }

    fn roll_fork(
        &mut self,
        id: Option<LocalForkId>,
        block_number: u64,
        env: &mut Env,
        journaled_state: &mut JournaledState,
    ) -> eyre::Result<()> {
        self.backend_mut(env).roll_fork(id, block_number, env, journaled_state)
    }

    fn roll_fork_to_transaction(
        &mut self,
        id: Option<LocalForkId>,
        transaction: B256,
        env: &mut Env,
        journaled_state: &mut JournaledState,
    ) -> eyre::Result<()> {
        self.backend_mut(env).roll_fork_to_transaction(id, transaction, env, journaled_state)
    }

    fn transact<I: InspectorExt<Backend>>(
        &mut self,
        id: Option<LocalForkId>,
        transaction: B256,
        env: &mut Env,
        journaled_state: &mut JournaledState,
        inspector: &mut I,
    ) -> eyre::Result<()> {
        self.backend_mut(env).transact(id, transaction, env, journaled_state, inspector)
    }

    fn active_fork_id(&self) -> Option<LocalForkId> {
        self.backend.active_fork_id()
    }

    fn active_fork_url(&self) -> Option<String> {
        self.backend.active_fork_url()
    }

    fn ensure_fork(&self, id: Option<LocalForkId>) -> eyre::Result<LocalForkId> {
        self.backend.ensure_fork(id)
    }

    fn ensure_fork_id(&self, id: LocalForkId) -> eyre::Result<&ForkId> {
        self.backend.ensure_fork_id(id)
    }

    fn diagnose_revert(
        &self,
        callee: Address,
        journaled_state: &JournaledState,
    ) -> Option<RevertDiagnostic> {
        self.backend.diagnose_revert(callee, journaled_state)
    }

    fn load_allocs(
        &mut self,
        allocs: &BTreeMap<Address, GenesisAccount>,
        journaled_state: &mut JournaledState,
    ) -> Result<(), DatabaseError> {
        self.backend_mut(&Env::default()).load_allocs(allocs, journaled_state)
    }

    fn is_persistent(&self, acc: &Address) -> bool {
        self.backend.is_persistent(acc)
    }

    fn remove_persistent_account(&mut self, account: &Address) -> bool {
        self.backend.to_mut().remove_persistent_account(account)
    }

    fn add_persistent_account(&mut self, account: Address) -> bool {
        self.backend.to_mut().add_persistent_account(account)
    }

    fn allow_cheatcode_access(&mut self, account: Address) -> bool {
        self.backend.to_mut().allow_cheatcode_access(account)
    }

    fn revoke_cheatcode_access(&mut self, account: &Address) -> bool {
        self.backend.to_mut().revoke_cheatcode_access(account)
    }

    fn has_cheatcode_access(&self, account: &Address) -> bool {
        self.backend.has_cheatcode_access(account)
    }

    fn get_block_number(&self, env: &Env) -> Result<U256, DatabaseError> {
        self.backend.get_block_number(env)
    }
}

impl<'a> DatabaseRef for CowBackend<'a> {
    type Error = DatabaseError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        DatabaseRef::basic_ref(self.backend.as_ref(), address)
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        DatabaseRef::code_by_hash_ref(self.backend.as_ref(), code_hash)
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        DatabaseRef::storage_ref(self.backend.as_ref(), address, index)
    }

    fn block_hash_ref(&self, number: U256) -> Result<B256, Self::Error> {
        DatabaseRef::block_hash_ref(self.backend.as_ref(), number)
    }
}

impl<'a> Database for CowBackend<'a> {
    type Error = DatabaseError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        DatabaseRef::basic_ref(self, address)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        DatabaseRef::code_by_hash_ref(self, code_hash)
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        DatabaseRef::storage_ref(self, address, index)
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        DatabaseRef::block_hash_ref(self, number)
    }
}

impl<'a> DatabaseCommit for CowBackend<'a> {
    fn commit(&mut self, changes: Map<Address, Account>) {
        self.backend.to_mut().commit(changes)
    }
}
