//! Foundry's main executor backend abstraction and implementation.

use crate::{
    arbitrum::{self, ARB_SYS_ADDRESS},
    constants::{CALLER, CHEATCODE_ADDRESS, DEFAULT_CREATE2_DEPLOYER, TEST_CONTRACT_ADDRESS},
    fork::{CreateFork, ForkId, MultiFork, SharedBackend},
    snapshot::Snapshots,
    utils::configure_tx_env,
    InspectorExt,
};
use alloy_genesis::GenesisAccount;
use alloy_primitives::{b256, keccak256, Address, B256, U256};
use alloy_rpc_types::{Block, BlockNumberOrTag, BlockTransactions, Transaction, WithOtherFields};
use eyre::Context;
use foundry_common::{is_known_system_sender, SYSTEM_TRANSACTION_TYPE};
use revm::{
    db::{CacheDB, DatabaseRef},
    inspectors::NoOpInspector,
    precompile::{PrecompileSpecId, Precompiles},
    primitives::{
        Account, AccountInfo, Bytecode, CreateScheme, Env, EnvWithHandlerCfg, HashMap as Map, Log,
        ResultAndState, SpecId, State, StorageSlot, TransactTo, KECCAK_EMPTY,
    },
    ContextPrecompile, ContextPrecompiles, Database, DatabaseCommit, JournaledState,
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
    time::Instant,
};

mod diagnostic;
pub use diagnostic::RevertDiagnostic;

mod error;
pub use error::{DatabaseError, DatabaseResult};

mod cow;
pub use cow::CowBackend;

mod in_memory_db;
pub use in_memory_db::{EmptyDBWrapper, FoundryEvmInMemoryDB, MemDb};

mod snapshot;
pub use snapshot::{BackendSnapshot, RevertSnapshotAction, StateSnapshot};

// A `revm::Database` that is used in forking mode
pub type ForkDB = CacheDB<SharedBackend>;

/// Represents a numeric `ForkId` valid only for the existence of the `Backend`.
/// The difference between `ForkId` and `LocalForkId` is that `ForkId` tracks pairs of `endpoint +
/// block` which can be reused by multiple tests, whereas the `LocalForkId` is unique within a test
pub type LocalForkId = U256;

/// Represents the index of a fork in the created forks vector
/// This is used for fast lookup
type ForkLookupIndex = usize;

/// All accounts that will have persistent storage across fork swaps. See also [`clone_data()`]
const DEFAULT_PERSISTENT_ACCOUNTS: [Address; 3] =
    [CHEATCODE_ADDRESS, DEFAULT_CREATE2_DEPLOYER, CALLER];

/// Slot corresponding to "failed" in bytes on the cheatcodes (HEVM) address.
/// Not prefixed with 0x.
const GLOBAL_FAILURE_SLOT: B256 =
    b256!("6661696c65640000000000000000000000000000000000000000000000000000");

/// An extension trait that allows us to easily extend the `revm::Inspector` capabilities
#[auto_impl::auto_impl(&mut)]
pub trait DatabaseExt: Database<Error = DatabaseError> {
    /// Creates a new snapshot at the current point of execution.
    ///
    /// A snapshot is associated with a new unique id that's created for the snapshot.
    /// Snapshots can be reverted: [DatabaseExt::revert], however, depending on the
    /// [RevertSnapshotAction], it will keep the snapshot alive or delete it.
    fn snapshot(&mut self, journaled_state: &JournaledState, env: &Env) -> U256;

    /// Reverts the snapshot if it exists
    ///
    /// Returns `true` if the snapshot was successfully reverted, `false` if no snapshot for that id
    /// exists.
    ///
    /// **N.B.** While this reverts the state of the evm to the snapshot, it keeps new logs made
    /// since the snapshots was created. This way we can show logs that were emitted between
    /// snapshot and its revert.
    /// This will also revert any changes in the `Env` and replace it with the captured `Env` of
    /// `Self::snapshot`.
    ///
    /// Depending on [RevertSnapshotAction] it will keep the snapshot alive or delete it.
    fn revert(
        &mut self,
        id: U256,
        journaled_state: &JournaledState,
        env: &mut Env,
        action: RevertSnapshotAction,
    ) -> Option<JournaledState>;

    /// Deletes the snapshot with the given `id`
    ///
    /// Returns `true` if the snapshot was successfully deleted, `false` if no snapshot for that id
    /// exists.
    fn delete_snapshot(&mut self, id: U256) -> bool;

    /// Deletes all snapshots.
    fn delete_snapshots(&mut self);

    /// Creates and also selects a new fork
    ///
    /// This is basically `create_fork` + `select_fork`
    fn create_select_fork<DB: DatabaseExt>(
        &mut self,
        fork: CreateFork,
        env: &mut Env,
        precompiles: &mut ContextPrecompiles<DB>,
        journaled_state: &mut JournaledState,
    ) -> eyre::Result<LocalForkId>
    where
        Self: Sized,
    {
        let id = self.create_fork(fork)?;
        self.select_fork(id, env, precompiles, journaled_state)?;
        Ok(id)
    }

    /// Creates and also selects a new fork
    ///
    /// This is basically `create_fork` + `select_fork`
    fn create_select_fork_at_transaction<DB: DatabaseExt>(
        &mut self,
        fork: CreateFork,
        env: &mut Env,
        journaled_state: &mut JournaledState,
        precompiles: &mut ContextPrecompiles<DB>,
        transaction: B256,
    ) -> eyre::Result<LocalForkId>
    where
        Self: Sized,
    {
        let id = self.create_fork_at_transaction(fork, transaction)?;
        self.select_fork(id, env, precompiles, journaled_state)?;
        Ok(id)
    }

    /// Creates a new fork but does _not_ select it
    fn create_fork(&mut self, fork: CreateFork) -> eyre::Result<LocalForkId>;

    /// Creates a new fork but does _not_ select it
    fn create_fork_at_transaction(
        &mut self,
        fork: CreateFork,
        transaction: B256,
    ) -> eyre::Result<LocalForkId>;

    /// Selects the fork's state
    ///
    /// This will also modify the current `Env`.
    ///
    /// **Note**: this does not change the local state, but swaps the remote state
    ///
    /// # Errors
    ///
    /// Returns an error if no fork with the given `id` exists
    fn select_fork<DB: DatabaseExt>(
        &mut self,
        id: LocalForkId,
        env: &mut Env,
        precompiles: &mut ContextPrecompiles<DB>,
        journaled_state: &mut JournaledState,
    ) -> eyre::Result<()>
    where
        Self: Sized;

    /// Updates the fork to given block number.
    ///
    /// This will essentially create a new fork at the given block height.
    ///
    /// # Errors
    ///
    /// Returns an error if not matching fork was found.
    fn roll_fork(
        &mut self,
        id: Option<LocalForkId>,
        block_number: u64,
        env: &mut Env,
        journaled_state: &mut JournaledState,
    ) -> eyre::Result<()>;

    /// Updates the fork to given transaction hash
    ///
    /// This will essentially create a new fork at the block this transaction was mined and replays
    /// all transactions up until the given transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if not matching fork was found.
    fn roll_fork_to_transaction(
        &mut self,
        id: Option<LocalForkId>,
        transaction: B256,
        env: &mut Env,
        journaled_state: &mut JournaledState,
    ) -> eyre::Result<()>;

    /// Fetches the given transaction for the fork and executes it, committing the state in the DB
    fn transact<I: InspectorExt<Backend>>(
        &mut self,
        id: Option<LocalForkId>,
        transaction: B256,
        env: &mut Env,
        journaled_state: &mut JournaledState,
        inspector: &mut I,
    ) -> eyre::Result<()>
    where
        Self: Sized;

    /// Returns the `ForkId` that's currently used in the database, if fork mode is on
    fn active_fork_id(&self) -> Option<LocalForkId>;

    /// Returns the Fork url that's currently used in the database, if fork mode is on
    fn active_fork_url(&self) -> Option<String>;

    /// Whether the database is currently in forked mode.
    fn is_forked_mode(&self) -> bool {
        self.active_fork_id().is_some()
    }

    /// Ensures that an appropriate fork exists
    ///
    /// If `id` contains a requested `Fork` this will ensure it exists.
    /// Otherwise, this returns the currently active fork.
    ///
    /// # Errors
    ///
    /// Returns an error if the given `id` does not match any forks
    ///
    /// Returns an error if no fork exists
    fn ensure_fork(&self, id: Option<LocalForkId>) -> eyre::Result<LocalForkId>;

    /// Ensures that a corresponding `ForkId` exists for the given local `id`
    fn ensure_fork_id(&self, id: LocalForkId) -> eyre::Result<&ForkId>;

