// We don't document function parameters individually so we can't enable `missing_docs` for this
// module. Instead, we emit custom diagnostics in `#[derive(Cheatcode)]`.
#![allow(missing_docs)]

use super::*;
use alloy_sol_types::sol;
use foundry_macros::Cheatcode;

sol! {
// Cheatcodes are marked as view/pure/none using the following rules:
// 0. A call's observable behaviour includes its return value, logs, reverts and state writes,
// 1. If you can influence a later call's observable behaviour, you're neither `view` nor `pure`
//    (you are modifying some state be it the EVM, interpreter, filesystem, etc),
// 2. Otherwise if you can be influenced by an earlier call, or if reading some state, you're `view`,
// 3. Otherwise you're `pure`.

/// Foundry cheatcodes interface.
#[derive(Debug, Cheatcode)] // Keep this list small to avoid unnecessary bloat.
interface Vm {
    //  ======== Types ========

    /// Error thrown by cheatcodes.
    error CheatcodeError(string message);

    /// A modification applied to either `msg.sender` or `tx.origin`. Returned by `readCallers`.
    enum CallerMode {
        /// No caller modification is currently active.
        None,
        /// A one time broadcast triggered by a `vm.broadcast()` call is currently active.
        Broadcast,
        /// A recurrent broadcast triggered by a `vm.startBroadcast()` call is currently active.
        RecurrentBroadcast,
        /// A one time prank triggered by a `vm.prank()` call is currently active.
        Prank,
        /// A recurrent prank triggered by a `vm.startPrank()` call is currently active.
        RecurrentPrank,
    }

    /// An Ethereum log. Returned by `getRecordedLogs`.
    struct Log {
        /// The topics of the log, including the signature, if any.
        bytes32[] topics;
        /// The raw data of the log.
        bytes data;
        /// The address of the log's emitter.
        address emitter;
    }

    /// An RPC URL and its alias. Returned by `rpcUrlStructs`.
    struct Rpc {
        /// The alias of the RPC URL.
        string key;
        /// The RPC URL.
        string url;
    }

    /// An RPC log object. Returned by `eth_getLogs`.
    struct EthGetLogs {
        /// The address of the log's emitter.
        address emitter;
        /// The topics of the log, including the signature, if any.
        bytes32[] topics;
        /// The raw data of the log.
        bytes data;
        /// The block hash.
        bytes32 blockHash;
        /// The block number.
        uint64 blockNumber;
        /// The transaction hash.
        bytes32 transactionHash;
        /// The transaction index in the block.
        uint64 transactionIndex;
        /// The log index.
        uint256 logIndex;
        /// Whether the log was removed.
        bool removed;
    }

    /// A single entry in a directory listing. Returned by `readDir`.
    struct DirEntry {
        /// The error message, if any.
        string errorMessage;
        /// The path of the entry.
        string path;
        /// The depth of the entry.
        uint64 depth;
        /// Whether the entry is a directory.
        bool isDir;
        /// Whether the entry is a symlink.
        bool isSymlink;
    }

    /// Metadata information about a file.
    ///
    /// This structure is returned from the [`fsMetadata`] function and represents known
    /// metadata about a file such as its permissions, size, modification
    /// times, etc.
    struct FsMetadata {
        /// True if this metadata is for a directory.
        bool isDir;
        /// True if this metadata is for a symlink.
        bool isSymlink;
        /// The size of the file, in bytes, this metadata is for.
        uint256 length;
        /// True if this metadata is for a readonly (unwritable) file.
        bool readOnly;
        /// The last modification time listed in this metadata.
        uint256 modified;
        /// The last access time of this metadata.
        uint256 accessed;
        /// The creation time listed in this metadata.
        uint256 created;
    }

    /// A wallet with a public and private key.
    struct Wallet {
        /// The wallet's address.
        address addr;
        /// The wallet's public key `X`.
        uint256 publicKeyX;
        /// The wallet's public key `Y`.
        uint256 publicKeyY;
        /// The wallet's private key.
        uint256 privateKey;
    }

    /// The result of a [`tryFfi`](tryFfiCall) call.
    struct FfiResult {
        /// The exit code of the call.
        int32 exitCode;
        /// The optionally hex-decoded `stdout` data.
        bytes stdout;
        /// The `stderr` data.
        bytes stderr;
    }

    // ======== EVM ========

    /// Gets the address for a given private key.
    #[cheatcode(group = Evm, safety = Safe)]
    function addr(uint256 privateKey) external pure returns (address keyAddr);

    /// Gets the nonce of an account.
    #[cheatcode(group = Evm, safety = Safe)]
    function getNonce(address account) external view returns (uint64 nonce);

    /// Loads a storage slot from an address.
    #[cheatcode(group = Evm, safety = Safe)]
    function load(address target, bytes32 slot) external view returns (bytes32 data);

    /// Load a genesis JSON file's `allocs` into the in-memory revm state.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function loadAllocs(string calldata pathToAllocsJson) external;

    /// Signs data.
    #[cheatcode(group = Evm, safety = Safe)]
    function sign(uint256 privateKey, bytes32 digest) external pure returns (uint8 v, bytes32 r, bytes32 s);

    // -------- Record Storage --------

    /// Records all storage reads and writes.
    #[cheatcode(group = Evm, safety = Safe)]
    function record() external;

    /// Gets all accessed reads and write slot from a `vm.record` session, for a given address.
    #[cheatcode(group = Evm, safety = Safe)]
    function accesses(address target) external returns (bytes32[] memory readSlots, bytes32[] memory writeSlots);

    // -------- Recording Map Writes --------

    /// Starts recording all map SSTOREs for later retrieval.
    #[cheatcode(group = Evm, safety = Safe)]
    function startMappingRecording() external;

    /// Stops recording all map SSTOREs for later retrieval and clears the recorded data.
    #[cheatcode(group = Evm, safety = Safe)]
    function stopMappingRecording() external;

    /// Gets the number of elements in the mapping at the given slot, for a given address.
    #[cheatcode(group = Evm, safety = Safe)]
    function getMappingLength(address target, bytes32 mappingSlot) external returns (uint256 length);

    /// Gets the elements at index idx of the mapping at the given slot, for a given address. The
    /// index must be less than the length of the mapping (i.e. the number of keys in the mapping).
    #[cheatcode(group = Evm, safety = Safe)]
    function getMappingSlotAt(address target, bytes32 mappingSlot, uint256 idx) external returns (bytes32 value);

    /// Gets the map key and parent of a mapping at a given slot, for a given address.
    #[cheatcode(group = Evm, safety = Safe)]
    function getMappingKeyAndParentOf(address target, bytes32 elementSlot)
        external
        returns (bool found, bytes32 key, bytes32 parent);

    // -------- Block and Transaction Properties --------

    /// Sets `block.chainid`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function chainId(uint256 newChainId) external;

    /// Sets `block.coinbase`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function coinbase(address newCoinbase) external;

    /// Sets `block.difficulty`.
    /// Not available on EVM versions from Paris onwards. Use `prevrandao` instead.
    /// Reverts if used on unsupported EVM versions.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function difficulty(uint256 newDifficulty) external;

    /// Sets `block.basefee`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function fee(uint256 newBasefee) external;

    /// Sets `block.prevrandao`.
    /// Not available on EVM versions before Paris. Use `difficulty` instead.
    /// If used on unsupported EVM versions it will revert.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function prevrandao(bytes32 newPrevrandao) external;

    /// Sets `block.height`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function roll(uint256 newHeight) external;

    /// Sets `tx.gasprice`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function txGasPrice(uint256 newGasPrice) external;

    /// Sets `block.timestamp`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function warp(uint256 newTimestamp) external;

    // -------- Account State --------

    /// Sets an address' balance.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function deal(address account, uint256 newBalance) external;

    /// Sets an address' code.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function etch(address target, bytes calldata newRuntimeBytecode) external;

    /// Resets the nonce of an account to 0 for EOAs and 1 for contract accounts.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function resetNonce(address account) external;

    /// Sets the nonce of an account. Must be higher than the current nonce of the account.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function setNonce(address account, uint64 newNonce) external;

    /// Sets the nonce of an account to an arbitrary value.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function setNonceUnsafe(address account, uint64 newNonce) external;

    /// Stores a value to an address' storage slot.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function store(address target, bytes32 slot, bytes32 value) external;

    /// Marks the `target` address cold, and is a no-op if the address is already cold.
    /// All storage slots are also made cold, but their values are preserved.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function cool(address target) external;

    // -------- Call Manipulation --------
    // --- Mocks ---

    /// Clears all mocked calls.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function clearMockedCalls() external;

    /// Mocks a call to an address, returning specified data.
    /// Calldata can either be strict or a partial match, e.g. if you only
    /// pass a Solidity selector to the expected calldata, then the entire Solidity
    /// function will be mocked.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockCall(address callee, bytes calldata data, bytes calldata returnData) external;

    /// Mocks a call to an address with a specific `msg.value`, returning specified data.
    /// Calldata match takes precedence over `msg.value` in case of ambiguity.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockCall(address callee, uint256 msgValue, bytes calldata data, bytes calldata returnData) external;

    /// Reverts a call to an address with specified revert data.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockCallRevert(address callee, bytes calldata data, bytes calldata revertData) external;

    /// Reverts a call to an address with a specific `msg.value`, with specified revert data.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockCallRevert(address callee, uint256 msgValue, bytes calldata data, bytes calldata revertData)
        external;

    // --- Impersonation (pranks) ---

    /// Sets the *next* call's `msg.sender` to be the input address.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function prank(address msgSender) external;

    /// Sets all subsequent calls' `msg.sender` to be the input address until `stopPrank` is called.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function startPrank(address msgSender) external;

    /// Sets the *next* call's `msg.sender` to be the input address, and the `tx.origin` to be the second input.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function prank(address msgSender, address txOrigin) external;

    /// Sets all subsequent calls' `msg.sender` to be the input address until `stopPrank` is called, and the `tx.origin` to be the second input.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function startPrank(address msgSender, address txOrigin) external;

    /// Resets subsequent calls' `msg.sender` to be `address(this)`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function stopPrank() external;

    /// Reads the current `msg.sender` and `tx.origin` from state and reports if there is any active caller modification.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function readCallers() external returns (CallerMode callerMode, address msgSender, address txOrigin);

    // -------- State Snapshots --------

    /// Snapshot the current state of the evm.
    /// Returns the ID of the snapshot that was created.
    /// To revert a snapshot use `revertTo`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function snapshot() external returns (uint256 snapshotId);

    /// Revert the state of the EVM to a previous snapshot
    /// Takes the snapshot ID to revert to.
    /// This deletes the snapshot and all snapshots taken after the given snapshot ID.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function revertTo(uint256 snapshotId) external returns (bool success);

    // -------- Forking --------
    // --- Creation and Selection ---

    /// Returns the identifier of the currently active fork. Reverts if no fork is currently active.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function activeFork() external view returns (uint256 forkId);

    /// Creates a new fork with the given endpoint and the _latest_ block and returns the identifier of the fork.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function createFork(string calldata urlOrAlias) external returns (uint256 forkId);
    /// Creates a new fork with the given endpoint and block and returns the identifier of the fork.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function createFork(string calldata urlOrAlias, uint256 blockNumber) external returns (uint256 forkId);
    /// Creates a new fork with the given endpoint and at the block the given transaction was mined in,
    /// replays all transaction mined in the block before the transaction, and returns the identifier of the fork.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function createFork(string calldata urlOrAlias, bytes32 txHash) external returns (uint256 forkId);

    /// Creates and also selects a new fork with the given endpoint and the latest block and returns the identifier of the fork.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function createSelectFork(string calldata urlOrAlias) external returns (uint256 forkId);
    /// Creates and also selects a new fork with the given endpoint and block and returns the identifier of the fork.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function createSelectFork(string calldata urlOrAlias, uint256 blockNumber) external returns (uint256 forkId);
    /// Creates and also selects new fork with the given endpoint and at the block the given transaction was mined in,
    /// replays all transaction mined in the block before the transaction, returns the identifier of the fork.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function createSelectFork(string calldata urlOrAlias, bytes32 txHash) external returns (uint256 forkId);

    /// Updates the currently active fork to given block number
    /// This is similar to `roll` but for the currently active fork.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function rollFork(uint256 blockNumber) external;
    /// Updates the currently active fork to given transaction. This will `rollFork` with the number
    /// of the block the transaction was mined in and replays all transaction mined before it in the block.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function rollFork(bytes32 txHash) external;
    /// Updates the given fork to given block number.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function rollFork(uint256 forkId, uint256 blockNumber) external;
    /// Updates the given fork to block number of the given transaction and replays all transaction mined before it in the block.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function rollFork(uint256 forkId, bytes32 txHash) external;

    /// Takes a fork identifier created by `createFork` and sets the corresponding forked state as active.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function selectFork(uint256 forkId) external;

    /// Fetches the given transaction from the active fork and executes it on the current state.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function transact(bytes32 txHash) external;
    /// Fetches the given transaction from the given fork and executes it on the current state.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function transact(uint256 forkId, bytes32 txHash) external;

    /// Performs an Ethereum JSON-RPC request to the current fork URL.
    #[cheatcode(group = Evm, safety = Safe)]
    function rpc(string calldata method, string calldata params) external returns (bytes memory data);

    /// Gets all the logs according to specified filter.
    #[cheatcode(group = Evm, safety = Safe)]
    function eth_getLogs(uint256 fromBlock, uint256 toBlock, address addr, bytes32[] memory topics)
        external
        returns (EthGetLogs[] memory logs);

    // --- Behavior ---

    /// In forking mode, explicitly grant the given address cheatcode access.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function allowCheatcodes(address account) external;

    /// Marks that the account(s) should use persistent storage across fork swaps in a multifork setup
    /// Meaning, changes made to the state of this account will be kept when switching forks.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function makePersistent(address account) external;
    /// See `makePersistent(address)`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function makePersistent(address account0, address account1) external;
    /// See `makePersistent(address)`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function makePersistent(address account0, address account1, address account2) external;
    /// See `makePersistent(address)`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function makePersistent(address[] calldata accounts) external;

    /// Revokes persistent status from the address, previously added via `makePersistent`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function revokePersistent(address account) external;
    /// See `revokePersistent(address)`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function revokePersistent(address[] calldata accounts) external;

    /// Returns true if the account is marked as persistent.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function isPersistent(address account) external view returns (bool persistent);

    // -------- Record Logs --------

    /// Record all the transaction logs.
    #[cheatcode(group = Evm, safety = Safe)]
    function recordLogs() external;

    /// Gets all the recorded logs.
    #[cheatcode(group = Evm, safety = Safe)]
    function getRecordedLogs() external returns (Log[] memory logs);

    // -------- Gas Metering --------
    // It's recommend to use the `noGasMetering` modifier included with forge-std, instead of
    // using these functions directly.

    /// Pauses gas metering (i.e. gas usage is not counted). Noop if already paused.
    #[cheatcode(group = Evm, safety = Safe)]
    function pauseGasMetering() external;

    /// Resumes gas metering (i.e. gas usage is counted again). Noop if already on.
    #[cheatcode(group = Evm, safety = Safe)]
    function resumeGasMetering() external;

    // ======== Test Assertions and Utilities ========

    /// If the condition is false, discard this run's fuzz inputs and generate new ones.
    #[cheatcode(group = Testing, safety = Safe)]
    function assume(bool condition) external pure;

    /// Writes a breakpoint to jump to in the debugger.
    #[cheatcode(group = Testing, safety = Safe)]
    function breakpoint(string calldata char) external;

    /// Writes a conditional breakpoint to jump to in the debugger.
    #[cheatcode(group = Testing, safety = Safe)]
    function breakpoint(string calldata char, bool value) external;

    /// Returns the RPC url for the given alias.
    #[cheatcode(group = Testing, safety = Safe)]
    function rpcUrl(string calldata rpcAlias) external view returns (string memory json);

    /// Returns all rpc urls and their aliases `[alias, url][]`.
    #[cheatcode(group = Testing, safety = Safe)]
    function rpcUrls() external view returns (string[2][] memory urls);

    /// Returns all rpc urls and their aliases as structs.
    #[cheatcode(group = Testing, safety = Safe)]
    function rpcUrlStructs() external view returns (Rpc[] memory urls);

    /// Suspends execution of the main thread for `duration` milliseconds.
    #[cheatcode(group = Testing, safety = Safe)]
    function sleep(uint256 duration) external;

    /// Expects a call to an address with the specified calldata.
    /// Calldata can either be a strict or a partial match.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectCall(address callee, bytes calldata data) external;

    /// Expects given number of calls to an address with the specified calldata.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectCall(address callee, bytes calldata data, uint64 count) external;

    /// Expects a call to an address with the specified `msg.value` and calldata.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectCall(address callee, uint256 msgValue, bytes calldata data) external;

    /// Expects given number of calls to an address with the specified `msg.value` and calldata.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectCall(address callee, uint256 msgValue, bytes calldata data, uint64 count) external;

    /// Expect a call to an address with the specified `msg.value`, gas, and calldata.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectCall(address callee, uint256 msgValue, uint64 gas, bytes calldata data) external;

    /// Expects given number of calls to an address with the specified `msg.value`, gas, and calldata.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectCall(address callee, uint256 msgValue, uint64 gas, bytes calldata data, uint64 count) external;

    /// Expect a call to an address with the specified `msg.value` and calldata, and a *minimum* amount of gas.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectCallMinGas(address callee, uint256 msgValue, uint64 minGas, bytes calldata data) external;

    /// Expect given number of calls to an address with the specified `msg.value` and calldata, and a *minimum* amount of gas.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectCallMinGas(address callee, uint256 msgValue, uint64 minGas, bytes calldata data, uint64 count)
        external;

    /// Prepare an expected log with (bool checkTopic1, bool checkTopic2, bool checkTopic3, bool checkData.).
    /// Call this function, then emit an event, then call a function. Internally after the call, we check if
    /// logs were emitted in the expected order with the expected topics and data (as specified by the booleans).
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectEmit(bool checkTopic1, bool checkTopic2, bool checkTopic3, bool checkData) external;

    /// Same as the previous method, but also checks supplied address against emitting contract.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectEmit(bool checkTopic1, bool checkTopic2, bool checkTopic3, bool checkData, address emitter)
        external;

    /// Prepare an expected log with all topic and data checks enabled.
    /// Call this function, then emit an event, then call a function. Internally after the call, we check if
    /// logs were emitted in the expected order with the expected topics and data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectEmit() external;

    /// Same as the previous method, but also checks supplied address against emitting contract.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectEmit(address emitter) external;

    /// Expects an error on next call with any revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert() external;

    /// Expects an error on next call that starts with the revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(bytes4 revertData) external;

    /// Expects an error on next call that exactly matches the revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(bytes calldata revertData) external;

    /// Only allows memory writes to offsets [0x00, 0x60) ∪ [min, max) in the current subcontext. If any other
    /// memory is written to, the test will fail. Can be called multiple times to add more ranges to the set.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectSafeMemory(uint64 min, uint64 max) external;

    /// Only allows memory writes to offsets [0x00, 0x60) ∪ [min, max) in the next created subcontext.
    /// If any other memory is written to, the test will fail. Can be called multiple times to add more ranges
    /// to the set.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectSafeMemoryCall(uint64 min, uint64 max) external;

    /// Marks a test as skipped. Must be called at the top of the test.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function skip(bool skipTest) external;

    // ======== OS and Filesystem ========

    // -------- Metadata --------

    /// Returns true if the given path points to an existing entity, else returns false.
    #[cheatcode(group = Filesystem)]
    function exists(string calldata path) external returns (bool result);

    /// Given a path, query the file system to get information about a file, directory, etc.
    #[cheatcode(group = Filesystem)]
    function fsMetadata(string calldata path) external view returns (FsMetadata memory metadata);

    /// Returns true if the path exists on disk and is pointing at a directory, else returns false.
    #[cheatcode(group = Filesystem)]
    function isDir(string calldata path) external returns (bool result);

    /// Returns true if the path exists on disk and is pointing at a regular file, else returns false.
    #[cheatcode(group = Filesystem)]
    function isFile(string calldata path) external returns (bool result);

    /// Get the path of the current project root.
    #[cheatcode(group = Filesystem)]
    function projectRoot() external view returns (string memory path);

    /// Returns the time since unix epoch in milliseconds.
    #[cheatcode(group = Filesystem)]
    function unixTime() external returns (uint256 milliseconds);

    // -------- Reading and writing --------

    /// Closes file for reading, resetting the offset and allowing to read it from beginning with readLine.
    /// `path` is relative to the project root.
    #[cheatcode(group = Filesystem)]
    function closeFile(string calldata path) external;

    /// Copies the contents of one file to another. This function will **overwrite** the contents of `to`.
    /// On success, the total number of bytes copied is returned and it is equal to the length of the `to` file as reported by `metadata`.
    /// Both `from` and `to` are relative to the project root.
    #[cheatcode(group = Filesystem)]
    function copyFile(string calldata from, string calldata to) external returns (uint64 copied);

    /// Creates a new, empty directory at the provided path.
    /// This cheatcode will revert in the following situations, but is not limited to just these cases:
    /// - User lacks permissions to modify `path`.
    /// - A parent of the given path doesn't exist and `recursive` is false.
    /// - `path` already exists and `recursive` is false.
    /// `path` is relative to the project root.
    #[cheatcode(group = Filesystem)]
    function createDir(string calldata path, bool recursive) external;

    /// Reads the directory at the given path recursively, up to `maxDepth`.
    /// `maxDepth` defaults to 1, meaning only the direct children of the given directory will be returned.
    /// Follows symbolic links if `followLinks` is true.
    #[cheatcode(group = Filesystem)]
    function readDir(string calldata path) external view returns (DirEntry[] memory entries);
    /// See `readDir(string)`.
    #[cheatcode(group = Filesystem)]
    function readDir(string calldata path, uint64 maxDepth) external view returns (DirEntry[] memory entries);
    /// See `readDir(string)`.
    #[cheatcode(group = Filesystem)]
    function readDir(string calldata path, uint64 maxDepth, bool followLinks)
        external
        view
        returns (DirEntry[] memory entries);

    /// Reads the entire content of file to string. `path` is relative to the project root.
    #[cheatcode(group = Filesystem)]
    function readFile(string calldata path) external view returns (string memory data);

    /// Reads the entire content of file as binary. `path` is relative to the project root.
    #[cheatcode(group = Filesystem)]
    function readFileBinary(string calldata path) external view returns (bytes memory data);

    /// Reads next line of file to string.
    #[cheatcode(group = Filesystem)]
    function readLine(string calldata path) external view returns (string memory line);

    /// Reads a symbolic link, returning the path that the link points to.
    /// This cheatcode will revert in the following situations, but is not limited to just these cases:
    /// - `path` is not a symbolic link.
    /// - `path` does not exist.
    #[cheatcode(group = Filesystem)]
    function readLink(string calldata linkPath) external view returns (string memory targetPath);

    /// Removes a directory at the provided path.
    /// This cheatcode will revert in the following situations, but is not limited to just these cases:
    /// - `path` doesn't exist.
    /// - `path` isn't a directory.
    /// - User lacks permissions to modify `path`.
    /// - The directory is not empty and `recursive` is false.
    /// `path` is relative to the project root.
    #[cheatcode(group = Filesystem)]
    function removeDir(string calldata path, bool recursive) external;

    /// Removes a file from the filesystem.
    /// This cheatcode will revert in the following situations, but is not limited to just these cases:
    /// - `path` points to a directory.
    /// - The file doesn't exist.
    /// - The user lacks permissions to remove the file.
    /// `path` is relative to the project root.
    #[cheatcode(group = Filesystem)]
    function removeFile(string calldata path) external;

    /// Writes data to file, creating a file if it does not exist, and entirely replacing its contents if it does.
    /// `path` is relative to the project root.
    #[cheatcode(group = Filesystem)]
    function writeFile(string calldata path, string calldata data) external;

    /// Writes binary data to a file, creating a file if it does not exist, and entirely replacing its contents if it does.
    /// `path` is relative to the project root.
    #[cheatcode(group = Filesystem)]
    function writeFileBinary(string calldata path, bytes calldata data) external;

    /// Writes line to file, creating a file if it does not exist.
    /// `path` is relative to the project root.
    #[cheatcode(group = Filesystem)]
    function writeLine(string calldata path, string calldata data) external;

    /// Gets the creation bytecode from an artifact file. Takes in the relative path to the json file.
    #[cheatcode(group = Filesystem)]
    function getCode(string calldata artifactPath) external view returns (bytes memory creationBytecode);

    /// Gets the deployed bytecode from an artifact file. Takes in the relative path to the json file.
    #[cheatcode(group = Filesystem)]
    function getDeployedCode(string calldata artifactPath) external view returns (bytes memory runtimeBytecode);

    // -------- Foreign Function Interface --------

    /// Performs a foreign function call via the terminal.
    #[cheatcode(group = Filesystem)]
    function ffi(string[] calldata commandInput) external returns (bytes memory result);

    /// Performs a foreign function call via terminal and returns the exit code, stdout, and stderr.
    #[cheatcode(group = Filesystem)]
    function tryFfi(string[] calldata commandInput) external returns (FfiResult memory result);

    // ======== Environment Variables ========

    /// Sets environment variables.
    #[cheatcode(group = Environment)]
    function setEnv(string calldata name, string calldata value) external;

    /// Gets the environment variable `name` and parses it as `bool`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envBool(string calldata name) external view returns (bool value);
    /// Gets the environment variable `name` and parses it as `uint256`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envUint(string calldata name) external view returns (uint256 value);
    /// Gets the environment variable `name` and parses it as `int256`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envInt(string calldata name) external view returns (int256 value);
    /// Gets the environment variable `name` and parses it as `address`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envAddress(string calldata name) external view returns (address value);
    /// Gets the environment variable `name` and parses it as `bytes32`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envBytes32(string calldata name) external view returns (bytes32 value);
    /// Gets the environment variable `name` and parses it as `string`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envString(string calldata name) external view returns (string memory value);
    /// Gets the environment variable `name` and parses it as `bytes`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envBytes(string calldata name) external view returns (bytes memory value);

    /// Gets the environment variable `name` and parses it as an array of `bool`, delimited by `delim`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envBool(string calldata name, string calldata delim) external view returns (bool[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `uint256`, delimited by `delim`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envUint(string calldata name, string calldata delim) external view returns (uint256[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `int256`, delimited by `delim`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envInt(string calldata name, string calldata delim) external view returns (int256[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `address`, delimited by `delim`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envAddress(string calldata name, string calldata delim) external view returns (address[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `bytes32`, delimited by `delim`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envBytes32(string calldata name, string calldata delim) external view returns (bytes32[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `string`, delimited by `delim`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envString(string calldata name, string calldata delim) external view returns (string[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `bytes`, delimited by `delim`.
    /// Reverts if the variable was not found or could not be parsed.
    #[cheatcode(group = Environment)]
    function envBytes(string calldata name, string calldata delim) external view returns (bytes[] memory value);

    /// Gets the environment variable `name` and parses it as `bool`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, bool defaultValue) external returns (bool value);
    /// Gets the environment variable `name` and parses it as `uint256`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, uint256 defaultValue) external returns (uint256 value);
    /// Gets the environment variable `name` and parses it as `int256`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, int256 defaultValue) external returns (int256 value);
    /// Gets the environment variable `name` and parses it as `address`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, address defaultValue) external returns (address value);
    /// Gets the environment variable `name` and parses it as `bytes32`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, bytes32 defaultValue) external returns (bytes32 value);
    /// Gets the environment variable `name` and parses it as `string`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata defaultValue) external returns (string memory value);
    /// Gets the environment variable `name` and parses it as `bytes`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, bytes calldata defaultValue) external returns (bytes memory value);

    /// Gets the environment variable `name` and parses it as an array of `bool`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, bool[] calldata defaultValue)
        external
        returns (bool[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `uint256`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, uint256[] calldata defaultValue)
        external
        returns (uint256[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `int256`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, int256[] calldata defaultValue)
        external
        returns (int256[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `address`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, address[] calldata defaultValue)
        external
        returns (address[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `bytes32`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, bytes32[] calldata defaultValue)
        external
        returns (bytes32[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `string`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, string[] calldata defaultValue)
        external
        returns (string[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `bytes`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, bytes[] calldata defaultValue)
        external
        returns (bytes[] memory value);

    // ======== Scripts ========

    // -------- Broadcasting Transactions --------

    /// Using the address that calls the test contract, has the next call (at this call depth only)
    /// create a transaction that can later be signed and sent onchain.
    #[cheatcode(group = Scripting)]
    function broadcast() external;

    /// Has the next call (at this call depth only) create a transaction with the address provided
    /// as the sender that can later be signed and sent onchain.
    #[cheatcode(group = Scripting)]
    function broadcast(address signer) external;

    /// Has the next call (at this call depth only) create a transaction with the private key
    /// provided as the sender that can later be signed and sent onchain.
    #[cheatcode(group = Scripting)]
    function broadcast(uint256 privateKey) external;

    /// Using the address that calls the test contract, has all subsequent calls
    /// (at this call depth only) create transactions that can later be signed and sent onchain.
    #[cheatcode(group = Scripting)]
    function startBroadcast() external;

    /// Has all subsequent calls (at this call depth only) create transactions with the address
    /// provided that can later be signed and sent onchain.
    #[cheatcode(group = Scripting)]
    function startBroadcast(address signer) external;

    /// Has all subsequent calls (at this call depth only) create transactions with the private key
    /// provided that can later be signed and sent onchain.
    #[cheatcode(group = Scripting)]
    function startBroadcast(uint256 privateKey) external;

    /// Stops collecting onchain transactions.
    #[cheatcode(group = Scripting)]
    function stopBroadcast() external;

    // ======== Utilities ========

    // -------- Strings --------

    /// Converts the given value to a `string`.
    #[cheatcode(group = String)]
    function toString(address value) external pure returns (string memory stringifiedValue);
    /// Converts the given value to a `string`.
    #[cheatcode(group = String)]
    function toString(bytes calldata value) external pure returns (string memory stringifiedValue);
    /// Converts the given value to a `string`.
    #[cheatcode(group = String)]
    function toString(bytes32 value) external pure returns (string memory stringifiedValue);
    /// Converts the given value to a `string`.
    #[cheatcode(group = String)]
    function toString(bool value) external pure returns (string memory stringifiedValue);
    /// Converts the given value to a `string`.
    #[cheatcode(group = String)]
    function toString(uint256 value) external pure returns (string memory stringifiedValue);
    /// Converts the given value to a `string`.
    #[cheatcode(group = String)]
    function toString(int256 value) external pure returns (string memory stringifiedValue);

    /// Parses the given `string` into `bytes`.
    #[cheatcode(group = String)]
    function parseBytes(string calldata stringifiedValue) external pure returns (bytes memory parsedValue);
    /// Parses the given `string` into an `address`.
    #[cheatcode(group = String)]
    function parseAddress(string calldata stringifiedValue) external pure returns (address parsedValue);
    /// Parses the given `string` into a `uint256`.
    #[cheatcode(group = String)]
    function parseUint(string calldata stringifiedValue) external pure returns (uint256 parsedValue);
    /// Parses the given `string` into a `int256`.
    #[cheatcode(group = String)]
    function parseInt(string calldata stringifiedValue) external pure returns (int256 parsedValue);
    /// Parses the given `string` into a `bytes32`.
    #[cheatcode(group = String)]
    function parseBytes32(string calldata stringifiedValue) external pure returns (bytes32 parsedValue);
    /// Parses the given `string` into a `bool`.
    #[cheatcode(group = String)]
    function parseBool(string calldata stringifiedValue) external pure returns (bool parsedValue);

    // ======== JSON Parsing and Manipulation ========

    // -------- Reading --------

    // NOTE: Please read https://book.getfoundry.sh/cheatcodes/parse-json to understand the
    // limitations and caveats of the JSON parsing cheats.

    /// Checks if `key` exists in a JSON object.
    #[cheatcode(group = Json)]
    function keyExists(string calldata json, string calldata key) external view returns (bool);

    /// ABI-encodes a JSON object.
    #[cheatcode(group = Json)]
    function parseJson(string calldata json) external pure returns (bytes memory abiEncodedData);
    /// ABI-encodes a JSON object at `key`.
    #[cheatcode(group = Json)]
    function parseJson(string calldata json, string calldata key) external pure returns (bytes memory abiEncodedData);

    // The following parseJson cheatcodes will do type coercion, for the type that they indicate.
    // For example, parseJsonUint will coerce all values to a uint256. That includes stringified numbers '12.'
    // and hex numbers '0xEF.'.
    // Type coercion works ONLY for discrete values or arrays. That means that the key must return a value or array, not
    // a JSON object.

    /// Parses a string of JSON data at `key` and coerces it to `uint256`.
    #[cheatcode(group = Json)]
    function parseJsonUint(string calldata json, string calldata key) external pure returns (uint256);
    /// Parses a string of JSON data at `key` and coerces it to `uint256[]`.
    #[cheatcode(group = Json)]
    function parseJsonUintArray(string calldata json, string calldata key) external pure returns (uint256[] memory);
    /// Parses a string of JSON data at `key` and coerces it to `int256`.
    #[cheatcode(group = Json)]
    function parseJsonInt(string calldata json, string calldata key) external pure returns (int256);
    /// Parses a string of JSON data at `key` and coerces it to `int256[]`.
    #[cheatcode(group = Json)]
    function parseJsonIntArray(string calldata json, string calldata key) external pure returns (int256[] memory);
    /// Parses a string of JSON data at `key` and coerces it to `bool`.
    #[cheatcode(group = Json)]
    function parseJsonBool(string calldata json, string calldata key) external pure returns (bool);
    /// Parses a string of JSON data at `key` and coerces it to `bool[]`.
    #[cheatcode(group = Json)]
    function parseJsonBoolArray(string calldata json, string calldata key) external pure returns (bool[] memory);
    /// Parses a string of JSON data at `key` and coerces it to `address`.
    #[cheatcode(group = Json)]
    function parseJsonAddress(string calldata json, string calldata key) external pure returns (address);
    /// Parses a string of JSON data at `key` and coerces it to `address[]`.
    #[cheatcode(group = Json)]
    function parseJsonAddressArray(string calldata json, string calldata key)
        external
        pure
        returns (address[] memory);
    /// Parses a string of JSON data at `key` and coerces it to `string`.
    #[cheatcode(group = Json)]
    function parseJsonString(string calldata json, string calldata key) external pure returns (string memory);
    /// Parses a string of JSON data at `key` and coerces it to `string[]`.
    #[cheatcode(group = Json)]
    function parseJsonStringArray(string calldata json, string calldata key) external pure returns (string[] memory);
    /// Parses a string of JSON data at `key` and coerces it to `bytes`.
    #[cheatcode(group = Json)]
    function parseJsonBytes(string calldata json, string calldata key) external pure returns (bytes memory);
    /// Parses a string of JSON data at `key` and coerces it to `bytes[]`.
    #[cheatcode(group = Json)]
    function parseJsonBytesArray(string calldata json, string calldata key) external pure returns (bytes[] memory);
    /// Parses a string of JSON data at `key` and coerces it to `bytes32`.
    #[cheatcode(group = Json)]
    function parseJsonBytes32(string calldata json, string calldata key) external pure returns (bytes32);
    /// Parses a string of JSON data at `key` and coerces it to `bytes32[]`.
    #[cheatcode(group = Json)]
    function parseJsonBytes32Array(string calldata json, string calldata key)
        external
        pure
        returns (bytes32[] memory);

    /// Returns an array of all the keys in a JSON object.
    #[cheatcode(group = Json)]
    function parseJsonKeys(string calldata json, string calldata key) external pure returns (string[] memory keys);

    // -------- Writing --------

    // NOTE: Please read https://book.getfoundry.sh/cheatcodes/serialize-json to understand how
    // to use the serialization cheats.

    /// Serializes a key and value to a JSON object stored in-memory that can be later written to a file.
    /// Returns the stringified version of the specific JSON file up to that moment.
    #[cheatcode(group = Json)]
    function serializeJson(string calldata objectKey, string calldata value) external returns (string memory json);

    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeBool(string calldata objectKey, string calldata valueKey, bool value)
        external
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeUint(string calldata objectKey, string calldata valueKey, uint256 value)
        external
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeInt(string calldata objectKey, string calldata valueKey, int256 value)
        external
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeAddress(string calldata objectKey, string calldata valueKey, address value)
        external
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeBytes32(string calldata objectKey, string calldata valueKey, bytes32 value)
        external
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeString(string calldata objectKey, string calldata valueKey, string calldata value)
        external
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeBytes(string calldata objectKey, string calldata valueKey, bytes calldata value)
        external
        returns (string memory json);

    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeBool(string calldata objectKey, string calldata valueKey, bool[] calldata values)
        external
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeUint(string calldata objectKey, string calldata valueKey, uint256[] calldata values)
        external
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeInt(string calldata objectKey, string calldata valueKey, int256[] calldata values)
        external
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeAddress(string calldata objectKey, string calldata valueKey, address[] calldata values)
        external
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeBytes32(string calldata objectKey, string calldata valueKey, bytes32[] calldata values)
        external
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeString(string calldata objectKey, string calldata valueKey, string[] calldata values)
        external
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeBytes(string calldata objectKey, string calldata valueKey, bytes[] calldata values)
        external
        returns (string memory json);

    // NOTE: Please read https://book.getfoundry.sh/cheatcodes/write-json to understand how
    // to use the JSON writing cheats.

    /// Write a serialized JSON object to a file. If the file exists, it will be overwritten.
    #[cheatcode(group = Json)]
    function writeJson(string calldata json, string calldata path) external;

    /// Write a serialized JSON object to an **existing** JSON file, replacing a value with key = <value_key.>
    /// This is useful to replace a specific value of a JSON file, without having to parse the entire thing.
    #[cheatcode(group = Json)]
    function writeJson(string calldata json, string calldata path, string calldata valueKey) external;

    // -------- Key Management --------

    /// Derives a private key from the name, labels the account with that name, and returns the wallet.
    #[cheatcode(group = Utilities)]
    function createWallet(string calldata walletLabel) external returns (Wallet memory wallet);

    /// Generates a wallet from the private key and returns the wallet.
    #[cheatcode(group = Utilities)]
    function createWallet(uint256 privateKey) external returns (Wallet memory wallet);

    /// Generates a wallet from the private key, labels the account with that name, and returns the wallet.
    #[cheatcode(group = Utilities)]
    function createWallet(uint256 privateKey, string calldata walletLabel) external returns (Wallet memory wallet);

    /// Get a `Wallet`'s nonce.
    #[cheatcode(group = Utilities)]
    function getNonce(Wallet calldata wallet) external returns (uint64 nonce);

    /// Signs data with a `Wallet`.
    #[cheatcode(group = Utilities)]
    function sign(Wallet calldata wallet, bytes32 digest) external returns (uint8 v, bytes32 r, bytes32 s);

    /// Derive a private key from a provided mnenomic string (or mnenomic file path)
    /// at the derivation path `m/44'/60'/0'/0/{index}`.
    #[cheatcode(group = Utilities)]
    function deriveKey(string calldata mnemonic, uint32 index) external pure returns (uint256 privateKey);
    /// Derive a private key from a provided mnenomic string (or mnenomic file path)
    /// at `{derivationPath}{index}`.
    #[cheatcode(group = Utilities)]
    function deriveKey(string calldata mnemonic, string calldata derivationPath, uint32 index)
        external
        pure
        returns (uint256 privateKey);
    /// Derive a private key from a provided mnenomic string (or mnenomic file path) in the specified language
    /// at the derivation path `m/44'/60'/0'/0/{index}`.
    #[cheatcode(group = Utilities)]
    function deriveKey(string calldata mnemonic, uint32 index, string calldata language)
        external
        pure
        returns (uint256 privateKey);
    /// Derive a private key from a provided mnenomic string (or mnenomic file path) in the specified language
    /// at `{derivationPath}{index}`.
    #[cheatcode(group = Utilities)]
    function deriveKey(string calldata mnemonic, string calldata derivationPath, uint32 index, string calldata language)
        external
        pure
        returns (uint256 privateKey);

    /// Adds a private key to the local forge wallet and returns the address.
    #[cheatcode(group = Utilities)]
    function rememberKey(uint256 privateKey) external returns (address keyAddr);

    // -------- Uncategorized Utilities --------

    /// Labels an address in call traces.
    #[cheatcode(group = Utilities)]
    function label(address account, string calldata newLabel) external;

    /// Gets the label for the specified address.
    #[cheatcode(group = Utilities)]
    function getLabel(address account) external returns (string memory currentLabel);

    /// Compute the address a contract will be deployed at for a given deployer address and nonce.
    #[cheatcode(group = Utilities)]
    function computeCreateAddress(address deployer, uint256 nonce) external pure returns (address);

    /// Compute the address of a contract created with CREATE2 using the given CREATE2 deployer.
    #[cheatcode(group = Utilities)]
    function computeCreate2Address(bytes32 salt, bytes32 initCodeHash, address deployer) external pure returns (address);

    /// Compute the address of a contract created with CREATE2 using the default CREATE2 deployer.
    #[cheatcode(group = Utilities)]
    function computeCreate2Address(bytes32 salt, bytes32 initCodeHash) external pure returns (address);
}
}
