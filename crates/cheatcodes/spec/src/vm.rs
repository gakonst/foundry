// We don't document function parameters individually so we can't enable `missing_docs` for this
// module. Instead, we emit custom diagnostics in `#[derive(Cheatcode)]`.
#![allow(missing_docs)]

use super::{Requires::Isolation, *};
use crate::Vm::ForgeContext;
use alloy_sol_types::sol;
use foundry_macros::Cheatcode;
use std::fmt;

sol! {
// Cheatcodes are marked as view/pure/none using the following rules:
// 0. A call's observable behaviour includes its return value, logs, reverts and state writes,
// 1. If you can influence a later call's observable behaviour, you're neither `view` nor `pure`
//    (you are modifying some state be it the EVM, interpreter, filesystem, etc),
// 2. Otherwise if you can be influenced by an earlier call, or if reading some state, you're `view`,
// 3. Otherwise you're `pure`.

/// Foundry cheatcodes interface.
#[derive(Debug, Cheatcode)] // Keep this list small to avoid unnecessary bloat.
#[sol(abi)]
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

    /// The kind of account access that occurred.
    enum AccountAccessKind {
        /// The account was called.
        Call,
        /// The account was called via delegatecall.
        DelegateCall,
        /// The account was called via callcode.
        CallCode,
        /// The account was called via staticcall.
        StaticCall,
        /// The account was created.
        Create,
        /// The account was selfdestructed.
        SelfDestruct,
        /// Synthetic access indicating the current context has resumed after a previous sub-context (AccountAccess).
        Resume,
        /// The account's balance was read.
        Balance,
        /// The account's codesize was read.
        Extcodesize,
        /// The account's codehash was read.
        Extcodehash,
        /// The account's code was copied.
        Extcodecopy,
    }

    /// Forge execution contexts.
    enum ForgeContext {
        /// Test group execution context (test, coverage or snapshot).
        TestGroup,
        /// `forge test` execution context.
        Test,
        /// `forge coverage` execution context.
        Coverage,
        /// `forge snapshot` execution context.
        Snapshot,
        /// Script group execution context (dry run, broadcast or resume).
        ScriptGroup,
        /// `forge script` execution context.
        ScriptDryRun,
        /// `forge script --broadcast` execution context.
        ScriptBroadcast,
        /// `forge script --resume` execution context.
        ScriptResume,
        /// Unknown `forge` execution context.
        Unknown,
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

    /// Gas used. Returned by `lastCallGas`.
    struct Gas {
        /// The gas limit of the call.
        uint64 gasLimit;
        /// The total gas used.
        uint64 gasTotalUsed;
        /// DEPRECATED: The amount of gas used for memory expansion. Ref: <https://github.com/foundry-rs/foundry/pull/7934#pullrequestreview-2069236939>
        uint64 gasMemoryUsed;
        /// The amount of gas refunded.
        int64 gasRefunded;
        /// The amount of gas remaining.
        uint64 gasRemaining;
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
    /// This structure is returned from the `fsMetadata` function and represents known
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

    /// The result of a `tryFfi` call.
    struct FfiResult {
        /// The exit code of the call.
        int32 exitCode;
        /// The optionally hex-decoded `stdout` data.
        bytes stdout;
        /// The `stderr` data.
        bytes stderr;
    }

    /// Information on the chain and fork.
    struct ChainInfo {
        /// The fork identifier. Set to zero if no fork is active.
        uint256 forkId;
        /// The chain ID of the current fork.
        uint256 chainId;
    }

    /// The storage accessed during an `AccountAccess`.
    struct StorageAccess {
        /// The account whose storage was accessed.
        address account;
        /// The slot that was accessed.
        bytes32 slot;
        /// If the access was a write.
        bool isWrite;
        /// The previous value of the slot.
        bytes32 previousValue;
        /// The new value of the slot.
        bytes32 newValue;
        /// If the access was reverted.
        bool reverted;
    }

    /// The result of a `stopAndReturnStateDiff` call.
    struct AccountAccess {
        /// The chain and fork the access occurred.
        ChainInfo chainInfo;
        /// The kind of account access that determines what the account is.
        /// If kind is Call, DelegateCall, StaticCall or CallCode, then the account is the callee.
        /// If kind is Create, then the account is the newly created account.
        /// If kind is SelfDestruct, then the account is the selfdestruct recipient.
        /// If kind is a Resume, then account represents a account context that has resumed.
        AccountAccessKind kind;
        /// The account that was accessed.
        /// It's either the account created, callee or a selfdestruct recipient for CREATE, CALL or SELFDESTRUCT.
        address account;
        /// What accessed the account.
        address accessor;
        /// If the account was initialized or empty prior to the access.
        /// An account is considered initialized if it has code, a
        /// non-zero nonce, or a non-zero balance.
        bool initialized;
        /// The previous balance of the accessed account.
        uint256 oldBalance;
        /// The potential new balance of the accessed account.
        /// That is, all balance changes are recorded here, even if reverts occurred.
        uint256 newBalance;
        /// Code of the account deployed by CREATE.
        bytes deployedCode;
        /// Value passed along with the account access
        uint256 value;
        /// Input data provided to the CREATE or CALL
        bytes data;
        /// If this access reverted in either the current or parent context.
        bool reverted;
        /// An ordered list of storage accesses made during an account access operation.
        StorageAccess[] storageAccesses;
        /// Call depth traversed during the recording of state differences
        uint64 depth;
    }

    /// The result of the `stopDebugTraceRecording` call
    struct DebugStep {
        /// The stack before executing the step of the run.
        /// stack\[0\] represents the top of the stack.
        /// and only stack data relevant to the opcode execution is contained.
        uint256[] stack;
        /// The memory input data before executing the step of the run.
        /// only input data relevant to the opcode execution is contained.
        ///
        /// e.g. for MLOAD, it will have memory\[offset:offset+32\] copied here.
        /// the offset value can be get by the stack data.
        bytes memoryInput;
        /// The opcode that was accessed.
        uint8 opcode;
        /// The call depth of the step.
        uint64 depth;
        /// Whether the call end up with out of gas error.
        bool isOutOfGas;
        /// The contract address where the opcode is running
        address contractAddr;
    }

    /// The transaction type (`txType`) of the broadcast.
    enum BroadcastTxType {
        /// Represents a CALL broadcast tx.
        Call,
        /// Represents a CREATE broadcast tx.
        Create,
        /// Represents a CREATE2 broadcast tx.
        Create2
    }

    /// Represents a transaction's broadcast details.
    struct BroadcastTxSummary {
        /// The hash of the transaction that was broadcasted
        bytes32 txHash;
        /// Represent the type of transaction among CALL, CREATE, CREATE2
        BroadcastTxType txType;
        /// The address of the contract that was called or created.
        /// This is address of the contract that is created if the txType is CREATE or CREATE2.
        address contractAddress;
        /// The block number the transaction landed in.
        uint64 blockNumber;
        /// Status of the transaction, retrieved from the transaction receipt.
        bool success;
    }

    /// Holds a signed EIP-7702 authorization for an authority account to delegate to an implementation.
    struct SignedDelegation {
        /// The y-parity of the recovered secp256k1 signature (0 or 1).
        uint8 v;
        /// First 32 bytes of the signature.
        bytes32 r;
        /// Second 32 bytes of the signature.
        bytes32 s;
        /// The current nonce of the authority account at signing time.
        /// Used to ensure signature can't be replayed after account nonce changes.
        uint64 nonce;
        /// Address of the contract implementation that will be delegated to.
        /// Gets encoded into delegation code: 0xef0100 || implementation.
        address implementation;
    }

    /// Represents a "potential" revert reason from a single subsequent call when using `vm.assumeNoReverts`.
    /// Reverts that match will result in a FOUNDRY::ASSUME rejection, whereas unmatched reverts will be surfaced
    /// as normal.
    struct PotentialRevert {
        /// The allowed origin of the revert opcode; address(0) allows reverts from any address
        address reverter;
        /// When true, only matches on the beginning of the revert data, otherwise, matches on entire revert data
        bool partialMatch;
        /// The data to use to match encountered reverts
        bytes revertData;
    }

    // ======== EVM ========

    /// Gets the address for a given private key.
    #[cheatcode(group = Evm, safety = Safe)]
    function addr(uint256 privateKey) external pure returns (address keyAddr);

    /// Dump a genesis JSON file's `allocs` to disk.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function dumpState(string calldata pathToStateJson) external;

    /// Gets the nonce of an account.
    #[cheatcode(group = Evm, safety = Safe)]
    function getNonce(address account) external view returns (uint64 nonce);

    /// Get the nonce of a `Wallet`.
    #[cheatcode(group = Evm, safety = Safe)]
    function getNonce(Wallet calldata wallet) external returns (uint64 nonce);

    /// Loads a storage slot from an address.
    #[cheatcode(group = Evm, safety = Safe)]
    function load(address target, bytes32 slot) external view returns (bytes32 data);

    /// Load a genesis JSON file's `allocs` into the in-memory EVM state.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function loadAllocs(string calldata pathToAllocsJson) external;

    // -------- Record Debug Traces --------

    /// Records the debug trace during the run.
    #[cheatcode(group = Evm, safety = Safe)]
    function startDebugTraceRecording() external;

    /// Stop debug trace recording and returns the recorded debug trace.
    #[cheatcode(group = Evm, safety = Safe)]
    function stopAndReturnDebugTraceRecording() external returns (DebugStep[] memory step);


    /// Clones a source account code, state, balance and nonce to a target account and updates in-memory EVM state.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function cloneAccount(address source, address target) external;

    // -------- Record Storage --------

    /// Records all storage reads and writes.
    #[cheatcode(group = Evm, safety = Safe)]
    function record() external;

    /// Gets all accessed reads and write slot from a `vm.record` session, for a given address.
    #[cheatcode(group = Evm, safety = Safe)]
    function accesses(address target) external returns (bytes32[] memory readSlots, bytes32[] memory writeSlots);

    /// Record all account accesses as part of CREATE, CALL or SELFDESTRUCT opcodes in order,
    /// along with the context of the calls
    #[cheatcode(group = Evm, safety = Safe)]
    function startStateDiffRecording() external;

    /// Returns an ordered array of all account accesses from a `vm.startStateDiffRecording` session.
    #[cheatcode(group = Evm, safety = Safe)]
    function stopAndReturnStateDiff() external returns (AccountAccess[] memory accountAccesses);

    /// Returns state diffs from current `vm.startStateDiffRecording` session.
    #[cheatcode(group = Evm, safety = Safe)]
    function getStateDiff() external view returns (string memory diff);

    /// Returns state diffs from current `vm.startStateDiffRecording` session, in json format.
    #[cheatcode(group = Evm, safety = Safe)]
    function getStateDiffJson() external view returns (string memory diff);

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
    /// Sets `block.prevrandao`.
    /// Not available on EVM versions before Paris. Use `difficulty` instead.
    /// If used on unsupported EVM versions it will revert.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function prevrandao(uint256 newPrevrandao) external;

    /// Sets the blobhashes in the transaction.
    /// Not available on EVM versions before Cancun.
    /// If used on unsupported EVM versions it will revert.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function blobhashes(bytes32[] calldata hashes) external;

    /// Gets the blockhashes from the current transaction.
    /// Not available on EVM versions before Cancun.
    /// If used on unsupported EVM versions it will revert.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function getBlobhashes() external view returns (bytes32[] memory hashes);

    /// Sets `block.height`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function roll(uint256 newHeight) external;

    /// Gets the current `block.number`.
    /// You should use this instead of `block.number` if you use `vm.roll`, as `block.number` is assumed to be constant across a transaction,
    /// and as a result will get optimized out by the compiler.
    /// See https://github.com/foundry-rs/foundry/issues/6180
    #[cheatcode(group = Evm, safety = Safe)]
    function getBlockNumber() external view returns (uint256 height);

    /// Sets `tx.gasprice`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function txGasPrice(uint256 newGasPrice) external;

    /// Sets `block.timestamp`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function warp(uint256 newTimestamp) external;

    /// Gets the current `block.timestamp`.
    /// You should use this instead of `block.timestamp` if you use `vm.warp`, as `block.timestamp` is assumed to be constant across a transaction,
    /// and as a result will get optimized out by the compiler.
    /// See https://github.com/foundry-rs/foundry/issues/6180
    #[cheatcode(group = Evm, safety = Safe)]
    function getBlockTimestamp() external view returns (uint256 timestamp);

    /// Sets `block.blobbasefee`
    #[cheatcode(group = Evm, safety = Unsafe)]
    function blobBaseFee(uint256 newBlobBaseFee) external;

    /// Gets the current `block.blobbasefee`.
    /// You should use this instead of `block.blobbasefee` if you use `vm.blobBaseFee`, as `block.blobbasefee` is assumed to be constant across a transaction,
    /// and as a result will get optimized out by the compiler.
    /// See https://github.com/foundry-rs/foundry/issues/6180
    #[cheatcode(group = Evm, safety = Safe)]
    function getBlobBaseFee() external view returns (uint256 blobBaseFee);

    /// Set blockhash for the current block.
    /// It only sets the blockhash for blocks where `block.number - 256 <= number < block.number`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function setBlockhash(uint256 blockNumber, bytes32 blockHash) external;

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

    /// Marks the slots of an account and the account address as cold.
    #[cheatcode(group = Evm, safety = Unsafe, status = Experimental)]
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

    /// Mocks a call to an address, returning specified data.
    /// Calldata can either be strict or a partial match, e.g. if you only
    /// pass a Solidity selector to the expected calldata, then the entire Solidity
    /// function will be mocked.
    ///
    /// Overload to pass the function selector directly `token.approve.selector` instead of `abi.encodeWithSelector(token.approve.selector)`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockCall(address callee, bytes4 data, bytes calldata returnData) external;

    /// Mocks a call to an address with a specific `msg.value`, returning specified data.
    /// Calldata match takes precedence over `msg.value` in case of ambiguity.
    ///
    /// Overload to pass the function selector directly `token.approve.selector` instead of `abi.encodeWithSelector(token.approve.selector)`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockCall(address callee, uint256 msgValue, bytes4 data, bytes calldata returnData) external;

    /// Mocks multiple calls to an address, returning specified data for each call.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockCalls(address callee, bytes calldata data, bytes[] calldata returnData) external;

    /// Mocks multiple calls to an address with a specific `msg.value`, returning specified data for each call.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockCalls(address callee, uint256 msgValue, bytes calldata data, bytes[] calldata returnData) external;

    /// Reverts a call to an address with specified revert data.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockCallRevert(address callee, bytes calldata data, bytes calldata revertData) external;

    /// Reverts a call to an address with a specific `msg.value`, with specified revert data.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockCallRevert(address callee, uint256 msgValue, bytes calldata data, bytes calldata revertData)
        external;

    /// Reverts a call to an address with specified revert data.
    ///
    /// Overload to pass the function selector directly `token.approve.selector` instead of `abi.encodeWithSelector(token.approve.selector)`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockCallRevert(address callee, bytes4 data, bytes calldata revertData) external;

    /// Reverts a call to an address with a specific `msg.value`, with specified revert data.
    ///
    /// Overload to pass the function selector directly `token.approve.selector` instead of `abi.encodeWithSelector(token.approve.selector)`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockCallRevert(address callee, uint256 msgValue, bytes4 data, bytes calldata revertData)
        external;

    /// Whenever a call is made to `callee` with calldata `data`, this cheatcode instead calls
    /// `target` with the same calldata. This functionality is similar to a delegate call made to
    /// `target` contract from `callee`.
    /// Can be used to substitute a call to a function with another implementation that captures
    /// the primary logic of the original function but is easier to reason about.
    /// If calldata is not a strict match then partial match by selector is attempted.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function mockFunction(address callee, address target, bytes calldata data) external;

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

    /// Sets the *next* delegate call's `msg.sender` to be the input address.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function prank(address msgSender, bool delegateCall) external;

    /// Sets all subsequent delegate calls' `msg.sender` to be the input address until `stopPrank` is called.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function startPrank(address msgSender, bool delegateCall) external;

    /// Sets the *next* delegate call's `msg.sender` to be the input address, and the `tx.origin` to be the second input.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function prank(address msgSender, address txOrigin, bool delegateCall) external;

    /// Sets all subsequent delegate calls' `msg.sender` to be the input address until `stopPrank` is called, and the `tx.origin` to be the second input.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function startPrank(address msgSender, address txOrigin, bool delegateCall) external;

    /// Resets subsequent calls' `msg.sender` to be `address(this)`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function stopPrank() external;

    /// Reads the current `msg.sender` and `tx.origin` from state and reports if there is any active caller modification.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function readCallers() external returns (CallerMode callerMode, address msgSender, address txOrigin);

    // ----- Arbitrary Snapshots -----

    /// Snapshot capture an arbitrary numerical value by name.
    /// The group name is derived from the contract name.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function snapshotValue(string calldata name, uint256 value) external;

    /// Snapshot capture an arbitrary numerical value by name in a group.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function snapshotValue(string calldata group, string calldata name, uint256 value) external;

    // -------- Gas Snapshots --------

    /// Snapshot capture the gas usage of the last call by name from the callee perspective.
    #[cheatcode(group = Evm, safety = Unsafe, requires = Isolation)]
    function snapshotGasLastCall(string calldata name) external returns (uint256 gasUsed);

    /// Snapshot capture the gas usage of the last call by name in a group from the callee perspective.
    #[cheatcode(group = Evm, safety = Unsafe, requires = Isolation)]
    function snapshotGasLastCall(string calldata group, string calldata name) external returns (uint256 gasUsed);

    /// Start a snapshot capture of the current gas usage by name.
    /// The group name is derived from the contract name.
    #[cheatcode(group = Evm, safety = Unsafe, requires = Isolation)]
    function startSnapshotGas(string calldata name) external;

    /// Start a snapshot capture of the current gas usage by name in a group.
    #[cheatcode(group = Evm, safety = Unsafe, requires = Isolation)]
    function startSnapshotGas(string calldata group, string calldata name) external;

    /// Stop the snapshot capture of the current gas by latest snapshot name, capturing the gas used since the start.
    #[cheatcode(group = Evm, safety = Unsafe, requires = Isolation)]
    function stopSnapshotGas() external returns (uint256 gasUsed);

    /// Stop the snapshot capture of the current gas usage by name, capturing the gas used since the start.
    /// The group name is derived from the contract name.
    #[cheatcode(group = Evm, safety = Unsafe, requires = Isolation)]
    function stopSnapshotGas(string calldata name) external returns (uint256 gasUsed);

    /// Stop the snapshot capture of the current gas usage by name in a group, capturing the gas used since the start.
    #[cheatcode(group = Evm, safety = Unsafe, requires = Isolation)]
    function stopSnapshotGas(string calldata group, string calldata name) external returns (uint256 gasUsed);

    // -------- State Snapshots --------

    /// `snapshot` is being deprecated in favor of `snapshotState`. It will be removed in future versions.
    #[cheatcode(group = Evm, safety = Unsafe, status = Deprecated(Some("replaced by `snapshotState`")))]
    function snapshot() external returns (uint256 snapshotId);

    /// Snapshot the current state of the evm.
    /// Returns the ID of the snapshot that was created.
    /// To revert a snapshot use `revertToState`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function snapshotState() external returns (uint256 snapshotId);

    /// `revertTo` is being deprecated in favor of `revertToState`. It will be removed in future versions.
    #[cheatcode(group = Evm, safety = Unsafe, status =  Deprecated(Some("replaced by `revertToState`")))]
    function revertTo(uint256 snapshotId) external returns (bool success);

    /// Revert the state of the EVM to a previous snapshot
    /// Takes the snapshot ID to revert to.
    ///
    /// Returns `true` if the snapshot was successfully reverted.
    /// Returns `false` if the snapshot does not exist.
    ///
    /// **Note:** This does not automatically delete the snapshot. To delete the snapshot use `deleteStateSnapshot`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function revertToState(uint256 snapshotId) external returns (bool success);

    /// `revertToAndDelete` is being deprecated in favor of `revertToStateAndDelete`. It will be removed in future versions.
    #[cheatcode(group = Evm, safety = Unsafe, status = Deprecated(Some("replaced by `revertToStateAndDelete`")))]
    function revertToAndDelete(uint256 snapshotId) external returns (bool success);

    /// Revert the state of the EVM to a previous snapshot and automatically deletes the snapshots
    /// Takes the snapshot ID to revert to.
    ///
    /// Returns `true` if the snapshot was successfully reverted and deleted.
    /// Returns `false` if the snapshot does not exist.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function revertToStateAndDelete(uint256 snapshotId) external returns (bool success);

    /// `deleteSnapshot` is being deprecated in favor of `deleteStateSnapshot`. It will be removed in future versions.
    #[cheatcode(group = Evm, safety = Unsafe, status = Deprecated(Some("replaced by `deleteStateSnapshot`")))]
    function deleteSnapshot(uint256 snapshotId) external returns (bool success);

    /// Removes the snapshot with the given ID created by `snapshot`.
    /// Takes the snapshot ID to delete.
    ///
    /// Returns `true` if the snapshot was successfully deleted.
    /// Returns `false` if the snapshot does not exist.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function deleteStateSnapshot(uint256 snapshotId) external returns (bool success);

    /// `deleteSnapshots` is being deprecated in favor of `deleteStateSnapshots`. It will be removed in future versions.
    #[cheatcode(group = Evm, safety = Unsafe, status = Deprecated(Some("replaced by `deleteStateSnapshots`")))]
    function deleteSnapshots() external;

    /// Removes _all_ snapshots previously created by `snapshot`.
    #[cheatcode(group = Evm, safety = Unsafe)]
    function deleteStateSnapshots() external;

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

    /// Performs an Ethereum JSON-RPC request to the given endpoint.
    #[cheatcode(group = Evm, safety = Safe)]
    function rpc(string calldata urlOrAlias, string calldata method, string calldata params)
        external
        returns (bytes memory data);

    /// Gets all the logs according to specified filter.
    #[cheatcode(group = Evm, safety = Safe)]
    function eth_getLogs(uint256 fromBlock, uint256 toBlock, address target, bytes32[] calldata topics)
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

    /// Reset gas metering (i.e. gas usage is set to gas limit).
    #[cheatcode(group = Evm, safety = Safe)]
    function resetGasMetering() external;

    // -------- Gas Measurement --------

    /// Gets the gas used in the last call from the callee perspective.
    #[cheatcode(group = Evm, safety = Safe)]
    function lastCallGas() external view returns (Gas memory gas);

    // ======== Test Assertions and Utilities ========

    /// If the condition is false, discard this run's fuzz inputs and generate new ones.
    #[cheatcode(group = Testing, safety = Safe)]
    function assume(bool condition) external pure;

    /// Discard this run's fuzz inputs and generate new ones if next call reverted.
    #[cheatcode(group = Testing, safety = Safe)]
    function assumeNoRevert() external pure;

    /// Discard this run's fuzz inputs and generate new ones if next call reverts with the potential revert parameters.
    #[cheatcode(group = Testing, safety = Safe)]
    function assumeNoRevert(PotentialRevert calldata potentialRevert) external pure;

    /// Discard this run's fuzz inputs and generate new ones if next call reverts with the any of the potential revert parameters.
    #[cheatcode(group = Testing, safety = Safe)]
    function assumeNoRevert(PotentialRevert[] calldata potentialReverts) external pure;

    /// Writes a breakpoint to jump to in the debugger.
    #[cheatcode(group = Testing, safety = Safe)]
    function breakpoint(string calldata char) external pure;

    /// Writes a conditional breakpoint to jump to in the debugger.
    #[cheatcode(group = Testing, safety = Safe)]
    function breakpoint(string calldata char, bool value) external pure;

    /// Returns the Foundry version.
    /// Format: <cargo_version>-<tag>+<git_sha_short>.<unix_build_timestamp>.<profile>
    /// Sample output: 0.3.0-nightly+3cb96bde9b.1737036656.debug
    /// Note: Build timestamps may vary slightly across platforms due to separate CI jobs.
    /// For reliable version comparisons, use UNIX format (e.g., >= 1700000000)
    /// to compare timestamps while ignoring minor time differences.
    #[cheatcode(group = Testing, safety = Safe)]
    function getFoundryVersion() external view returns (string memory version);

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

    /// Expect a given number of logs with the provided topics.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectEmit(bool checkTopic1, bool checkTopic2, bool checkTopic3, bool checkData, uint64 count) external;

    /// Expect a given number of logs from a specific emitter with the provided topics.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectEmit(bool checkTopic1, bool checkTopic2, bool checkTopic3, bool checkData, address emitter, uint64 count)
        external;

    /// Expect a given number of logs with all topic and data checks enabled.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectEmit(uint64 count) external;

    /// Expect a given number of logs from a specific emitter with all topic and data checks enabled.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectEmit(address emitter, uint64 count) external;

    /// Prepare an expected anonymous log with (bool checkTopic1, bool checkTopic2, bool checkTopic3, bool checkData.).
    /// Call this function, then emit an anonymous event, then call a function. Internally after the call, we check if
    /// logs were emitted in the expected order with the expected topics and data (as specified by the booleans).
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectEmitAnonymous(bool checkTopic0, bool checkTopic1, bool checkTopic2, bool checkTopic3, bool checkData) external;

    /// Same as the previous method, but also checks supplied address against emitting contract.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectEmitAnonymous(bool checkTopic0, bool checkTopic1, bool checkTopic2, bool checkTopic3, bool checkData, address emitter)
        external;

    /// Prepare an expected anonymous log with all topic and data checks enabled.
    /// Call this function, then emit an anonymous event, then call a function. Internally after the call, we check if
    /// logs were emitted in the expected order with the expected topics and data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectEmitAnonymous() external;

    /// Same as the previous method, but also checks supplied address against emitting contract.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectEmitAnonymous(address emitter) external;

    /// Expects an error on next call with any revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert() external;

    /// Expects an error on next call that exactly matches the revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(bytes4 revertData) external;

    /// Expects an error on next call that exactly matches the revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(bytes calldata revertData) external;

    /// Expects an error with any revert data on next call to reverter address.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(address reverter) external;

    /// Expects an error from reverter address on next call, with any revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(bytes4 revertData, address reverter) external;

    /// Expects an error from reverter address on next call, that exactly matches the revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(bytes calldata revertData, address reverter) external;

    /// Expects a `count` number of reverts from the upcoming calls with any revert data or reverter.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(uint64 count) external;

    /// Expects a `count` number of reverts from the upcoming calls that match the revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(bytes4 revertData, uint64 count) external;

    /// Expects a `count` number of reverts from the upcoming calls that exactly match the revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(bytes calldata revertData, uint64 count) external;

    /// Expects a `count` number of reverts from the upcoming calls from the reverter address.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(address reverter, uint64 count) external;

    /// Expects a `count` number of reverts from the upcoming calls from the reverter address that match the revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(bytes4 revertData, address reverter, uint64 count) external;

    /// Expects a `count` number of reverts from the upcoming calls from the reverter address that exactly match the revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectRevert(bytes calldata revertData, address reverter, uint64 count) external;

    /// Expects an error on next call that starts with the revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectPartialRevert(bytes4 revertData) external;

    /// Expects an error on next call to reverter address, that starts with the revert data.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectPartialRevert(bytes4 revertData, address reverter) external;

    /// Expects an error on next cheatcode call with any revert data.
    #[cheatcode(group = Testing, safety = Unsafe, status = Internal)]
    function _expectCheatcodeRevert() external;

    /// Expects an error on next cheatcode call that starts with the revert data.
    #[cheatcode(group = Testing, safety = Unsafe, status = Internal)]
    function _expectCheatcodeRevert(bytes4 revertData) external;

    /// Expects an error on next cheatcode call that exactly matches the revert data.
    #[cheatcode(group = Testing, safety = Unsafe, status = Internal)]
    function _expectCheatcodeRevert(bytes calldata revertData) external;

    /// Only allows memory writes to offsets [0x00, 0x60) ∪ [min, max) in the current subcontext. If any other
    /// memory is written to, the test will fail. Can be called multiple times to add more ranges to the set.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectSafeMemory(uint64 min, uint64 max) external;

    /// Stops all safe memory expectation in the current subcontext.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function stopExpectSafeMemory() external;

    /// Only allows memory writes to offsets [0x00, 0x60) ∪ [min, max) in the next created subcontext.
    /// If any other memory is written to, the test will fail. Can be called multiple times to add more ranges
    /// to the set.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function expectSafeMemoryCall(uint64 min, uint64 max) external;

    /// Marks a test as skipped. Must be called at the top level of a test.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function skip(bool skipTest) external;

    /// Marks a test as skipped with a reason. Must be called at the top level of a test.
    #[cheatcode(group = Testing, safety = Unsafe)]
    function skip(bool skipTest, string calldata reason) external;

    /// Asserts that the given condition is true.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertTrue(bool condition) external pure;

    /// Asserts that the given condition is true and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertTrue(bool condition, string calldata error) external pure;

    /// Asserts that the given condition is false.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertFalse(bool condition) external pure;

    /// Asserts that the given condition is false and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertFalse(bool condition, string calldata error) external pure;

    /// Asserts that two `bool` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(bool left, bool right) external pure;

    /// Asserts that two `bool` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(bool left, bool right, string calldata error) external pure;

    /// Asserts that two `uint256` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(uint256 left, uint256 right) external pure;

    /// Asserts that two `uint256` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(uint256 left, uint256 right, string calldata error) external pure;

    /// Asserts that two `int256` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(int256 left, int256 right) external pure;

    /// Asserts that two `int256` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(int256 left, int256 right, string calldata error) external pure;

    /// Asserts that two `address` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(address left, address right) external pure;

    /// Asserts that two `address` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(address left, address right, string calldata error) external pure;

    /// Asserts that two `bytes32` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(bytes32 left, bytes32 right) external pure;

    /// Asserts that two `bytes32` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(bytes32 left, bytes32 right, string calldata error) external pure;

    /// Asserts that two `string` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(string calldata left, string calldata right) external pure;

    /// Asserts that two `string` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(string calldata left, string calldata right, string calldata error) external pure;

    /// Asserts that two `bytes` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(bytes calldata left, bytes calldata right) external pure;

    /// Asserts that two `bytes` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(bytes calldata left, bytes calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `bool` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(bool[] calldata left, bool[] calldata right) external pure;

    /// Asserts that two arrays of `bool` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(bool[] calldata left, bool[] calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `uint256 values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(uint256[] calldata left, uint256[] calldata right) external pure;

    /// Asserts that two arrays of `uint256` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(uint256[] calldata left, uint256[] calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `int256` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(int256[] calldata left, int256[] calldata right) external pure;

    /// Asserts that two arrays of `int256` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(int256[] calldata left, int256[] calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `address` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(address[] calldata left, address[] calldata right) external pure;

    /// Asserts that two arrays of `address` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(address[] calldata left, address[] calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `bytes32` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(bytes32[] calldata left, bytes32[] calldata right) external pure;

    /// Asserts that two arrays of `bytes32` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(bytes32[] calldata left, bytes32[] calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `string` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(string[] calldata left, string[] calldata right) external pure;

    /// Asserts that two arrays of `string` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(string[] calldata left, string[] calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `bytes` values are equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(bytes[] calldata left, bytes[] calldata right) external pure;

    /// Asserts that two arrays of `bytes` values are equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEq(bytes[] calldata left, bytes[] calldata right, string calldata error) external pure;

    /// Asserts that two `uint256` values are equal, formatting them with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEqDecimal(uint256 left, uint256 right, uint256 decimals) external pure;

    /// Asserts that two `uint256` values are equal, formatting them with decimals in failure message.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEqDecimal(uint256 left, uint256 right, uint256 decimals, string calldata error) external pure;

    /// Asserts that two `int256` values are equal, formatting them with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEqDecimal(int256 left, int256 right, uint256 decimals) external pure;

    /// Asserts that two `int256` values are equal, formatting them with decimals in failure message.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertEqDecimal(int256 left, int256 right, uint256 decimals, string calldata error) external pure;

    /// Asserts that two `bool` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(bool left, bool right) external pure;

    /// Asserts that two `bool` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(bool left, bool right, string calldata error) external pure;

    /// Asserts that two `uint256` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(uint256 left, uint256 right) external pure;

    /// Asserts that two `uint256` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(uint256 left, uint256 right, string calldata error) external pure;

    /// Asserts that two `int256` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(int256 left, int256 right) external pure;

    /// Asserts that two `int256` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(int256 left, int256 right, string calldata error) external pure;

    /// Asserts that two `address` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(address left, address right) external pure;

    /// Asserts that two `address` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(address left, address right, string calldata error) external pure;

    /// Asserts that two `bytes32` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(bytes32 left, bytes32 right) external pure;

    /// Asserts that two `bytes32` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(bytes32 left, bytes32 right, string calldata error) external pure;

    /// Asserts that two `string` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(string calldata left, string calldata right) external pure;

    /// Asserts that two `string` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(string calldata left, string calldata right, string calldata error) external pure;

    /// Asserts that two `bytes` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(bytes calldata left, bytes calldata right) external pure;

    /// Asserts that two `bytes` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(bytes calldata left, bytes calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `bool` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(bool[] calldata left, bool[] calldata right) external pure;

    /// Asserts that two arrays of `bool` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(bool[] calldata left, bool[] calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `uint256` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(uint256[] calldata left, uint256[] calldata right) external pure;

    /// Asserts that two arrays of `uint256` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(uint256[] calldata left, uint256[] calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `int256` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(int256[] calldata left, int256[] calldata right) external pure;

    /// Asserts that two arrays of `int256` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(int256[] calldata left, int256[] calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `address` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(address[] calldata left, address[] calldata right) external pure;

    /// Asserts that two arrays of `address` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(address[] calldata left, address[] calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `bytes32` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(bytes32[] calldata left, bytes32[] calldata right) external pure;

    /// Asserts that two arrays of `bytes32` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(bytes32[] calldata left, bytes32[] calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `string` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(string[] calldata left, string[] calldata right) external pure;

    /// Asserts that two arrays of `string` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(string[] calldata left, string[] calldata right, string calldata error) external pure;

    /// Asserts that two arrays of `bytes` values are not equal.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(bytes[] calldata left, bytes[] calldata right) external pure;

    /// Asserts that two arrays of `bytes` values are not equal and includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEq(bytes[] calldata left, bytes[] calldata right, string calldata error) external pure;

    /// Asserts that two `uint256` values are not equal, formatting them with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEqDecimal(uint256 left, uint256 right, uint256 decimals) external pure;

    /// Asserts that two `uint256` values are not equal, formatting them with decimals in failure message.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEqDecimal(uint256 left, uint256 right, uint256 decimals, string calldata error) external pure;

    /// Asserts that two `int256` values are not equal, formatting them with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEqDecimal(int256 left, int256 right, uint256 decimals) external pure;

    /// Asserts that two `int256` values are not equal, formatting them with decimals in failure message.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertNotEqDecimal(int256 left, int256 right, uint256 decimals, string calldata error) external pure;

    /// Compares two `uint256` values. Expects first value to be greater than second.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGt(uint256 left, uint256 right) external pure;

    /// Compares two `uint256` values. Expects first value to be greater than second.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGt(uint256 left, uint256 right, string calldata error) external pure;

    /// Compares two `int256` values. Expects first value to be greater than second.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGt(int256 left, int256 right) external pure;

    /// Compares two `int256` values. Expects first value to be greater than second.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGt(int256 left, int256 right, string calldata error) external pure;

    /// Compares two `uint256` values. Expects first value to be greater than second.
    /// Formats values with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGtDecimal(uint256 left, uint256 right, uint256 decimals) external pure;

    /// Compares two `uint256` values. Expects first value to be greater than second.
    /// Formats values with decimals in failure message. Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGtDecimal(uint256 left, uint256 right, uint256 decimals, string calldata error) external pure;

    /// Compares two `int256` values. Expects first value to be greater than second.
    /// Formats values with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGtDecimal(int256 left, int256 right, uint256 decimals) external pure;

    /// Compares two `int256` values. Expects first value to be greater than second.
    /// Formats values with decimals in failure message. Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGtDecimal(int256 left, int256 right, uint256 decimals, string calldata error) external pure;

    /// Compares two `uint256` values. Expects first value to be greater than or equal to second.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGe(uint256 left, uint256 right) external pure;

    /// Compares two `uint256` values. Expects first value to be greater than or equal to second.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGe(uint256 left, uint256 right, string calldata error) external pure;

    /// Compares two `int256` values. Expects first value to be greater than or equal to second.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGe(int256 left, int256 right) external pure;

    /// Compares two `int256` values. Expects first value to be greater than or equal to second.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGe(int256 left, int256 right, string calldata error) external pure;

    /// Compares two `uint256` values. Expects first value to be greater than or equal to second.
    /// Formats values with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGeDecimal(uint256 left, uint256 right, uint256 decimals) external pure;

    /// Compares two `uint256` values. Expects first value to be greater than or equal to second.
    /// Formats values with decimals in failure message. Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGeDecimal(uint256 left, uint256 right, uint256 decimals, string calldata error) external pure;

    /// Compares two `int256` values. Expects first value to be greater than or equal to second.
    /// Formats values with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGeDecimal(int256 left, int256 right, uint256 decimals) external pure;

    /// Compares two `int256` values. Expects first value to be greater than or equal to second.
    /// Formats values with decimals in failure message. Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertGeDecimal(int256 left, int256 right, uint256 decimals, string calldata error) external pure;

    /// Compares two `uint256` values. Expects first value to be less than second.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLt(uint256 left, uint256 right) external pure;

    /// Compares two `uint256` values. Expects first value to be less than second.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLt(uint256 left, uint256 right, string calldata error) external pure;

    /// Compares two `int256` values. Expects first value to be less than second.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLt(int256 left, int256 right) external pure;

    /// Compares two `int256` values. Expects first value to be less than second.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLt(int256 left, int256 right, string calldata error) external pure;

    /// Compares two `uint256` values. Expects first value to be less than second.
    /// Formats values with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLtDecimal(uint256 left, uint256 right, uint256 decimals) external pure;

    /// Compares two `uint256` values. Expects first value to be less than second.
    /// Formats values with decimals in failure message. Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLtDecimal(uint256 left, uint256 right, uint256 decimals, string calldata error) external pure;

    /// Compares two `int256` values. Expects first value to be less than second.
    /// Formats values with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLtDecimal(int256 left, int256 right, uint256 decimals) external pure;

    /// Compares two `int256` values. Expects first value to be less than second.
    /// Formats values with decimals in failure message. Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLtDecimal(int256 left, int256 right, uint256 decimals, string calldata error) external pure;

    /// Compares two `uint256` values. Expects first value to be less than or equal to second.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLe(uint256 left, uint256 right) external pure;

    /// Compares two `uint256` values. Expects first value to be less than or equal to second.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLe(uint256 left, uint256 right, string calldata error) external pure;

    /// Compares two `int256` values. Expects first value to be less than or equal to second.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLe(int256 left, int256 right) external pure;

    /// Compares two `int256` values. Expects first value to be less than or equal to second.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLe(int256 left, int256 right, string calldata error) external pure;

    /// Compares two `uint256` values. Expects first value to be less than or equal to second.
    /// Formats values with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLeDecimal(uint256 left, uint256 right, uint256 decimals) external pure;

    /// Compares two `uint256` values. Expects first value to be less than or equal to second.
    /// Formats values with decimals in failure message. Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLeDecimal(uint256 left, uint256 right, uint256 decimals, string calldata error) external pure;

    /// Compares two `int256` values. Expects first value to be less than or equal to second.
    /// Formats values with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLeDecimal(int256 left, int256 right, uint256 decimals) external pure;

    /// Compares two `int256` values. Expects first value to be less than or equal to second.
    /// Formats values with decimals in failure message. Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertLeDecimal(int256 left, int256 right, uint256 decimals, string calldata error) external pure;

    /// Compares two `uint256` values. Expects difference to be less than or equal to `maxDelta`.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqAbs(uint256 left, uint256 right, uint256 maxDelta) external pure;

    /// Compares two `uint256` values. Expects difference to be less than or equal to `maxDelta`.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqAbs(uint256 left, uint256 right, uint256 maxDelta, string calldata error) external pure;

    /// Compares two `int256` values. Expects difference to be less than or equal to `maxDelta`.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqAbs(int256 left, int256 right, uint256 maxDelta) external pure;

    /// Compares two `int256` values. Expects difference to be less than or equal to `maxDelta`.
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqAbs(int256 left, int256 right, uint256 maxDelta, string calldata error) external pure;

    /// Compares two `uint256` values. Expects difference to be less than or equal to `maxDelta`.
    /// Formats values with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqAbsDecimal(uint256 left, uint256 right, uint256 maxDelta, uint256 decimals) external pure;

    /// Compares two `uint256` values. Expects difference to be less than or equal to `maxDelta`.
    /// Formats values with decimals in failure message. Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqAbsDecimal(
        uint256 left,
        uint256 right,
        uint256 maxDelta,
        uint256 decimals,
        string calldata error
    ) external pure;

    /// Compares two `int256` values. Expects difference to be less than or equal to `maxDelta`.
    /// Formats values with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqAbsDecimal(int256 left, int256 right, uint256 maxDelta, uint256 decimals) external pure;

    /// Compares two `int256` values. Expects difference to be less than or equal to `maxDelta`.
    /// Formats values with decimals in failure message. Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqAbsDecimal(
        int256 left,
        int256 right,
        uint256 maxDelta,
        uint256 decimals,
        string calldata error
    ) external pure;

    /// Compares two `uint256` values. Expects relative difference in percents to be less than or equal to `maxPercentDelta`.
    /// `maxPercentDelta` is an 18 decimal fixed point number, where 1e18 == 100%
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqRel(uint256 left, uint256 right, uint256 maxPercentDelta) external pure;

    /// Compares two `uint256` values. Expects relative difference in percents to be less than or equal to `maxPercentDelta`.
    /// `maxPercentDelta` is an 18 decimal fixed point number, where 1e18 == 100%
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqRel(uint256 left, uint256 right, uint256 maxPercentDelta, string calldata error) external pure;

    /// Compares two `int256` values. Expects relative difference in percents to be less than or equal to `maxPercentDelta`.
    /// `maxPercentDelta` is an 18 decimal fixed point number, where 1e18 == 100%
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqRel(int256 left, int256 right, uint256 maxPercentDelta) external pure;

    /// Compares two `int256` values. Expects relative difference in percents to be less than or equal to `maxPercentDelta`.
    /// `maxPercentDelta` is an 18 decimal fixed point number, where 1e18 == 100%
    /// Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqRel(int256 left, int256 right, uint256 maxPercentDelta, string calldata error) external pure;

    /// Compares two `uint256` values. Expects relative difference in percents to be less than or equal to `maxPercentDelta`.
    /// `maxPercentDelta` is an 18 decimal fixed point number, where 1e18 == 100%
    /// Formats values with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqRelDecimal(
        uint256 left,
        uint256 right,
        uint256 maxPercentDelta,
        uint256 decimals
    ) external pure;

    /// Compares two `uint256` values. Expects relative difference in percents to be less than or equal to `maxPercentDelta`.
    /// `maxPercentDelta` is an 18 decimal fixed point number, where 1e18 == 100%
    /// Formats values with decimals in failure message. Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqRelDecimal(
        uint256 left,
        uint256 right,
        uint256 maxPercentDelta,
        uint256 decimals,
        string calldata error
    ) external pure;

    /// Compares two `int256` values. Expects relative difference in percents to be less than or equal to `maxPercentDelta`.
    /// `maxPercentDelta` is an 18 decimal fixed point number, where 1e18 == 100%
    /// Formats values with decimals in failure message.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqRelDecimal(
        int256 left,
        int256 right,
        uint256 maxPercentDelta,
        uint256 decimals
    ) external pure;

    /// Compares two `int256` values. Expects relative difference in percents to be less than or equal to `maxPercentDelta`.
    /// `maxPercentDelta` is an 18 decimal fixed point number, where 1e18 == 100%
    /// Formats values with decimals in failure message. Includes error message into revert string on failure.
    #[cheatcode(group = Testing, safety = Safe)]
    function assertApproxEqRelDecimal(
        int256 left,
        int256 right,
        uint256 maxPercentDelta,
        uint256 decimals,
        string calldata error
    ) external pure;

    // ======== OS and Filesystem ========

    // -------- Metadata --------

    /// Returns true if the given path points to an existing entity, else returns false.
    #[cheatcode(group = Filesystem)]
    function exists(string calldata path) external view returns (bool result);

    /// Given a path, query the file system to get information about a file, directory, etc.
    #[cheatcode(group = Filesystem)]
    function fsMetadata(string calldata path) external view returns (FsMetadata memory metadata);

    /// Returns true if the path exists on disk and is pointing at a directory, else returns false.
    #[cheatcode(group = Filesystem)]
    function isDir(string calldata path) external view returns (bool result);

    /// Returns true if the path exists on disk and is pointing at a regular file, else returns false.
    #[cheatcode(group = Filesystem)]
    function isFile(string calldata path) external view returns (bool result);

    /// Get the path of the current project root.
    #[cheatcode(group = Filesystem)]
    function projectRoot() external view returns (string memory path);

    /// Returns the time since unix epoch in milliseconds.
    #[cheatcode(group = Filesystem)]
    function unixTime() external view returns (uint256 milliseconds);

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

    /// Gets the artifact path from code (aka. creation code).
    #[cheatcode(group = Filesystem)]
    function getArtifactPathByCode(bytes calldata code) external view returns (string memory path);

    /// Gets the artifact path from deployed code (aka. runtime code).
    #[cheatcode(group = Filesystem)]
    function getArtifactPathByDeployedCode(bytes calldata deployedCode) external view returns (string memory path);

    /// Gets the creation bytecode from an artifact file. Takes in the relative path to the json file or the path to the
    /// artifact in the form of <path>:<contract>:<version> where <contract> and <version> parts are optional.
    #[cheatcode(group = Filesystem)]
    function getCode(string calldata artifactPath) external view returns (bytes memory creationBytecode);

    /// Deploys a contract from an artifact file. Takes in the relative path to the json file or the path to the
    /// artifact in the form of <path>:<contract>:<version> where <contract> and <version> parts are optional.
    #[cheatcode(group = Filesystem)]
    function deployCode(string calldata artifactPath) external returns (address deployedAddress);

    /// Deploys a contract from an artifact file. Takes in the relative path to the json file or the path to the
    /// artifact in the form of <path>:<contract>:<version> where <contract> and <version> parts are optional.
    ///
    /// Additionally accepts abi-encoded constructor arguments.
    #[cheatcode(group = Filesystem)]
    function deployCode(string calldata artifactPath, bytes calldata constructorArgs) external returns (address deployedAddress);

    /// Gets the deployed bytecode from an artifact file. Takes in the relative path to the json file or the path to the
    /// artifact in the form of <path>:<contract>:<version> where <contract> and <version> parts are optional.
    #[cheatcode(group = Filesystem)]
    function getDeployedCode(string calldata artifactPath) external view returns (bytes memory runtimeBytecode);

    /// Returns the most recent broadcast for the given contract on `chainId` matching `txType`.
    ///
    /// For example:
    ///
    /// The most recent deployment can be fetched by passing `txType` as `CREATE` or `CREATE2`.
    ///
    /// The most recent call can be fetched by passing `txType` as `CALL`.
    #[cheatcode(group = Filesystem)]
    function getBroadcast(string calldata contractName, uint64 chainId, BroadcastTxType txType) external view returns (BroadcastTxSummary memory);

    /// Returns all broadcasts for the given contract on `chainId` with the specified `txType`.
    ///
    /// Sorted such that the most recent broadcast is the first element, and the oldest is the last. i.e descending order of BroadcastTxSummary.blockNumber.
    #[cheatcode(group = Filesystem)]
    function getBroadcasts(string calldata contractName, uint64 chainId, BroadcastTxType txType) external view returns (BroadcastTxSummary[] memory);

    /// Returns all broadcasts for the given contract on `chainId`.
    ///
    /// Sorted such that the most recent broadcast is the first element, and the oldest is the last. i.e descending order of BroadcastTxSummary.blockNumber.
    #[cheatcode(group = Filesystem)]
    function getBroadcasts(string calldata contractName, uint64 chainId) external view returns (BroadcastTxSummary[] memory);

    /// Returns the most recent deployment for the current `chainId`.
    #[cheatcode(group = Filesystem)]
    function getDeployment(string calldata contractName) external view returns (address deployedAddress);

    /// Returns the most recent deployment for the given contract on `chainId`
    #[cheatcode(group = Filesystem)]
    function getDeployment(string calldata contractName, uint64 chainId) external view returns (address deployedAddress);

    /// Returns all deployments for the given contract on `chainId`
    ///
    /// Sorted in descending order of deployment time i.e descending order of BroadcastTxSummary.blockNumber.
    ///
    /// The most recent deployment is the first element, and the oldest is the last.
    #[cheatcode(group = Filesystem)]
    function getDeployments(string calldata contractName, uint64 chainId) external view returns (address[] memory deployedAddresses);

    // -------- Foreign Function Interface --------

    /// Performs a foreign function call via the terminal.
    #[cheatcode(group = Filesystem)]
    function ffi(string[] calldata commandInput) external returns (bytes memory result);

    /// Performs a foreign function call via terminal and returns the exit code, stdout, and stderr.
    #[cheatcode(group = Filesystem)]
    function tryFfi(string[] calldata commandInput) external returns (FfiResult memory result);

    // -------- User Interaction --------

    /// Prompts the user for a string value in the terminal.
    #[cheatcode(group = Filesystem)]
    function prompt(string calldata promptText) external returns (string memory input);

    /// Prompts the user for a hidden string value in the terminal.
    #[cheatcode(group = Filesystem)]
    function promptSecret(string calldata promptText) external returns (string memory input);

    /// Prompts the user for hidden uint256 in the terminal (usually pk).
    #[cheatcode(group = Filesystem)]
    function promptSecretUint(string calldata promptText) external returns (uint256);

    /// Prompts the user for an address in the terminal.
    #[cheatcode(group = Filesystem)]
    function promptAddress(string calldata promptText) external returns (address);

    /// Prompts the user for uint256 in the terminal.
    #[cheatcode(group = Filesystem)]
    function promptUint(string calldata promptText) external returns (uint256);

    // ======== Environment Variables ========

    /// Sets environment variables.
    #[cheatcode(group = Environment)]
    function setEnv(string calldata name, string calldata value) external;

    /// Gets the environment variable `name` and returns true if it exists, else returns false.
    #[cheatcode(group = Environment)]
    function envExists(string calldata name) external view returns (bool result);

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
    function envOr(string calldata name, bool defaultValue) external view returns (bool value);
    /// Gets the environment variable `name` and parses it as `uint256`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, uint256 defaultValue) external view returns (uint256 value);
    /// Gets the environment variable `name` and parses it as `int256`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, int256 defaultValue) external view returns (int256 value);
    /// Gets the environment variable `name` and parses it as `address`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, address defaultValue) external view returns (address value);
    /// Gets the environment variable `name` and parses it as `bytes32`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, bytes32 defaultValue) external view returns (bytes32 value);
    /// Gets the environment variable `name` and parses it as `string`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata defaultValue) external view returns (string memory value);
    /// Gets the environment variable `name` and parses it as `bytes`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, bytes calldata defaultValue) external view returns (bytes memory value);

    /// Gets the environment variable `name` and parses it as an array of `bool`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, bool[] calldata defaultValue)
        external view
        returns (bool[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `uint256`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, uint256[] calldata defaultValue)
        external view
        returns (uint256[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `int256`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, int256[] calldata defaultValue)
        external view
        returns (int256[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `address`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, address[] calldata defaultValue)
        external view
        returns (address[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `bytes32`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, bytes32[] calldata defaultValue)
        external view
        returns (bytes32[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `string`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, string[] calldata defaultValue)
        external view
        returns (string[] memory value);
    /// Gets the environment variable `name` and parses it as an array of `bytes`, delimited by `delim`.
    /// Reverts if the variable could not be parsed.
    /// Returns `defaultValue` if the variable was not found.
    #[cheatcode(group = Environment)]
    function envOr(string calldata name, string calldata delim, bytes[] calldata defaultValue)
        external view
        returns (bytes[] memory value);

    /// Returns true if `forge` command was executed in given context.
    #[cheatcode(group = Environment)]
    function isContext(ForgeContext context) external view returns (bool result);

    // ======== Scripts ========

    // -------- Broadcasting Transactions --------

    /// Has the next call (at this call depth only) create transactions that can later be signed and sent onchain.
    ///
    /// Broadcasting address is determined by checking the following in order:
    /// 1. If `--sender` argument was provided, that address is used.
    /// 2. If exactly one signer (e.g. private key, hw wallet, keystore) is set when `forge broadcast` is invoked, that signer is used.
    /// 3. Otherwise, default foundry sender (1804c8AB1F12E6bbf3894d4083f33e07309d1f38) is used.
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

    /// Has all subsequent calls (at this call depth only) create transactions that can later be signed and sent onchain.
    ///
    /// Broadcasting address is determined by checking the following in order:
    /// 1. If `--sender` argument was provided, that address is used.
    /// 2. If exactly one signer (e.g. private key, hw wallet, keystore) is set when `forge broadcast` is invoked, that signer is used.
    /// 3. Otherwise, default foundry sender (1804c8AB1F12E6bbf3894d4083f33e07309d1f38) is used.
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

    /// Takes a signed transaction and broadcasts it to the network.
    #[cheatcode(group = Scripting)]
    function broadcastRawTransaction(bytes calldata data) external;

    /// Sign an EIP-7702 authorization for delegation
    #[cheatcode(group = Scripting)]
    function signDelegation(address implementation, uint256 privateKey) external returns (SignedDelegation memory signedDelegation);

    /// Designate the next call as an EIP-7702 transaction
    #[cheatcode(group = Scripting)]
    function attachDelegation(SignedDelegation calldata signedDelegation) external;

    /// Sign an EIP-7702 authorization and designate the next call as an EIP-7702 transaction
    #[cheatcode(group = Scripting)]
    function signAndAttachDelegation(address implementation, uint256 privateKey) external returns (SignedDelegation memory signedDelegation);

    /// Returns addresses of available unlocked wallets in the script environment.
    #[cheatcode(group = Scripting)]
    function getWallets() external returns (address[] memory wallets);

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

    /// Converts the given `string` value to Lowercase.
    #[cheatcode(group = String)]
    function toLowercase(string calldata input) external pure returns (string memory output);
    /// Converts the given `string` value to Uppercase.
    #[cheatcode(group = String)]
    function toUppercase(string calldata input) external pure returns (string memory output);
    /// Trims leading and trailing whitespace from the given `string` value.
    #[cheatcode(group = String)]
    function trim(string calldata input) external pure returns (string memory output);
    /// Replaces occurrences of `from` in the given `string` with `to`.
    #[cheatcode(group = String)]
    function replace(string calldata input, string calldata from, string calldata to) external pure returns (string memory output);
    /// Splits the given `string` into an array of strings divided by the `delimiter`.
    #[cheatcode(group = String)]
    function split(string calldata input, string calldata delimiter) external pure returns (string[] memory outputs);
    /// Returns the index of the first occurrence of a `key` in an `input` string.
    /// Returns `NOT_FOUND` (i.e. `type(uint256).max`) if the `key` is not found.
    /// Returns 0 in case of an empty `key`.
    #[cheatcode(group = String)]
    function indexOf(string calldata input, string calldata key) external pure returns (uint256);
    /// Returns true if `search` is found in `subject`, false otherwise.
    #[cheatcode(group = String)]
    function contains(string calldata subject, string calldata search) external returns (bool result);

    // ======== JSON Parsing and Manipulation ========

    // -------- Reading --------

    // NOTE: Please read https://book.getfoundry.sh/cheatcodes/parse-json to understand the
    // limitations and caveats of the JSON parsing cheats.

    /// Checks if `key` exists in a JSON object
    /// `keyExists` is being deprecated in favor of `keyExistsJson`. It will be removed in future versions.
    #[cheatcode(group = Json, status = Deprecated(Some("replaced by `keyExistsJson`")))]
    function keyExists(string calldata json, string calldata key) external view returns (bool);
    /// Checks if `key` exists in a JSON object.
    #[cheatcode(group = Json)]
    function keyExistsJson(string calldata json, string calldata key) external view returns (bool);

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

    /// Parses a string of JSON data and coerces it to type corresponding to `typeDescription`.
    #[cheatcode(group = Json)]
    function parseJsonType(string calldata json, string calldata typeDescription) external pure returns (bytes memory);
    /// Parses a string of JSON data at `key` and coerces it to type corresponding to `typeDescription`.
    #[cheatcode(group = Json)]
    function parseJsonType(string calldata json, string calldata key, string calldata typeDescription) external pure returns (bytes memory);
    /// Parses a string of JSON data at `key` and coerces it to type array corresponding to `typeDescription`.
    #[cheatcode(group = Json)]
    function parseJsonTypeArray(string calldata json, string calldata key, string calldata typeDescription)
        external
        pure
        returns (bytes memory);

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
    function serializeUintToHex(string calldata objectKey, string calldata valueKey, uint256 value)
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
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeJsonType(string calldata typeDescription, bytes calldata value)
        external
        pure
        returns (string memory json);
    /// See `serializeJson`.
    #[cheatcode(group = Json)]
    function serializeJsonType(string calldata objectKey, string calldata valueKey, string calldata typeDescription, bytes calldata value)
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

    // ======== TOML Parsing and Manipulation ========

    // -------- Reading --------

    // NOTE: Please read https://book.getfoundry.sh/cheatcodes/parse-toml to understand the
    // limitations and caveats of the TOML parsing cheat.

    /// Checks if `key` exists in a TOML table.
    #[cheatcode(group = Toml)]
    function keyExistsToml(string calldata toml, string calldata key) external view returns (bool);

    /// ABI-encodes a TOML table.
    #[cheatcode(group = Toml)]
    function parseToml(string calldata toml) external pure returns (bytes memory abiEncodedData);

    /// ABI-encodes a TOML table at `key`.
    #[cheatcode(group = Toml)]
    function parseToml(string calldata toml, string calldata key) external pure returns (bytes memory abiEncodedData);

    // The following parseToml cheatcodes will do type coercion, for the type that they indicate.
    // For example, parseTomlUint will coerce all values to a uint256. That includes stringified numbers '12.'
    // and hex numbers '0xEF.'.
    // Type coercion works ONLY for discrete values or arrays. That means that the key must return a value or array, not
    // a TOML table.

    /// Parses a string of TOML data at `key` and coerces it to `uint256`.
    #[cheatcode(group = Toml)]
    function parseTomlUint(string calldata toml, string calldata key) external pure returns (uint256);
    /// Parses a string of TOML data at `key` and coerces it to `uint256[]`.
    #[cheatcode(group = Toml)]
    function parseTomlUintArray(string calldata toml, string calldata key) external pure returns (uint256[] memory);
    /// Parses a string of TOML data at `key` and coerces it to `int256`.
    #[cheatcode(group = Toml)]
    function parseTomlInt(string calldata toml, string calldata key) external pure returns (int256);
    /// Parses a string of TOML data at `key` and coerces it to `int256[]`.
    #[cheatcode(group = Toml)]
    function parseTomlIntArray(string calldata toml, string calldata key) external pure returns (int256[] memory);
    /// Parses a string of TOML data at `key` and coerces it to `bool`.
    #[cheatcode(group = Toml)]
    function parseTomlBool(string calldata toml, string calldata key) external pure returns (bool);
    /// Parses a string of TOML data at `key` and coerces it to `bool[]`.
    #[cheatcode(group = Toml)]
    function parseTomlBoolArray(string calldata toml, string calldata key) external pure returns (bool[] memory);
    /// Parses a string of TOML data at `key` and coerces it to `address`.
    #[cheatcode(group = Toml)]
    function parseTomlAddress(string calldata toml, string calldata key) external pure returns (address);
    /// Parses a string of TOML data at `key` and coerces it to `address[]`.
    #[cheatcode(group = Toml)]
    function parseTomlAddressArray(string calldata toml, string calldata key)
        external
        pure
        returns (address[] memory);
    /// Parses a string of TOML data at `key` and coerces it to `string`.
    #[cheatcode(group = Toml)]
    function parseTomlString(string calldata toml, string calldata key) external pure returns (string memory);
    /// Parses a string of TOML data at `key` and coerces it to `string[]`.
    #[cheatcode(group = Toml)]
    function parseTomlStringArray(string calldata toml, string calldata key) external pure returns (string[] memory);
    /// Parses a string of TOML data at `key` and coerces it to `bytes`.
    #[cheatcode(group = Toml)]
    function parseTomlBytes(string calldata toml, string calldata key) external pure returns (bytes memory);
    /// Parses a string of TOML data at `key` and coerces it to `bytes[]`.
    #[cheatcode(group = Toml)]
    function parseTomlBytesArray(string calldata toml, string calldata key) external pure returns (bytes[] memory);
    /// Parses a string of TOML data at `key` and coerces it to `bytes32`.
    #[cheatcode(group = Toml)]
    function parseTomlBytes32(string calldata toml, string calldata key) external pure returns (bytes32);
    /// Parses a string of TOML data at `key` and coerces it to `bytes32[]`.
    #[cheatcode(group = Toml)]
    function parseTomlBytes32Array(string calldata toml, string calldata key)
        external
        pure
        returns (bytes32[] memory);

    /// Parses a string of TOML data and coerces it to type corresponding to `typeDescription`.
    #[cheatcode(group = Toml)]
    function parseTomlType(string calldata toml, string calldata typeDescription) external pure returns (bytes memory);
    /// Parses a string of TOML data at `key` and coerces it to type corresponding to `typeDescription`.
    #[cheatcode(group = Toml)]
    function parseTomlType(string calldata toml, string calldata key, string calldata typeDescription) external pure returns (bytes memory);
    /// Parses a string of TOML data at `key` and coerces it to type array corresponding to `typeDescription`.
    #[cheatcode(group = Toml)]
    function parseTomlTypeArray(string calldata toml, string calldata key, string calldata typeDescription)
        external
        pure
        returns (bytes memory);

    /// Returns an array of all the keys in a TOML table.
    #[cheatcode(group = Toml)]
    function parseTomlKeys(string calldata toml, string calldata key) external pure returns (string[] memory keys);

    // -------- Writing --------

    // NOTE: Please read https://book.getfoundry.sh/cheatcodes/write-toml to understand how
    // to use the TOML writing cheat.

    /// Takes serialized JSON, converts to TOML and write a serialized TOML to a file.
    #[cheatcode(group = Toml)]
    function writeToml(string calldata json, string calldata path) external;

    /// Takes serialized JSON, converts to TOML and write a serialized TOML table to an **existing** TOML file, replacing a value with key = <value_key.>
    /// This is useful to replace a specific value of a TOML file, without having to parse the entire thing.
    #[cheatcode(group = Toml)]
    function writeToml(string calldata json, string calldata path, string calldata valueKey) external;

    // ======== Cryptography ========

    // -------- Key Management --------

    /// Derives a private key from the name, labels the account with that name, and returns the wallet.
    #[cheatcode(group = Crypto)]
    function createWallet(string calldata walletLabel) external returns (Wallet memory wallet);

    /// Generates a wallet from the private key and returns the wallet.
    #[cheatcode(group = Crypto)]
    function createWallet(uint256 privateKey) external returns (Wallet memory wallet);

    /// Generates a wallet from the private key, labels the account with that name, and returns the wallet.
    #[cheatcode(group = Crypto)]
    function createWallet(uint256 privateKey, string calldata walletLabel) external returns (Wallet memory wallet);

    /// Signs data with a `Wallet`.
    #[cheatcode(group = Crypto)]
    function sign(Wallet calldata wallet, bytes32 digest) external returns (uint8 v, bytes32 r, bytes32 s);

    /// Signs data with a `Wallet`.
    ///
    /// Returns a compact signature (`r`, `vs`) as per EIP-2098, where `vs` encodes both the
    /// signature's `s` value, and the recovery id `v` in a single bytes32.
    /// This format reduces the signature size from 65 to 64 bytes.
    #[cheatcode(group = Crypto)]
    function signCompact(Wallet calldata wallet, bytes32 digest) external returns (bytes32 r, bytes32 vs);

    /// Signs `digest` with `privateKey` using the secp256k1 curve.
    #[cheatcode(group = Crypto)]
    function sign(uint256 privateKey, bytes32 digest) external pure returns (uint8 v, bytes32 r, bytes32 s);

    /// Signs `digest` with `privateKey` using the secp256k1 curve.
    ///
    /// Returns a compact signature (`r`, `vs`) as per EIP-2098, where `vs` encodes both the
    /// signature's `s` value, and the recovery id `v` in a single bytes32.
    /// This format reduces the signature size from 65 to 64 bytes.
    #[cheatcode(group = Crypto)]
    function signCompact(uint256 privateKey, bytes32 digest) external pure returns (bytes32 r, bytes32 vs);

    /// Signs `digest` with signer provided to script using the secp256k1 curve.
    ///
    /// If `--sender` is provided, the signer with provided address is used, otherwise,
    /// if exactly one signer is provided to the script, that signer is used.
    ///
    /// Raises error if signer passed through `--sender` does not match any unlocked signers or
    /// if `--sender` is not provided and not exactly one signer is passed to the script.
    #[cheatcode(group = Crypto)]
    function sign(bytes32 digest) external pure returns (uint8 v, bytes32 r, bytes32 s);

    /// Signs `digest` with signer provided to script using the secp256k1 curve.
    ///
    /// Returns a compact signature (`r`, `vs`) as per EIP-2098, where `vs` encodes both the
    /// signature's `s` value, and the recovery id `v` in a single bytes32.
    /// This format reduces the signature size from 65 to 64 bytes.
    ///
    /// If `--sender` is provided, the signer with provided address is used, otherwise,
    /// if exactly one signer is provided to the script, that signer is used.
    ///
    /// Raises error if signer passed through `--sender` does not match any unlocked signers or
    /// if `--sender` is not provided and not exactly one signer is passed to the script.
    #[cheatcode(group = Crypto)]
    function signCompact(bytes32 digest) external pure returns (bytes32 r, bytes32 vs);

    /// Signs `digest` with signer provided to script using the secp256k1 curve.
    ///
    /// Raises error if none of the signers passed into the script have provided address.
    #[cheatcode(group = Crypto)]
    function sign(address signer, bytes32 digest) external pure returns (uint8 v, bytes32 r, bytes32 s);

    /// Signs `digest` with signer provided to script using the secp256k1 curve.
    ///
    /// Returns a compact signature (`r`, `vs`) as per EIP-2098, where `vs` encodes both the
    /// signature's `s` value, and the recovery id `v` in a single bytes32.
    /// This format reduces the signature size from 65 to 64 bytes.
    ///
    /// Raises error if none of the signers passed into the script have provided address.
    #[cheatcode(group = Crypto)]
    function signCompact(address signer, bytes32 digest) external pure returns (bytes32 r, bytes32 vs);

    /// Signs `digest` with `privateKey` using the secp256r1 curve.
    #[cheatcode(group = Crypto)]
    function signP256(uint256 privateKey, bytes32 digest) external pure returns (bytes32 r, bytes32 s);

    /// Derives secp256r1 public key from the provided `privateKey`.
    #[cheatcode(group = Crypto)]
    function publicKeyP256(uint256 privateKey) external pure returns (uint256 publicKeyX, uint256 publicKeyY);

    /// Derive a private key from a provided mnenomic string (or mnenomic file path)
    /// at the derivation path `m/44'/60'/0'/0/{index}`.
    #[cheatcode(group = Crypto)]
    function deriveKey(string calldata mnemonic, uint32 index) external pure returns (uint256 privateKey);
    /// Derive a private key from a provided mnenomic string (or mnenomic file path)
    /// at `{derivationPath}{index}`.
    #[cheatcode(group = Crypto)]
    function deriveKey(string calldata mnemonic, string calldata derivationPath, uint32 index)
        external
        pure
        returns (uint256 privateKey);
    /// Derive a private key from a provided mnenomic string (or mnenomic file path) in the specified language
    /// at the derivation path `m/44'/60'/0'/0/{index}`.
    #[cheatcode(group = Crypto)]
    function deriveKey(string calldata mnemonic, uint32 index, string calldata language)
        external
        pure
        returns (uint256 privateKey);
    /// Derive a private key from a provided mnenomic string (or mnenomic file path) in the specified language
    /// at `{derivationPath}{index}`.
    #[cheatcode(group = Crypto)]
    function deriveKey(string calldata mnemonic, string calldata derivationPath, uint32 index, string calldata language)
        external
        pure
        returns (uint256 privateKey);

    /// Adds a private key to the local forge wallet and returns the address.
    #[cheatcode(group = Crypto)]
    function rememberKey(uint256 privateKey) external returns (address keyAddr);

    /// Derive a set number of wallets from a mnemonic at the derivation path `m/44'/60'/0'/0/{0..count}`.
    ///
    /// The respective private keys are saved to the local forge wallet for later use and their addresses are returned.
    #[cheatcode(group = Crypto)]
    function rememberKeys(string calldata mnemonic, string calldata derivationPath, uint32 count) external returns (address[] memory keyAddrs);

    /// Derive a set number of wallets from a mnemonic in the specified language at the derivation path `m/44'/60'/0'/0/{0..count}`.
    ///
    /// The respective private keys are saved to the local forge wallet for later use and their addresses are returned.
    #[cheatcode(group = Crypto)]
    function rememberKeys(string calldata mnemonic, string calldata derivationPath, string calldata language, uint32 count)
        external
        returns (address[] memory keyAddrs);

    // -------- Uncategorized Utilities --------

    /// Labels an address in call traces.
    #[cheatcode(group = Utilities)]
    function label(address account, string calldata newLabel) external;

    /// Gets the label for the specified address.
    #[cheatcode(group = Utilities)]
    function getLabel(address account) external view returns (string memory currentLabel);

    /// Compute the address a contract will be deployed at for a given deployer address and nonce.
    #[cheatcode(group = Utilities)]
    function computeCreateAddress(address deployer, uint256 nonce) external pure returns (address);

    /// Compute the address of a contract created with CREATE2 using the given CREATE2 deployer.
    #[cheatcode(group = Utilities)]
    function computeCreate2Address(bytes32 salt, bytes32 initCodeHash, address deployer) external pure returns (address);

    /// Compute the address of a contract created with CREATE2 using the default CREATE2 deployer.
    #[cheatcode(group = Utilities)]
    function computeCreate2Address(bytes32 salt, bytes32 initCodeHash) external pure returns (address);

    /// Encodes a `bytes` value to a base64 string.
    #[cheatcode(group = Utilities)]
    function toBase64(bytes calldata data) external pure returns (string memory);

    /// Encodes a `string` value to a base64 string.
    #[cheatcode(group = Utilities)]
    function toBase64(string calldata data) external pure returns (string memory);

    /// Encodes a `bytes` value to a base64url string.
    #[cheatcode(group = Utilities)]
    function toBase64URL(bytes calldata data) external pure returns (string memory);

    /// Encodes a `string` value to a base64url string.
    #[cheatcode(group = Utilities)]
    function toBase64URL(string calldata data) external pure returns (string memory);

    /// Returns ENS namehash for provided string.
    #[cheatcode(group = Utilities)]
    function ensNamehash(string calldata name) external pure returns (bytes32);

    /// Returns a random uint256 value.
    #[cheatcode(group = Utilities)]
    function randomUint() external returns (uint256);

    /// Returns random uint256 value between the provided range (=min..=max).
    #[cheatcode(group = Utilities)]
    function randomUint(uint256 min, uint256 max) external returns (uint256);

    /// Returns a random `uint256` value of given bits.
    #[cheatcode(group = Utilities)]
    function randomUint(uint256 bits) external view returns (uint256);

    /// Returns a random `address`.
    #[cheatcode(group = Utilities)]
    function randomAddress() external returns (address);

    /// Returns a random `int256` value.
    #[cheatcode(group = Utilities)]
    function randomInt() external view returns (int256);

    /// Returns a random `int256` value of given bits.
    #[cheatcode(group = Utilities)]
    function randomInt(uint256 bits) external view returns (int256);

    /// Returns a random `bool`.
    #[cheatcode(group = Utilities)]
    function randomBool() external view returns (bool);

    /// Returns a random byte array value of the given length.
    #[cheatcode(group = Utilities)]
    function randomBytes(uint256 len) external view returns (bytes memory);

    /// Returns a random fixed-size byte array of length 4.
    #[cheatcode(group = Utilities)]
    function randomBytes4() external view returns (bytes4);

    /// Returns a random fixed-size byte array of length 8.
    #[cheatcode(group = Utilities)]
    function randomBytes8() external view returns (bytes8);

    /// Pauses collection of call traces. Useful in cases when you want to skip tracing of
    /// complex calls which are not useful for debugging.
    #[cheatcode(group = Utilities)]
    function pauseTracing() external view;

    /// Unpauses collection of call traces.
    #[cheatcode(group = Utilities)]
    function resumeTracing() external view;

    /// Utility cheatcode to copy storage of `from` contract to another `to` contract.
    #[cheatcode(group = Utilities)]
    function copyStorage(address from, address to) external;

    /// Utility cheatcode to set arbitrary storage for given target address.
    #[cheatcode(group = Utilities)]
    function setArbitraryStorage(address target) external;
}
}

impl PartialEq for ForgeContext {
    // Handles test group case (any of test, coverage or snapshot)
    // and script group case (any of dry run, broadcast or resume).
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (_, Self::TestGroup) => {
                matches!(self, Self::Test | Self::Snapshot | Self::Coverage)
            }
            (_, Self::ScriptGroup) => {
                matches!(self, Self::ScriptDryRun | Self::ScriptBroadcast | Self::ScriptResume)
            }
            (Self::Test, Self::Test) |
            (Self::Snapshot, Self::Snapshot) |
            (Self::Coverage, Self::Coverage) |
            (Self::ScriptDryRun, Self::ScriptDryRun) |
            (Self::ScriptBroadcast, Self::ScriptBroadcast) |
            (Self::ScriptResume, Self::ScriptResume) |
            (Self::Unknown, Self::Unknown) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Vm::CheatcodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl fmt::Display for Vm::VmErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CheatcodeError(err) => err.fmt(f),
        }
    }
}

#[track_caller]
const fn panic_unknown_safety() -> ! {
    panic!("cannot determine safety from the group, add a `#[cheatcode(safety = ...)]` attribute")
}