    /// Handling multiple accounts/new contracts in a multifork environment can be challenging since
    /// every fork has its own standalone storage section. So this can be a common error to run
    /// into:
    ///
    /// ```solidity
    /// function testCanDeploy() public {
    ///    vm.selectFork(mainnetFork);
    ///    // contract created while on `mainnetFork`
    ///    DummyContract dummy = new DummyContract();
    ///    // this will succeed
    ///    dummy.hello();
    ///
    ///    vm.selectFork(optimismFork);
    ///
    ///    vm.expectRevert();
    ///    // this will revert since `dummy` contract only exists on `mainnetFork`
    ///    dummy.hello();
    /// }
    /// ```
    ///
    /// If this happens (`dummy.hello()`), or more general, a call on an address that's not a
    /// contract, revm will revert without useful context. This call will check in this context if
    /// `address(dummy)` belongs to an existing contract and if not will check all other forks if
    /// the contract is deployed there.
    ///
    /// Returns a more useful error message if that's the case
    fn diagnose_revert(
        &self,
        callee: Address,
        journaled_state: &JournaledState,
    ) -> Option<RevertDiagnostic>;

    /// Loads the account allocs from the given `allocs` map into the passed [JournaledState].
    ///
    /// Returns [Ok] if all accounts were successfully inserted into the journal, [Err] otherwise.
    fn load_allocs(
        &mut self,
        allocs: &BTreeMap<Address, GenesisAccount>,
        journaled_state: &mut JournaledState,
    ) -> Result<(), DatabaseError>;

    /// Returns true if the given account is currently marked as persistent.
    fn is_persistent(&self, acc: &Address) -> bool;

    /// Revokes persistent status from the given account.
    fn remove_persistent_account(&mut self, account: &Address) -> bool;

    /// Marks the given account as persistent.
    fn add_persistent_account(&mut self, account: Address) -> bool;

    /// Removes persistent status from all given accounts
    fn remove_persistent_accounts(&mut self, accounts: impl IntoIterator<Item = Address>)
    where
        Self: Sized,
    {
        for acc in accounts {
            self.remove_persistent_account(&acc);
        }
    }

    /// Extends the persistent accounts with the accounts the iterator yields.
    fn extend_persistent_accounts(&mut self, accounts: impl IntoIterator<Item = Address>)
    where
        Self: Sized,
    {
        for acc in accounts {
            self.add_persistent_account(acc);
        }
    }

    /// Grants cheatcode access for the given `account`
    ///
    /// Returns true if the `account` already has access
    fn allow_cheatcode_access(&mut self, account: Address) -> bool;

    /// Revokes cheatcode access for the given account
    ///
    /// Returns true if the `account` was previously allowed cheatcode access
    fn revoke_cheatcode_access(&mut self, account: &Address) -> bool;

    /// Returns `true` if the given account is allowed to execute cheatcodes
    fn has_cheatcode_access(&self, account: &Address) -> bool;

    /// Ensures that `account` is allowed to execute cheatcodes
    ///
    /// Returns an error if [`Self::has_cheatcode_access`] returns `false`
    fn ensure_cheatcode_access(&self, account: &Address) -> Result<(), DatabaseError> {
        if !self.has_cheatcode_access(account) {
            return Err(DatabaseError::NoCheats(*account));
        }
        Ok(())
    }

    /// Same as [`Self::ensure_cheatcode_access()`] but only enforces it if the backend is currently
    /// in forking mode
    fn ensure_cheatcode_access_forking_mode(&self, account: &Address) -> Result<(), DatabaseError> {
        if self.is_forked_mode() {
            return self.ensure_cheatcode_access(account);
        }
        Ok(())
    }

    /// NUMBER opcode handler accounting for potential chain-specifics.
    fn get_block_number(&self, env: &Env) -> Result<U256, DatabaseError>;
}

struct _ObjectSafe(dyn DatabaseExt);

/// Provides the underlying `revm::Database` implementation.
///
/// A `Backend` can be initialised in two forms:
///
/// # 1. Empty in-memory Database
/// This is the default variant: an empty `revm::Database`
///
/// # 2. Forked Database
/// A `revm::Database` that forks off a remote client
///
///
/// In addition to that we support forking manually on the fly.
/// Additional forks can be created. Each unique fork is identified by its unique `ForkId`. We treat
/// forks as unique if they have the same `(endpoint, block number)` pair.
///
/// When it comes to testing, it's intended that each contract will use its own `Backend`
/// (`Backend::clone`). This way each contract uses its own encapsulated evm state. For in-memory
/// testing, the database is just an owned `revm::InMemoryDB`.
///
/// Each `Fork`, identified by a unique id, uses completely separate storage, write operations are
/// performed only in the fork's own database, `ForkDB`.
///
/// A `ForkDB` consists of 2 halves:
///   - everything fetched from the remote is readonly
///   - all local changes (instructed by the contract) are written to the backend's `db` and don't
///     alter the state of the remote client.
///
/// # Fork swapping
///
/// Multiple "forks" can be created `Backend::create_fork()`, however only 1 can be used by the
/// `db`. However, their state can be hot-swapped by swapping the read half of `db` from one fork to
/// another.
/// When swapping forks (`Backend::select_fork()`) we also update the current `Env` of the `EVM`
/// accordingly, so that all `block.*` config values match
///
/// When another for is selected [`DatabaseExt::select_fork()`] the entire storage, including
/// `JournaledState` is swapped, but the storage of the caller's and the test contract account is
/// _always_ cloned. This way a fork has entirely separate storage but data can still be shared
/// across fork boundaries via stack and contract variables.
///
/// # Snapshotting
///
/// A snapshot of the current overall state can be taken at any point in time. A snapshot is
/// identified by a unique id that's returned when a snapshot is created. A snapshot can only be
/// reverted _once_. After a successful revert, the same snapshot id cannot be used again. Reverting
/// a snapshot replaces the current active state with the snapshot state, the snapshot is deleted
/// afterwards, as well as any snapshots taken after the reverted snapshot, (e.g.: reverting to id
/// 0x1 will delete snapshots with ids 0x1, 0x2, etc.)
///
/// **Note:** Snapshots work across fork-swaps, e.g. if fork `A` is currently active, then a
/// snapshot is created before fork `B` is selected, then fork `A` will be the active fork again
/// after reverting the snapshot.
#[derive(Clone, Debug)]
pub struct Backend {
    /// The access point for managing forks
    forks: MultiFork,
    // The default in memory db
    mem_db: FoundryEvmInMemoryDB,
    /// The journaled_state to use to initialize new forks with
    ///
    /// The way [`revm::JournaledState`] works is, that it holds the "hot" accounts loaded from the
    /// underlying `Database` that feeds the Account and State data ([`revm::AccountInfo`])to the
    /// journaled_state so it can apply changes to the state while the evm executes.
    ///
    /// In a way the `JournaledState` is something like a cache that
    /// 1. check if account is already loaded (hot)
    /// 2. if not load from the `Database` (this will then retrieve the account via RPC in forking
    /// mode)
    ///
    /// To properly initialize we store the `JournaledState` before the first fork is selected
    /// ([`DatabaseExt::select_fork`]).
    ///
    /// This will be an empty `JournaledState`, which will be populated with persistent accounts,
    /// See [`Self::update_fork_db()`] and [`clone_data()`].
    fork_init_journaled_state: JournaledState,
    /// The currently active fork database
    ///
    /// If this is set, then the Backend is currently in forking mode
    active_fork_ids: Option<(LocalForkId, ForkLookupIndex)>,
    /// holds additional Backend data
    inner: BackendInner,
}

// === impl Backend ===

impl Backend {
    /// Creates a new Backend with a spawned multi fork thread.
    pub fn spawn(fork: Option<CreateFork>) -> Self {
        Self::new(MultiFork::spawn(), fork)
    }

    /// Creates a new instance of `Backend`
    ///
    /// if `fork` is `Some` this will launch with a `fork` database, otherwise with an in-memory
    /// database
    pub fn new(forks: MultiFork, fork: Option<CreateFork>) -> Self {
        trace!(target: "backend", forking_mode=?fork.is_some(), "creating executor backend");
        // Note: this will take of registering the `fork`
        let inner = BackendInner {
            persistent_accounts: HashSet::from(DEFAULT_PERSISTENT_ACCOUNTS),
            ..Default::default()
        };

        let mut backend = Self {
            forks,
            mem_db: CacheDB::new(Default::default()),
            fork_init_journaled_state: inner.new_journaled_state(),
            active_fork_ids: None,
            inner,
        };

        if let Some(fork) = fork {
            let (fork_id, fork, fork_env) =
                backend.forks.create_fork(fork).expect("Unable to create fork");
            let precompiles = ForkSpecificPrecompiles::new(fork_env.cfg.chain_id);
            let fork_db = ForkDB::new(fork);
            let fork_ids = backend.inner.insert_new_fork(
                fork_id.clone(),
                fork_db,
                backend.inner.new_journaled_state(),
                precompiles,
            );
            backend.inner.launched_with_fork = Some((fork_id, fork_ids.0, fork_ids.1));
            backend.active_fork_ids = Some(fork_ids);
        }

        trace!(target: "backend", forking_mode=? backend.active_fork_ids.is_some(), "created executor backend");

        backend
    }

    /// Creates a new instance of `Backend` with fork added to the fork database and sets the fork
    /// as active
    pub(crate) fn new_with_fork(id: &ForkId, fork: Fork, journaled_state: JournaledState) -> Self {
        let mut backend = Self::spawn(None);
        let fork_ids =
            backend.inner.insert_new_fork(id.clone(), fork.db, journaled_state, fork.precompiles);
        backend.inner.launched_with_fork = Some((id.clone(), fork_ids.0, fork_ids.1));
        backend.active_fork_ids = Some(fork_ids);
        backend
    }

    /// Creates a new instance with a `BackendDatabase::InMemory` cache layer for the `CacheDB`
    pub fn clone_empty(&self) -> Self {
        Self {
            forks: self.forks.clone(),
            mem_db: CacheDB::new(Default::default()),
            fork_init_journaled_state: self.inner.new_journaled_state(),
            active_fork_ids: None,
            inner: Default::default(),
        }
    }

    pub fn insert_account_info(&mut self, address: Address, account: AccountInfo) {
        if let Some(db) = self.active_fork_db_mut() {
            db.insert_account_info(address, account)
        } else {
            self.mem_db.insert_account_info(address, account)
        }
    }

    /// Inserts a value on an account's storage without overriding account info
    pub fn insert_account_storage(
        &mut self,
        address: Address,
        slot: U256,
        value: U256,
    ) -> Result<(), DatabaseError> {
        let ret = if let Some(db) = self.active_fork_db_mut() {
            db.insert_account_storage(address, slot, value)
        } else {
            self.mem_db.insert_account_storage(address, slot, value)
        };

        debug_assert!(self.storage(address, slot).unwrap() == value);

        ret
    }

    /// Completely replace an account's storage without overriding account info.
    ///
    /// When forking, this causes the backend to assume a `0` value for all
    /// unset storage slots instead of trying to fetch it.
    pub fn replace_account_storage(
        &mut self,
        address: Address,
        storage: Map<U256, U256>,
    ) -> Result<(), DatabaseError> {
        if let Some(db) = self.active_fork_db_mut() {
            db.replace_account_storage(address, storage)
        } else {
            self.mem_db.replace_account_storage(address, storage)
        }
    }

    /// Returns all snapshots created in this backend
    pub fn snapshots(&self) -> &Snapshots<BackendSnapshot<BackendDatabaseSnapshot>> {
        &self.inner.snapshots
    }

    /// Sets the address of the `DSTest` contract that is being executed
    ///
    /// This will also mark the caller as persistent and remove the persistent status from the
    /// previous test contract address
    ///
    /// This will also grant cheatcode access to the test account
    pub fn set_test_contract(&mut self, acc: Address) -> &mut Self {
        trace!(?acc, "setting test account");
        // toggle the previous sender
        if let Some(current) = self.inner.test_contract_address.take() {
            self.remove_persistent_account(&current);
            self.revoke_cheatcode_access(&acc);
        }

        self.add_persistent_account(acc);
        self.allow_cheatcode_access(acc);
        self.inner.test_contract_address = Some(acc);
        self
    }

    /// Sets the caller address
    pub fn set_caller(&mut self, acc: Address) -> &mut Self {
        trace!(?acc, "setting caller account");
        self.inner.caller = Some(acc);
        self.allow_cheatcode_access(acc);
        self
    }

    /// Sets the current spec id
    pub fn set_spec_id(&mut self, spec_id: SpecId) -> &mut Self {
        trace!(?spec_id, "setting spec ID");
        self.inner.spec_id = spec_id;
        self
    }

    /// Returns the address of the set `DSTest` contract
    pub fn test_contract_address(&self) -> Option<Address> {
        self.inner.test_contract_address
    }

    /// Returns the set caller address
    pub fn caller_address(&self) -> Option<Address> {
        self.inner.caller
    }

    /// Failures occurred in snapshots are tracked when the snapshot is reverted
    ///
    /// If an error occurs in a restored snapshot, the test is considered failed.
    ///
    /// This returns whether there was a reverted snapshot that recorded an error
    pub fn has_snapshot_failure(&self) -> bool {
        self.inner.has_snapshot_failure
    }

    /// Sets the snapshot failure flag.
    pub fn set_snapshot_failure(&mut self, has_snapshot_failure: bool) {
        self.inner.has_snapshot_failure = has_snapshot_failure
    }

    /// Checks if the test contract associated with this backend failed, See
    /// [Self::is_failed_test_contract]
    pub fn is_failed(&self) -> bool {
        self.has_snapshot_failure() ||
            self.test_contract_address()
                .map(|addr| self.is_failed_test_contract(addr))
                .unwrap_or_default()
    }

    /// Checks if the given test function failed
    ///
    /// DSTest will not revert inside its `assertEq`-like functions which allows
    /// to test multiple assertions in 1 test function while also preserving logs.
    /// Instead, it stores whether an `assert` failed in a boolean variable that we can read
    pub fn is_failed_test_contract(&self, address: Address) -> bool {
        /*
         contract DSTest {
            bool public IS_TEST = true;
            // slot 0 offset 1 => second byte of slot0
            bool private _failed;
         }
        */
        let value = self.storage_ref(address, U256::ZERO).unwrap_or_default();
        value.as_le_bytes()[1] != 0
    }

    /// Checks if the given test function failed by looking at the present value of the test
    /// contract's `JournaledState`
    ///
    /// See [`Self::is_failed_test_contract()]`
    ///
    /// Note: we assume the test contract is either `forge-std/Test` or `DSTest`
    pub fn is_failed_test_contract_state(
        &self,
        address: Address,
        current_state: &JournaledState,
    ) -> bool {
        if let Some(account) = current_state.state.get(&address) {
            let value = account
                .storage
                .get(&revm::primitives::U256::ZERO)
                .cloned()
                .unwrap_or_default()
                .present_value();
            return value.as_le_bytes()[1] != 0;
        }

        false
    }

    /// In addition to the `_failed` variable, `DSTest::fail()` stores a failure
    /// in "failed"
    /// See <https://github.com/dapphub/ds-test/blob/9310e879db8ba3ea6d5c6489a579118fd264a3f5/src/test.sol#L66-L72>
    pub fn is_global_failure(&self, current_state: &JournaledState) -> bool {
        if let Some(account) = current_state.state.get(&CHEATCODE_ADDRESS) {
            let slot: U256 = GLOBAL_FAILURE_SLOT.into();
            let value = account.storage.get(&slot).cloned().unwrap_or_default().present_value();
            return value == revm::primitives::U256::from(1);
        }

        false
    }

    /// When creating or switching forks, we update the AccountInfo of the contract
    pub(crate) fn update_fork_db(
        &self,
        active_journaled_state: &mut JournaledState,
        target_fork: &mut Fork,
    ) {
        debug_assert!(
            self.inner.test_contract_address.is_some(),
            "Test contract address must be set"
        );

        self.update_fork_db_contracts(
            self.inner.persistent_accounts.iter().copied(),
            active_journaled_state,
            target_fork,
        )
    }

    /// Merges the state of all `accounts` from the currently active db into the given `fork`
    pub(crate) fn update_fork_db_contracts(
        &self,
        accounts: impl IntoIterator<Item = Address>,
        active_journaled_state: &mut JournaledState,
        target_fork: &mut Fork,
    ) {
        if let Some((_, fork_idx)) = self.active_fork_ids.as_ref() {
            let active = self.inner.get_fork(*fork_idx);
            merge_account_data(accounts, &active.db, active_journaled_state, target_fork)
        } else {
            merge_account_data(accounts, &self.mem_db, active_journaled_state, target_fork)
        }
    }

    /// Returns the memory db used if not in forking mode
    pub fn mem_db(&self) -> &FoundryEvmInMemoryDB {
        &self.mem_db
    }

    /// Returns true if the `id` is currently active
    pub fn is_active_fork(&self, id: LocalForkId) -> bool {
        self.active_fork_ids.map(|(i, _)| i == id).unwrap_or_default()
    }

    /// Returns `true` if the `Backend` is currently in forking mode
    pub fn is_in_forking_mode(&self) -> bool {
        self.active_fork().is_some()
    }

    /// Returns the currently active `Fork`, if any
    pub fn active_fork(&self) -> Option<&Fork> {
        self.active_fork_ids.map(|(_, idx)| self.inner.get_fork(idx))
    }

    /// Returns the currently active `Fork`, if any
    pub fn active_fork_mut(&mut self) -> Option<&mut Fork> {
        self.active_fork_ids.map(|(_, idx)| self.inner.get_fork_mut(idx))
    }

    /// Returns the currently active `ForkDB`, if any
    pub fn active_fork_db(&self) -> Option<&ForkDB> {
        self.active_fork().map(|f| &f.db)
    }

    /// Returns the currently active `ForkDB`, if any
    pub fn active_fork_db_mut(&mut self) -> Option<&mut ForkDB> {
        self.active_fork_mut().map(|f| &mut f.db)
    }

    /// Creates a snapshot of the currently active database
    pub(crate) fn create_db_snapshot(&self) -> BackendDatabaseSnapshot {
        if let Some((id, idx)) = self.active_fork_ids {
            let fork = self.inner.get_fork(idx).clone();
            let fork_id = self.inner.ensure_fork_id(id).cloned().expect("Exists; qed");
            BackendDatabaseSnapshot::Forked(id, fork_id, idx, Box::new(fork))
        } else {
            BackendDatabaseSnapshot::InMemory(self.mem_db.clone())
        }
    }

    /// Since each `Fork` tracks logs separately, we need to merge them to get _all_ of them
    pub fn merged_logs(&self, mut logs: Vec<Log>) -> Vec<Log> {
        if let Some((_, active)) = self.active_fork_ids {
            let mut all_logs = Vec::with_capacity(logs.len());

            self.inner
                .forks
                .iter()
                .enumerate()
                .filter_map(|(idx, f)| f.as_ref().map(|f| (idx, f)))
                .for_each(|(idx, f)| {
                    if idx == active {
                        all_logs.append(&mut logs);
                    } else {
                        all_logs.extend(f.journaled_state.logs.clone())
                    }
                });
            return all_logs;
        }

        logs
    }

    /// Initializes settings we need to keep track of.
    ///
    /// We need to track these mainly to prevent issues when switching between different evms
    pub(crate) fn initialize(&mut self, env: &EnvWithHandlerCfg) {
        self.set_caller(env.tx.caller);
        self.set_spec_id(env.handler_cfg.spec_id);

        let test_contract = match env.tx.transact_to {
            TransactTo::Call(to) => to,
            TransactTo::Create(CreateScheme::Create) => {
                env.tx.caller.create(env.tx.nonce.unwrap_or_default())
            }
            TransactTo::Create(CreateScheme::Create2 { salt }) => {
                let code_hash = B256::from_slice(keccak256(&env.tx.data).as_slice());
                env.tx.caller.create2(B256::from(salt), code_hash)
            }
        };
        self.set_test_contract(test_contract);
    }

    /// Returns the `EnvWithHandlerCfg` with the current `spec_id` set.
    fn env_with_handler_cfg(&self, env: Env) -> EnvWithHandlerCfg {
        EnvWithHandlerCfg::new_with_spec_id(Box::new(env), self.inner.spec_id)
    }

    /// Executes the configured test call of the `env` without committing state changes.
    ///
    /// Note: in case there are any cheatcodes executed that modify the environment, this will
    /// update the given `env` with the new values.
    pub fn inspect<'a, I: InspectorExt<&'a mut Self>>(
        &'a mut self,
        env: &mut EnvWithHandlerCfg,
        inspector: I,
    ) -> eyre::Result<ResultAndState> {
        self.initialize(env);
        let mut evm = crate::utils::new_extended_evm_with_inspector(self, env.clone(), inspector);

        let res = evm.transact().wrap_err("backend: failed while inspecting")?;

        env.env = evm.context.evm.inner.env;

        Ok(res)
    }

    /// Returns true if the address is a precompile
    pub fn is_existing_precompile(&self, addr: &Address) -> bool {
        self.inner.precompiles().contains(addr)
    }

    /// Sets the initial journaled state to use when initializing forks
    #[inline]
    fn set_init_journaled_state(&mut self, journaled_state: JournaledState) {
        trace!("recording fork init journaled_state");
        self.fork_init_journaled_state = journaled_state;
    }

    /// Cleans up already loaded accounts that would be initialized without the correct data from
    /// the fork.
    ///
    /// It can happen that an account is loaded before the first fork is selected, like
    /// `getNonce(addr)`, which will load an empty account by default.
    ///
    /// This account data then would not match the account data of a fork if it exists.
    /// So when the first fork is initialized we replace these accounts with the actual account as
    /// it exists on the fork.
    fn prepare_init_journal_state(&mut self) -> Result<(), DatabaseError> {
        let loaded_accounts = self
            .fork_init_journaled_state
            .state
            .iter()
            .filter(|(addr, _)| !self.is_existing_precompile(addr) && !self.is_persistent(addr))
            .map(|(addr, _)| addr)
            .copied()
            .collect::<Vec<_>>();

        for fork in self.inner.forks_iter_mut() {
            let mut journaled_state = self.fork_init_journaled_state.clone();
            for loaded_account in loaded_accounts.iter().copied() {
                trace!(?loaded_account, "replacing account on init");
                let init_account =
                    journaled_state.state.get_mut(&loaded_account).expect("exists; qed");

                // here's an edge case where we need to check if this account has been created, in
                // which case we don't need to replace it with the account from the fork because the
                // created account takes precedence: for example contract creation in setups
                if init_account.is_created() {
                    trace!(?loaded_account, "skipping created account");
                    continue
                }

                // otherwise we need to replace the account's info with the one from the fork's
                // database
                let fork_account = Database::basic(&mut fork.db, loaded_account)?
                    .ok_or(DatabaseError::MissingAccount(loaded_account))?;
                init_account.info = fork_account;
            }
            fork.journaled_state = journaled_state;
        }
        Ok(())
    }

    /// Returns the block numbers required for replaying a transaction
    fn get_block_number_and_block_for_transaction(
        &self,
        id: LocalForkId,
        transaction: B256,
    ) -> eyre::Result<(u64, Block)> {
        let fork = self.inner.get_fork_by_id(id)?;
        let tx = fork.db.db.get_transaction(transaction)?;

        // get the block number we need to fork
        if let Some(tx_block) = tx.block_number {
            let block = fork.db.db.get_full_block(tx_block)?;

            // we need to subtract 1 here because we want the state before the transaction
            // was mined
            let fork_block = tx_block - 1;
            Ok((fork_block, block))
        } else {
            let block = fork.db.db.get_full_block(BlockNumberOrTag::Latest)?;

            let number = block
                .header
                .number
                .ok_or_else(|| DatabaseError::BlockNotFound(BlockNumberOrTag::Latest.into()))?;

            Ok((number, block))
        }
    }

    /// Replays all the transactions at the forks current block that were mined before the `tx`
    ///
    /// Returns the _unmined_ transaction that corresponds to the given `tx_hash`
    pub fn replay_until(
        &mut self,
        id: LocalForkId,
        env: Env,
        tx_hash: B256,
        journaled_state: &mut JournaledState,
    ) -> eyre::Result<Option<Transaction>> {
        trace!(?id, ?tx_hash, "replay until transaction");

        let fork_id = self.ensure_fork_id(id)?.clone();

        let env = self.env_with_handler_cfg(env);
        let fork = self.inner.get_fork_by_id_mut(id)?;
        let full_block = fork.db.db.get_full_block(env.block.number.to::<u64>())?;

        if let BlockTransactions::Full(txs) = full_block.transactions {
            for tx in txs.into_iter() {
                // System transactions such as on L2s don't contain any pricing info so we skip them
                // otherwise this would cause reverts
                if is_known_system_sender(tx.from) ||
                    tx.transaction_type == Some(SYSTEM_TRANSACTION_TYPE)
                {
                    trace!(tx=?tx.hash, "skipping system transaction");
                    continue;
                }

                if tx.hash == tx_hash {
                    // found the target transaction
                    return Ok(Some(tx))
                }
                trace!(tx=?tx.hash, "committing transaction");

                commit_transaction(
                    WithOtherFields::new(tx),
                    env.clone(),
                    journaled_state,
                    fork,
                    &fork_id,
                    &mut NoOpInspector,
                )?;
            }
        }

        Ok(None)
    }
}

// === impl a bunch of `revm::Database` adjacent implementations ===

impl DatabaseExt for Backend {
    fn snapshot(&mut self, journaled_state: &JournaledState, env: &Env) -> U256 {
        trace!("create snapshot");
        let id = self.inner.snapshots.insert(BackendSnapshot::new(
            self.create_db_snapshot(),
            journaled_state.clone(),
            env.clone(),
        ));
        trace!(target: "backend", "Created new snapshot {}", id);
        id
    }

    fn revert(
        &mut self,
        id: U256,
        current_state: &JournaledState,
        current: &mut Env,
        action: RevertSnapshotAction,
    ) -> Option<JournaledState> {
        trace!(?id, "revert snapshot");
        if let Some(mut snapshot) = self.inner.snapshots.remove_at(id) {
            // Re-insert snapshot to persist it
            if action.is_keep() {
                self.inner.snapshots.insert_at(snapshot.clone(), id);
            }
            // need to check whether there's a global failure which means an error occurred either
            // during the snapshot or even before
            if self.is_global_failure(current_state) {
                self.set_snapshot_failure(true);
            }

            // merge additional logs
            snapshot.merge(current_state);
            let BackendSnapshot { db, mut journaled_state, env } = snapshot;
            match db {
                BackendDatabaseSnapshot::InMemory(mem_db) => {
                    self.mem_db = mem_db;
                }
                BackendDatabaseSnapshot::Forked(id, fork_id, idx, mut fork) => {
                    // there might be the case where the snapshot was created during `setUp` with
                    // another caller, so we need to ensure the caller account is present in the
                    // journaled state and database
                    let caller = current.tx.caller;
                    journaled_state.state.entry(caller).or_insert_with(|| {
                        let caller_account = current_state
                            .state
                            .get(&caller)
                            .map(|acc| acc.info.clone())
                            .unwrap_or_default();

                        if !fork.db.accounts.contains_key(&caller) {
                            // update the caller account which is required by the evm
                            fork.db.insert_account_info(caller, caller_account.clone());
                        }
                        caller_account.into()
                    });
                    self.inner.revert_snapshot(id, fork_id, idx, *fork);
                    self.active_fork_ids = Some((id, idx))
                }
            }

            update_current_env_with_fork_env(current, env);
            trace!(target: "backend", "Reverted snapshot {}", id);

            Some(journaled_state)
        } else {
            warn!(target: "backend", "No snapshot to revert for {}", id);
            None
        }
    }

    fn delete_snapshot(&mut self, id: U256) -> bool {
        self.inner.snapshots.remove_at(id).is_some()
    }

    fn delete_snapshots(&mut self) {
        self.inner.snapshots.clear()
    }

    fn create_fork(&mut self, create_fork: CreateFork) -> eyre::Result<LocalForkId> {
        trace!("create fork");
        let (fork_id, fork, fork_env) = self.forks.create_fork(create_fork)?;

        let fork_db = ForkDB::new(fork);
        let precompiles = ForkSpecificPrecompiles::new(fork_env.cfg.chain_id);
        let (id, _) = self.inner.insert_new_fork(
            fork_id,
            fork_db,
            self.fork_init_journaled_state.clone(),
            precompiles,
        );
        Ok(id)
    }

    fn create_fork_at_transaction(
        &mut self,
        fork: CreateFork,
        transaction: B256,
    ) -> eyre::Result<LocalForkId> {
        trace!(?transaction, "create fork at transaction");
        let id = self.create_fork(fork)?;
        let fork_id = self.ensure_fork_id(id).cloned()?;
        let mut env = self
            .forks
            .get_env(fork_id)?
            .ok_or_else(|| eyre::eyre!("Requested fork `{}` does not exit", id))?;

        // we still need to roll to the transaction, but we only need an empty dummy state since we
        // don't need to update the active journaled state yet
        self.roll_fork_to_transaction(
            Some(id),
            transaction,
            &mut env,
            &mut self.inner.new_journaled_state(),
        )?;
        Ok(id)
    }

    /// Select an existing fork by id.
    /// When switching forks we copy the shared state
    fn select_fork<DB: DatabaseExt>(
        &mut self,
        id: LocalForkId,
        env: &mut Env,
        precompiles: &mut ContextPrecompiles<DB>,
        active_journaled_state: &mut JournaledState,
    ) -> eyre::Result<()> {
        trace!(?id, "select fork");
        if self.is_active_fork(id) {
            // nothing to do
            return Ok(());
        }

        let fork_id = self.ensure_fork_id(id).cloned()?;
        let idx = self.inner.ensure_fork_index(&fork_id)?;
        let fork_env = self
            .forks
            .get_env(fork_id)?
            .ok_or_else(|| eyre::eyre!("Requested fork `{}` does not exit", id))?;

        // If we're currently in forking mode we need to update the journaled_state to this point,
        // this ensures the changes performed while the fork was active are recorded
        if let Some(active) = self.active_fork_mut() {
            active.journaled_state = active_journaled_state.clone();

            for address in active.precompiles.addresses() {
                precompiles.remove(&address);
            }

            let caller = env.tx.caller;
            let caller_account = active.journaled_state.state.get(&env.tx.caller).cloned();
            let target_fork = self.inner.get_fork_mut(idx);

            // depth 0 will be the default value when the fork was created
            if target_fork.journaled_state.depth == 0 {
                // Initialize caller with its fork info
                if let Some(mut acc) = caller_account {
                    let fork_account = Database::basic(&mut target_fork.db, caller)?
                        .ok_or(DatabaseError::MissingAccount(caller))?;

                    acc.info = fork_account;
                    target_fork.journaled_state.state.insert(caller, acc);
                }
            }
        } else {
            // this is the first time a fork is selected. This means up to this point all changes
            // are made in a single `JournaledState`, for example after a `setup` that only created
            // different forks. Since the `JournaledState` is valid for all forks until the
            // first fork is selected, we need to update it for all forks and use it as init state
            // for all future forks

            self.set_init_journaled_state(active_journaled_state.clone());
            self.prepare_init_journal_state()?;

            // Make sure that the next created fork has a depth of 0.
            self.fork_init_journaled_state.depth = 0;
        }

        {
            // update the shared state and track
            let mut fork = self.inner.take_fork(idx);

            // since all forks handle their state separately, the depth can drift
            // this is a handover where the target fork starts at the same depth where it was
            // selected. This ensures that there are no gaps in depth which would
            // otherwise cause issues with the tracer
            fork.journaled_state.depth = active_journaled_state.depth;

            // another edge case where a fork is created and selected during setup with not
            // necessarily the same caller as for the test, however we must always
            // ensure that fork's state contains the current sender
            let caller = env.tx.caller;
            fork.journaled_state.state.entry(caller).or_insert_with(|| {
                let caller_account = active_journaled_state
                    .state
                    .get(&env.tx.caller)
                    .map(|acc| acc.info.clone())
                    .unwrap_or_default();

                if !fork.db.accounts.contains_key(&caller) {
                    // update the caller account which is required by the evm
                    fork.db.insert_account_info(caller, caller_account.clone());
                }
                caller_account.into()
            });

            self.update_fork_db(active_journaled_state, &mut fork);

            fork.precompiles.fill_precompiles(precompiles);

            // insert the fork back
            self.inner.set_fork(idx, fork);
        }

        self.active_fork_ids = Some((id, idx));
        // update the environment accordingly
        update_current_env_with_fork_env(env, fork_env);

        Ok(())
    }

    /// This is effectively the same as [`Self::create_select_fork()`] but updating an existing
    /// [ForkId] that is mapped to the [LocalForkId]
    fn roll_fork(
        &mut self,
        id: Option<LocalForkId>,
        block_number: u64,
        env: &mut Env,
        journaled_state: &mut JournaledState,
    ) -> eyre::Result<()> {
        trace!(?id, ?block_number, "roll fork");
        let id = self.ensure_fork(id)?;
        let (fork_id, backend, fork_env) =
            self.forks.roll_fork(self.inner.ensure_fork_id(id).cloned()?, block_number)?;
        // this will update the local mapping
        self.inner.roll_fork(id, fork_id, backend)?;

        if let Some((active_id, active_idx)) = self.active_fork_ids {
            // the currently active fork is the targeted fork of this call
            if active_id == id {
                // need to update the block's env settings right away, which is otherwise set when
                // forks are selected `select_fork`
                update_current_env_with_fork_env(env, fork_env);

                // we also need to update the journaled_state right away, this has essentially the
                // same effect as selecting (`select_fork`) by discarding
                // non-persistent storage from the journaled_state. This which will
                // reset cached state from the previous block
                let mut persistent_addrs = self.inner.persistent_accounts.clone();
                // we also want to copy the caller state here
                persistent_addrs.extend(self.caller_address());

                let active = self.inner.get_fork_mut(active_idx);
                active.journaled_state = self.fork_init_journaled_state.clone();

                active.journaled_state.depth = journaled_state.depth;
                for addr in persistent_addrs {
                    merge_journaled_state_data(addr, journaled_state, &mut active.journaled_state);
                }

                // ensure all previously loaded accounts are present in the journaled state to
                // prevent issues in the new journalstate, e.g. assumptions that accounts are loaded
                // if the account is not touched, we reload it, if it's touched we clone it
                for (addr, acc) in journaled_state.state.iter() {
                    if acc.is_touched() {
                        merge_journaled_state_data(
                            *addr,
                            journaled_state,
                            &mut active.journaled_state,
                        );
                    } else {
                        let _ = active.journaled_state.load_account(*addr, &mut active.db);
                    }
                }

                *journaled_state = active.journaled_state.clone();
            }
        }
        Ok(())
    }

    fn roll_fork_to_transaction(
        &mut self,
        id: Option<LocalForkId>,
        transaction: B256,
        env: &mut Env,
        journaled_state: &mut JournaledState,
    ) -> eyre::Result<()> {
        trace!(?id, ?transaction, "roll fork to transaction");
        let id = self.ensure_fork(id)?;

        let (fork_block, block) =
            self.get_block_number_and_block_for_transaction(id, transaction)?;

        // roll the fork to the transaction's block or latest if it's pending
        self.roll_fork(Some(id), fork_block, env, journaled_state)?;

        update_env_block(env, fork_block, &block);

        // replay all transactions that came before
        let env = env.clone();

        self.replay_until(id, env, transaction, journaled_state)?;

        Ok(())
    }

    fn transact<I: InspectorExt<Backend>>(
        &mut self,
        maybe_id: Option<LocalForkId>,
        transaction: B256,
        env: &mut Env,
        journaled_state: &mut JournaledState,
        inspector: &mut I,
    ) -> eyre::Result<()> {
        trace!(?maybe_id, ?transaction, "execute transaction");
        let id = self.ensure_fork(maybe_id)?;
        let fork_id = self.ensure_fork_id(id).cloned()?;

        let tx = {
            let fork = self.inner.get_fork_by_id_mut(id)?;
            fork.db.db.get_transaction(transaction)?
        };

        // This is a bit ambiguous because the user wants to transact an arbitrary transaction in the current context, but we're assuming the user wants to transact the transaction as it was mined. Usually this is used in a combination of a fork at the transaction's parent transaction in the block and then the transaction is transacted: <https://github.com/foundry-rs/foundry/issues/6538>
        // So we modify the env to match the transaction's block
        let (fork_block, block) =
            self.get_block_number_and_block_for_transaction(id, transaction)?;
        let mut env = env.clone();
        update_env_block(&mut env, fork_block, &block);

        let env = self.env_with_handler_cfg(env);
        let fork = self.inner.get_fork_by_id_mut(id)?;
        commit_transaction(tx, env, journaled_state, fork, &fork_id, inspector)
    }

    fn active_fork_id(&self) -> Option<LocalForkId> {
        self.active_fork_ids.map(|(id, _)| id)
    }

    fn active_fork_url(&self) -> Option<String> {
        let fork = self.inner.issued_local_fork_ids.get(&self.active_fork_id()?)?;
        self.forks.get_fork_url(fork.clone()).ok()?
    }

    fn ensure_fork(&self, id: Option<LocalForkId>) -> eyre::Result<LocalForkId> {
        if let Some(id) = id {
            if self.inner.issued_local_fork_ids.contains_key(&id) {
                return Ok(id);
            }
            eyre::bail!("Requested fork `{}` does not exit", id)
        }
        if let Some(id) = self.active_fork_id() {
            Ok(id)
        } else {
            eyre::bail!("No fork active")
        }
    }

    fn ensure_fork_id(&self, id: LocalForkId) -> eyre::Result<&ForkId> {
        self.inner.ensure_fork_id(id)
    }

    fn diagnose_revert(
        &self,
        callee: Address,
        journaled_state: &JournaledState,
    ) -> Option<RevertDiagnostic> {
        let active_id = self.active_fork_id()?;
        let active_fork = self.active_fork()?;

        if self.inner.forks.len() == 1 {
            // we only want to provide additional diagnostics here when in multifork mode with > 1
            // forks
            return None;
        }

        if !active_fork.is_contract(callee) && !is_contract_in_state(journaled_state, callee) {
            // no contract for `callee` available on current fork, check if available on other forks
            let mut available_on = Vec::new();
            for (id, fork) in self.inner.forks_iter().filter(|(id, _)| *id != active_id) {
                trace!(?id, address=?callee, "checking if account exists");
                if fork.is_contract(callee) {
                    available_on.push(id);
                }
            }

            return if available_on.is_empty() {
                Some(RevertDiagnostic::ContractDoesNotExist {
                    contract: callee,
                    active: active_id,
                    persistent: self.is_persistent(&callee),
                })
            } else {
                // likely user error: called a contract that's not available on active fork but is
                // present other forks
                Some(RevertDiagnostic::ContractExistsOnOtherForks {
                    contract: callee,
                    active: active_id,
                    available_on,
                })
            };
        }
        None
    }

    /// Loads the account allocs from the given `allocs` map into the passed [JournaledState].
    ///
    /// Returns [Ok] if all accounts were successfully inserted into the journal, [Err] otherwise.
    fn load_allocs(
        &mut self,
        allocs: &BTreeMap<Address, GenesisAccount>,
        journaled_state: &mut JournaledState,
    ) -> Result<(), DatabaseError> {
        // Loop through all of the allocs defined in the map and commit them to the journal.
        for (addr, acc) in allocs.iter() {
            // Fetch the account from the journaled state. Will create a new account if it does
            // not already exist.
            let (state_acc, _) = journaled_state.load_account(*addr, self)?;

            // Set the account's bytecode and code hash, if the `bytecode` field is present.
            if let Some(bytecode) = acc.code.as_ref() {
                state_acc.info.code_hash = keccak256(bytecode);
                let bytecode = Bytecode::new_raw(bytecode.0.clone().into());
                state_acc.info.code = Some(bytecode);
            }

            // Set the account's storage, if the `storage` field is present.
            if let Some(storage) = acc.storage.as_ref() {
                state_acc.storage = storage
                    .iter()
                    .map(|(slot, value)| {
                        let slot = U256::from_be_bytes(slot.0);
                        (
                            slot,
                            StorageSlot::new_changed(
                                state_acc
                                    .storage
                                    .get(&slot)
                                    .map(|s| s.present_value)
                                    .unwrap_or_default(),
                                U256::from_be_bytes(value.0),
                            ),
                        )
                    })
                    .collect();
            }
            // Set the account's nonce and balance.
            state_acc.info.nonce = acc.nonce.unwrap_or_default();
            state_acc.info.balance = acc.balance;

            // Touch the account to ensure the loaded information persists if called in `setUp`.
            journaled_state.touch(addr);
        }

        Ok(())
    }

    fn add_persistent_account(&mut self, account: Address) -> bool {
        trace!(?account, "add persistent account");
        self.inner.persistent_accounts.insert(account)
    }

    fn remove_persistent_account(&mut self, account: &Address) -> bool {
        trace!(?account, "remove persistent account");
        self.inner.persistent_accounts.remove(account)
    }

    fn is_persistent(&self, acc: &Address) -> bool {
        self.inner.persistent_accounts.contains(acc)
    }

    fn allow_cheatcode_access(&mut self, account: Address) -> bool {
        trace!(?account, "allow cheatcode access");
        self.inner.cheatcode_access_accounts.insert(account)
    }

    fn revoke_cheatcode_access(&mut self, account: &Address) -> bool {
        trace!(?account, "revoke cheatcode access");
        self.inner.cheatcode_access_accounts.remove(account)
    }

    fn has_cheatcode_access(&self, account: &Address) -> bool {
        self.inner.cheatcode_access_accounts.contains(account)
    }

    fn get_block_number(&self, env: &Env) -> Result<U256, DatabaseError> {
        if let Some(active_fork_db) = self.active_fork_db() {
            if arbitrum::is_arbitrum(env.cfg.chain_id) {
                return arbitrum::get_block_number(active_fork_db, env)
                    .map_err(|e| DatabaseError::Other(e.to_string()));
            }
        }
        Ok(env.block.number)
    }
}

impl DatabaseRef for Backend {
    type Error = DatabaseError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        if let Some(db) = self.active_fork_db() {
            db.basic_ref(address)
        } else {
            Ok(self.mem_db.basic_ref(address)?)
        }
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        if let Some(db) = self.active_fork_db() {
            db.code_by_hash_ref(code_hash)
        } else {
            Ok(self.mem_db.code_by_hash_ref(code_hash)?)
        }
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        if let Some(db) = self.active_fork_db() {
            DatabaseRef::storage_ref(db, address, index)
        } else {
            Ok(DatabaseRef::storage_ref(&self.mem_db, address, index)?)
        }
    }

    fn block_hash_ref(&self, number: U256) -> Result<B256, Self::Error> {
        if let Some(db) = self.active_fork_db() {
            db.block_hash_ref(number)
        } else {
            Ok(self.mem_db.block_hash_ref(number)?)
        }
    }
}

impl DatabaseCommit for Backend {
    fn commit(&mut self, changes: Map<Address, Account>) {
        if let Some(db) = self.active_fork_db_mut() {
            db.commit(changes)
        } else {
            self.mem_db.commit(changes)
        }
    }
}

impl Database for Backend {
    type Error = DatabaseError;
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        if let Some(db) = self.active_fork_db_mut() {
            db.basic(address)
        } else {
            Ok(self.mem_db.basic(address)?)
        }
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        if let Some(db) = self.active_fork_db_mut() {
            db.code_by_hash(code_hash)
        } else {
            Ok(self.mem_db.code_by_hash(code_hash)?)
        }
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        if let Some(db) = self.active_fork_db_mut() {
            Database::storage(db, address, index)
        } else {
            Ok(Database::storage(&mut self.mem_db, address, index)?)
        }
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        if let Some(db) = self.active_fork_db_mut() {
            db.block_hash(number)
        } else {
            Ok(self.mem_db.block_hash(number)?)
        }
    }
}

/// Variants of a [revm::Database]
#[derive(Clone, Debug)]
pub enum BackendDatabaseSnapshot {
    /// Simple in-memory [revm::Database]
    InMemory(FoundryEvmInMemoryDB),
    /// Contains the entire forking mode database
    Forked(LocalForkId, ForkId, ForkLookupIndex, Box<Fork>),
}

#[derive(Clone, Debug)]
pub struct ForkSpecificPrecompiles {
    arb_sys: Option<Arc<arbitrum::ArbSysPrecompile>>,
}

impl ForkSpecificPrecompiles {
    pub fn new(chain_id: u64) -> Self {
        let is_arbitrum = arbitrum::is_arbitrum(chain_id);

        let mut arb_sys = None;
        if is_arbitrum {
            arb_sys = Some(Arc::new(arbitrum::ArbSysPrecompile {}));
        }

        Self { arb_sys }
    }

    pub fn addresses(&self) -> Vec<Address> {
        let mut addresses = Vec::new();

        if self.arb_sys.is_some() {
            addresses.push(ARB_SYS_ADDRESS);
        }

        addresses
    }

    pub fn fill_precompiles<DB: DatabaseExt>(&self, precompiles: &mut ContextPrecompiles<DB>) {
        if let Some(arb_sys) = &self.arb_sys {
            precompiles
                .insert(ARB_SYS_ADDRESS, ContextPrecompile::ContextStateful(arb_sys.clone()));
        }
    }
}

/// Represents a fork
#[derive(Clone, Debug)]
pub struct Fork {
    db: ForkDB,
    journaled_state: JournaledState,
    pub precompiles: ForkSpecificPrecompiles,
}

// === impl Fork ===

impl Fork {
    /// Returns true if the account is a contract
    pub fn is_contract(&self, acc: Address) -> bool {
        if let Ok(Some(acc)) = self.db.basic_ref(acc) {
            if acc.code_hash != KECCAK_EMPTY {
                return true;
            }
        }
        is_contract_in_state(&self.journaled_state, acc)
    }
}

/// Container type for various Backend related data
#[derive(Clone, Debug)]
pub struct BackendInner {
    /// Stores the `ForkId` of the fork the `Backend` launched with from the start.
    ///
    /// In other words if [`Backend::spawn()`] was called with a `CreateFork` command, to launch
    /// directly in fork mode, this holds the corresponding fork identifier of this fork.
    pub launched_with_fork: Option<(ForkId, LocalForkId, ForkLookupIndex)>,
    /// This tracks numeric fork ids and the `ForkId` used by the handler.
    ///
    /// This is necessary, because there can be multiple `Backends` associated with a single
    /// `ForkId` which is only a pair of endpoint + block. Since an existing fork can be
    /// modified (e.g. `roll_fork`), but this should only affect the fork that's unique for the
    /// test and not the `ForkId`
    ///
    /// This ensures we can treat forks as unique from the context of a test, so rolling to another
    /// is basically creating(or reusing) another `ForkId` that's then mapped to the previous
    /// issued _local_ numeric identifier, that remains constant, even if the underlying fork
    /// backend changes.
    pub issued_local_fork_ids: HashMap<LocalForkId, ForkId>,
    /// tracks all the created forks
    /// Contains the index of the corresponding `ForkDB` in the `forks` vec
    pub created_forks: HashMap<ForkId, ForkLookupIndex>,
    /// Holds all created fork databases
    // Note: data is stored in an `Option` so we can remove it without reshuffling
    pub forks: Vec<Option<Fork>>,
    /// Contains snapshots made at a certain point
    pub snapshots: Snapshots<BackendSnapshot<BackendDatabaseSnapshot>>,
    /// Tracks whether there was a failure in a snapshot that was reverted
    ///
    /// The Test contract contains a bool variable that is set to true when an `assert` function
    /// failed. When a snapshot is reverted, it reverts the state of the evm, but we still want
    /// to know if there was an `assert` that failed after the snapshot was taken so that we can
    /// check if the test function passed all asserts even across snapshots. When a snapshot is
    /// reverted we get the _current_ `revm::JournaledState` which contains the state that we can
    /// check if the `_failed` variable is set,
    /// additionally
    pub has_snapshot_failure: bool,
    /// Tracks the address of a Test contract
    ///
    /// This address can be used to inspect the state of the contract when a test is being
    /// executed. E.g. the `_failed` variable of `DSTest`
    pub test_contract_address: Option<Address>,
    /// Tracks the caller of the test function
    pub caller: Option<Address>,
    /// Tracks numeric identifiers for forks
    pub next_fork_id: LocalForkId,
    /// All accounts that should be kept persistent when switching forks.
    /// This means all accounts stored here _don't_ use a separate storage section on each fork
    /// instead the use only one that's persistent across fork swaps.
    ///
    /// See also [`clone_data()`]
    pub persistent_accounts: HashSet<Address>,
    /// The configured spec id
    pub spec_id: SpecId,
    /// All accounts that are allowed to execute cheatcodes
    pub cheatcode_access_accounts: HashSet<Address>,
}

// === impl BackendInner ===

impl BackendInner {
    pub fn ensure_fork_id(&self, id: LocalForkId) -> eyre::Result<&ForkId> {
        self.issued_local_fork_ids
            .get(&id)
            .ok_or_else(|| eyre::eyre!("No matching fork found for {}", id))
    }

    pub fn ensure_fork_index(&self, id: &ForkId) -> eyre::Result<ForkLookupIndex> {
        self.created_forks
            .get(id)
            .copied()
            .ok_or_else(|| eyre::eyre!("No matching fork found for {}", id))
    }

    pub fn ensure_fork_index_by_local_id(&self, id: LocalForkId) -> eyre::Result<ForkLookupIndex> {
        self.ensure_fork_index(self.ensure_fork_id(id)?)
    }

    /// Returns the underlying fork mapped to the index
    #[track_caller]
    fn get_fork(&self, idx: ForkLookupIndex) -> &Fork {
        debug_assert!(idx < self.forks.len(), "fork lookup index must exist");
        self.forks[idx].as_ref().unwrap()
    }

    /// Returns the underlying fork mapped to the index
    #[track_caller]
    fn get_fork_mut(&mut self, idx: ForkLookupIndex) -> &mut Fork {
        debug_assert!(idx < self.forks.len(), "fork lookup index must exist");
        self.forks[idx].as_mut().unwrap()
    }

    /// Returns the underlying fork corresponding to the id
    #[track_caller]
    fn get_fork_by_id_mut(&mut self, id: LocalForkId) -> eyre::Result<&mut Fork> {
        let idx = self.ensure_fork_index_by_local_id(id)?;
        Ok(self.get_fork_mut(idx))
    }

    /// Returns the underlying fork corresponding to the id
    #[track_caller]
    fn get_fork_by_id(&self, id: LocalForkId) -> eyre::Result<&Fork> {
        let idx = self.ensure_fork_index_by_local_id(id)?;
        Ok(self.get_fork(idx))
    }

    /// Removes the fork
    fn take_fork(&mut self, idx: ForkLookupIndex) -> Fork {
        debug_assert!(idx < self.forks.len(), "fork lookup index must exist");
        self.forks[idx].take().unwrap()
    }

    fn set_fork(&mut self, idx: ForkLookupIndex, fork: Fork) {
        self.forks[idx] = Some(fork)
    }

    /// Returns an iterator over Forks
    pub fn forks_iter(&self) -> impl Iterator<Item = (LocalForkId, &Fork)> + '_ {
        self.issued_local_fork_ids
            .iter()
            .map(|(id, fork_id)| (*id, self.get_fork(self.created_forks[fork_id])))
    }

    /// Returns a mutable iterator over all Forks
    pub fn forks_iter_mut(&mut self) -> impl Iterator<Item = &mut Fork> + '_ {
        self.forks.iter_mut().filter_map(|f| f.as_mut())
    }

    /// Reverts the entire fork database
    pub fn revert_snapshot(
        &mut self,
        id: LocalForkId,
        fork_id: ForkId,
        idx: ForkLookupIndex,
        fork: Fork,
    ) {
        self.created_forks.insert(fork_id.clone(), idx);
        self.issued_local_fork_ids.insert(id, fork_id);
        self.set_fork(idx, fork)
    }

    /// Updates the fork and the local mapping and returns the new index for the `fork_db`
    pub fn update_fork_mapping(
        &mut self,
        id: LocalForkId,
        fork_id: ForkId,
        db: ForkDB,
        journaled_state: JournaledState,
        precompiles: ForkSpecificPrecompiles,
    ) -> ForkLookupIndex {
        let idx = self.forks.len();
        self.issued_local_fork_ids.insert(id, fork_id.clone());
        self.created_forks.insert(fork_id, idx);

        let fork = Fork { db, journaled_state, precompiles };
        self.forks.push(Some(fork));
        idx
    }

    pub fn roll_fork(
        &mut self,
        id: LocalForkId,
        new_fork_id: ForkId,
        backend: SharedBackend,
    ) -> eyre::Result<ForkLookupIndex> {
        let fork_id = self.ensure_fork_id(id)?;
        let idx = self.ensure_fork_index(fork_id)?;

        if let Some(active) = self.forks[idx].as_mut() {
            // we initialize a _new_ `ForkDB` but keep the state of persistent accounts
            let mut new_db = ForkDB::new(backend);
            for addr in self.persistent_accounts.iter().copied() {
                merge_db_account_data(addr, &active.db, &mut new_db);
            }
            active.db = new_db;
        }
        // update mappings
        self.issued_local_fork_ids.insert(id, new_fork_id.clone());
        self.created_forks.insert(new_fork_id, idx);
        Ok(idx)
    }

    /// Inserts a _new_ `ForkDB` and issues a new local fork identifier
    ///
    /// Also returns the index where the `ForDB` is stored
    pub fn insert_new_fork(
        &mut self,
        fork_id: ForkId,
        db: ForkDB,
        journaled_state: JournaledState,
        precompiles: ForkSpecificPrecompiles,
    ) -> (LocalForkId, ForkLookupIndex) {
        let idx = self.forks.len();
        self.created_forks.insert(fork_id.clone(), idx);
        let id = self.next_id();
        self.issued_local_fork_ids.insert(id, fork_id);
        let fork = Fork { db, journaled_state, precompiles };
        self.forks.push(Some(fork));
        (id, idx)
    }

    fn next_id(&mut self) -> U256 {
        let id = self.next_fork_id;
        self.next_fork_id += U256::from(1);
        id
    }

    /// Returns the number of issued ids
    pub fn len(&self) -> usize {
        self.issued_local_fork_ids.len()
    }

    /// Returns true if no forks are issued
    pub fn is_empty(&self) -> bool {
        self.issued_local_fork_ids.is_empty()
    }

    pub fn precompiles(&self) -> &'static Precompiles {
        Precompiles::new(PrecompileSpecId::from_spec_id(self.spec_id))
    }

    /// Returns a new, empty, `JournaledState` with set precompiles
    pub fn new_journaled_state(&self) -> JournaledState {
        JournaledState::new(self.spec_id, self.precompiles().addresses().copied().collect())
    }
}

impl Default for BackendInner {
    fn default() -> Self {
        Self {
            launched_with_fork: None,
            issued_local_fork_ids: Default::default(),
            created_forks: Default::default(),
            forks: vec![],
            snapshots: Default::default(),
            has_snapshot_failure: false,
            test_contract_address: None,
            caller: None,
            next_fork_id: Default::default(),
            persistent_accounts: Default::default(),
            spec_id: SpecId::LATEST,
            // grant the cheatcode,default test and caller address access to execute cheatcodes
            // itself
            cheatcode_access_accounts: HashSet::from([
                CHEATCODE_ADDRESS,
                TEST_CONTRACT_ADDRESS,
                CALLER,
            ]),
        }
    }
}

/// This updates the currently used env with the fork's environment
pub(crate) fn update_current_env_with_fork_env(current: &mut Env, fork: Env) {
    current.block = fork.block;
    current.cfg = fork.cfg;
    current.tx.chain_id = fork.tx.chain_id;
}

/// Clones the data of the given `accounts` from the `active` database into the `fork_db`
/// This includes the data held in storage (`CacheDB`) and kept in the `JournaledState`.
pub(crate) fn merge_account_data<ExtDB: DatabaseRef>(
    accounts: impl IntoIterator<Item = Address>,
    active: &CacheDB<ExtDB>,
    active_journaled_state: &mut JournaledState,
    target_fork: &mut Fork,
) {
    for addr in accounts.into_iter() {
        merge_db_account_data(addr, active, &mut target_fork.db);
        merge_journaled_state_data(addr, active_journaled_state, &mut target_fork.journaled_state);
    }

    // need to mock empty journal entries in case the current checkpoint is higher than the existing
    // journal entries
    while active_journaled_state.journal.len() > target_fork.journaled_state.journal.len() {
        target_fork.journaled_state.journal.push(Default::default());
    }

    *active_journaled_state = target_fork.journaled_state.clone();
}

/// Clones the account data from the `active_journaled_state`  into the `fork_journaled_state`
fn merge_journaled_state_data(
    addr: Address,
    active_journaled_state: &JournaledState,
    fork_journaled_state: &mut JournaledState,
) {
    if let Some(mut acc) = active_journaled_state.state.get(&addr).cloned() {
        trace!(?addr, "updating journaled_state account data");
        if let Some(fork_account) = fork_journaled_state.state.get_mut(&addr) {
            // This will merge the fork's tracked storage with active storage and update values
            fork_account.storage.extend(std::mem::take(&mut acc.storage));
            // swap them so we can insert the account as whole in the next step
            std::mem::swap(&mut fork_account.storage, &mut acc.storage);
        }
        fork_journaled_state.state.insert(addr, acc);
    }
}

/// Clones the account data from the `active` db into the `ForkDB`
fn merge_db_account_data<ExtDB: DatabaseRef>(
    addr: Address,
    active: &CacheDB<ExtDB>,
    fork_db: &mut ForkDB,
) {
    trace!(?addr, "merging database data");

    let mut acc = if let Some(acc) = active.accounts.get(&addr).cloned() {
        acc
    } else {
        // Account does not exist
        return;
    };

    if let Some(code) = active.contracts.get(&acc.info.code_hash).cloned() {
        fork_db.contracts.insert(acc.info.code_hash, code);
    }

    if let Some(fork_account) = fork_db.accounts.get_mut(&addr) {
        // This will merge the fork's tracked storage with active storage and update values
        fork_account.storage.extend(std::mem::take(&mut acc.storage));
        // swap them so we can insert the account as whole in the next step
        std::mem::swap(&mut fork_account.storage, &mut acc.storage);
    }

    fork_db.accounts.insert(addr, acc);
}

/// Returns true of the address is a contract
fn is_contract_in_state(journaled_state: &JournaledState, acc: Address) -> bool {
    journaled_state
        .state
        .get(&acc)
        .map(|acc| acc.info.code_hash != KECCAK_EMPTY)
        .unwrap_or_default()
}

/// Updates the env's block with the block's data
fn update_env_block(env: &mut Env, fork_block: u64, block: &Block) {
    env.block.timestamp = U256::from(block.header.timestamp);
    env.block.coinbase = block.header.miner;
    env.block.difficulty = block.header.difficulty;
    env.block.prevrandao = Some(block.header.mix_hash.unwrap_or_default());
    env.block.basefee = U256::from(block.header.base_fee_per_gas.unwrap_or_default());
    env.block.gas_limit = U256::from(block.header.gas_limit);
    env.block.number = U256::from(block.header.number.unwrap_or(fork_block));
}

/// Executes the given transaction and commits state changes to the database _and_ the journaled
/// state, with an optional inspector
fn commit_transaction<I: InspectorExt<Backend>>(
    tx: WithOtherFields<Transaction>,
    mut env: EnvWithHandlerCfg,
    journaled_state: &mut JournaledState,
    fork: &mut Fork,
    fork_id: &ForkId,
    inspector: I,
) -> eyre::Result<()> {
    configure_tx_env(&mut env.env, &tx);

    let now = Instant::now();
    let res = {
        let fork = fork.clone();
        let journaled_state = journaled_state.clone();
        let db = Backend::new_with_fork(fork_id, fork, journaled_state);
        crate::utils::new_extended_evm_with_inspector(db, env, inspector)
            .transact()
            .wrap_err("backend: failed committing transaction")?
    };
    trace!(elapsed = ?now.elapsed(), "transacted transaction");

    apply_state_changeset(res.state, journaled_state, fork)?;
    Ok(())
}

/// Helper method which updates data in the state with the data from the database.
pub fn update_state<DB: Database>(state: &mut State, db: &mut DB) -> Result<(), DB::Error> {
    for (addr, acc) in state.iter_mut() {
        acc.info = db.basic(*addr)?.unwrap_or_default();
        for (key, val) in acc.storage.iter_mut() {
            val.present_value = db.storage(*addr, *key)?;
        }
    }

    Ok(())
}

/// Applies the changeset of a transaction to the active journaled state and also commits it in the
/// forked db
fn apply_state_changeset(
    state: Map<revm::primitives::Address, Account>,
    journaled_state: &mut JournaledState,
    fork: &mut Fork,
) -> Result<(), DatabaseError> {
    // commit the state and update the loaded accounts
    fork.db.commit(state);

    update_state(&mut journaled_state.state, &mut fork.db)?;
    update_state(&mut fork.journaled_state.state, &mut fork.db)?;

    Ok(())
}
