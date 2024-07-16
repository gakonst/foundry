use crate::{
    cmd::{
        access_list::AccessListArgs, bind::BindArgs, call::CallArgs, create2::Create2Args,
        estimate::EstimateArgs, find_block::FindBlockArgs, interface::InterfaceArgs,
        logs::LogsArgs, mktx::MakeTxArgs, rpc::RpcArgs, run::RunArgs, send::SendTxArgs,
        storage::StorageArgs, wallet::WalletSubcommands,
    },
    Cast, SimpleCast,
};
use alloy_primitives::{Address, B256, U256};
use alloy_rpc_types::BlockId;
use clap::{CommandFactory, Parser, Subcommand, ValueHint};
use eyre::Result;
use foundry_cli::opts::{EtherscanOpts, RpcOpts};
use foundry_common::ens::NameOrAddress;
use std::{path::PathBuf, str::FromStr};

use alloy_primitives::{hex, keccak256};
use alloy_provider::Provider;
use alloy_rpc_types::BlockNumberOrTag::Latest;
use clap_complete::generate;
use foundry_cli::{prompt, stdin, utils};
use foundry_common::{
    abi::get_event,
    ens::{namehash, ProviderEnsExt},
    fmt::{format_tokens, format_uint_exp},
    fs,
    selectors::{
        decode_calldata, decode_event_topic, decode_function_selector, decode_selectors,
        import_selectors, parse_signatures, pretty_calldata, ParsedSignatures, SelectorImportData,
        SelectorType,
    },
};
use foundry_config::Config;
use std::time::Instant;

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("VERGEN_GIT_SHA"),
    " ",
    env!("VERGEN_BUILD_TIMESTAMP"),
    ")"
);

/// Perform Ethereum RPC calls from the comfort of your command line.
#[derive(Parser)]
#[command(
    name = "cast",
    version = VERSION_MESSAGE,
    after_help = "Find more information in the book: http://book.getfoundry.sh/reference/cast/cast.html",
    next_display_order = None,
)]
pub struct CastArgs {
    #[command(subcommand)]
    pub cmd: CastSubcommand,
}

#[derive(Subcommand)]
pub enum CastSubcommand {
    /// Prints the maximum value of the given integer type.
    #[command(visible_aliases = &["--max-int", "maxi"])]
    MaxInt {
        /// The integer type to get the maximum value of.
        #[arg(default_value = "int256")]
        r#type: String,
    },

    /// Prints the minimum value of the given integer type.
    #[command(visible_aliases = &["--min-int", "mini"])]
    MinInt {
        /// The integer type to get the minimum value of.
        #[arg(default_value = "int256")]
        r#type: String,
    },

    /// Prints the maximum value of the given integer type.
    #[command(visible_aliases = &["--max-uint", "maxu"])]
    MaxUint {
        /// The unsigned integer type to get the maximum value of.
        #[arg(default_value = "uint256")]
        r#type: String,
    },

    /// Prints the zero address.
    #[command(visible_aliases = &["--address-zero", "az"])]
    AddressZero,

    /// Prints the zero hash.
    #[command(visible_aliases = &["--hash-zero", "hz"])]
    HashZero,

    /// Convert UTF8 text to hex.
    #[command(
        visible_aliases = &[
        "--from-ascii",
        "--from-utf8",
        "from-ascii",
        "fu",
        "fa"]
    )]
    FromUtf8 {
        /// The text to convert.
        text: Option<String>,
    },

    /// Concatenate hex strings.
    #[command(visible_aliases = &["--concat-hex", "ch"])]
    ConcatHex {
        /// The data to concatenate.
        data: Vec<String>,
    },

    /// Convert binary data into hex data.
    #[command(visible_aliases = &["--from-bin", "from-binx", "fb"])]
    FromBin,

    /// Normalize the input to lowercase, 0x-prefixed hex.
    ///
    /// The input can be:
    /// - mixed case hex with or without 0x prefix
    /// - 0x prefixed hex, concatenated with a ':'
    /// - an absolute path to file
    /// - @tag, where the tag is defined in an environment variable
    #[command(visible_aliases = &["--to-hexdata", "thd", "2hd"])]
    ToHexdata {
        /// The input to normalize.
        input: Option<String>,
    },

    /// Convert an address to a checksummed format (EIP-55).
    #[command(
        visible_aliases = &["--to-checksum-address",
        "--to-checksum",
        "to-checksum",
        "ta",
        "2a"]
    )]
    ToCheckSumAddress {
        /// The address to convert.
        address: Option<Address>,
    },

    /// Convert hex data to an ASCII string.
    #[command(visible_aliases = &["--to-ascii", "tas", "2as"])]
    ToAscii {
        /// The hex data to convert.
        hexdata: Option<String>,
    },

    /// Convert hex data to a utf-8 string.
    #[command(visible_aliases = &["--to-utf8", "tu8", "2u8"])]
    ToUtf8 {
        /// The hex data to convert.
        hexdata: Option<String>,
    },

    /// Convert a fixed point number into an integer.
    #[command(visible_aliases = &["--from-fix", "ff"])]
    FromFixedPoint {
        /// The number of decimals to use.
        decimals: Option<String>,

        /// The value to convert.
        #[arg(allow_hyphen_values = true)]
        value: Option<String>,
    },

    /// Right-pads hex data to 32 bytes.
    #[command(visible_aliases = &["--to-bytes32", "tb", "2b"])]
    ToBytes32 {
        /// The hex data to convert.
        bytes: Option<String>,
    },

    /// Convert an integer into a fixed point number.
    #[command(visible_aliases = &["--to-fix", "tf", "2f"])]
    ToFixedPoint {
        /// The number of decimals to use.
        decimals: Option<String>,

        /// The value to convert.
        #[arg(allow_hyphen_values = true)]
        value: Option<String>,
    },

    /// Convert a number to a hex-encoded uint256.
    #[command(name = "to-uint256", visible_aliases = &["--to-uint256", "tu", "2u"])]
    ToUint256 {
        /// The value to convert.
        value: Option<String>,
    },

    /// Convert a number to a hex-encoded int256.
    #[command(name = "to-int256", visible_aliases = &["--to-int256", "ti", "2i"])]
    ToInt256 {
        /// The value to convert.
        value: Option<String>,
    },

    /// Perform a left shifting operation
    #[command(name = "shl")]
    LeftShift {
        /// The value to shift.
        value: String,

        /// The number of bits to shift.
        bits: String,

        /// The input base.
        #[arg(long)]
        base_in: Option<String>,

        /// The output base.
        #[arg(long, default_value = "16")]
        base_out: String,
    },

    /// Perform a right shifting operation
    #[command(name = "shr")]
    RightShift {
        /// The value to shift.
        value: String,

        /// The number of bits to shift.
        bits: String,

        /// The input base,
        #[arg(long)]
        base_in: Option<String>,

        /// The output base,
        #[arg(long, default_value = "16")]
        base_out: String,
    },

    /// Convert an ETH amount into another unit (ether, gwei or wei).
    ///
    /// Examples:
    /// - 1ether wei
    /// - "1 ether" wei
    /// - 1ether
    /// - 1 gwei
    /// - 1gwei ether
    #[command(visible_aliases = &["--to-unit", "tun", "2un"])]
    ToUnit {
        /// The value to convert.
        value: Option<String>,

        /// The unit to convert to (ether, gwei, wei).
        #[arg(default_value = "wei")]
        unit: String,
    },

    /// Convert an ETH amount to wei.
    ///
    /// Consider using --to-unit.
    #[command(visible_aliases = &["--to-wei", "tw", "2w"])]
    ToWei {
        /// The value to convert.
        #[arg(allow_hyphen_values = true)]
        value: Option<String>,

        /// The unit to convert from (ether, gwei, wei).
        #[arg(default_value = "eth")]
        unit: String,
    },

    /// Convert wei into an ETH amount.
    ///
    /// Consider using --to-unit.
    #[command(visible_aliases = &["--from-wei", "fw"])]
    FromWei {
        /// The value to convert.
        #[arg(allow_hyphen_values = true)]
        value: Option<String>,

        /// The unit to convert from (ether, gwei, wei).
        #[arg(default_value = "eth")]
        unit: String,
    },

    /// RLP encodes hex data, or an array of hex data.
    #[command(visible_aliases = &["--to-rlp"])]
    ToRlp {
        /// The value to convert.
        value: Option<String>,
    },

    /// Decodes RLP encoded data.
    ///
    /// Input must be hexadecimal.
    #[command(visible_aliases = &["--from-rlp"])]
    FromRlp {
        /// The value to convert.
        value: Option<String>,
    },

    /// Converts a number of one base to another
    #[command(visible_aliases = &["--to-hex", "th", "2h"])]
    ToHex(ToBaseArgs),

    /// Converts a number of one base to decimal
    #[command(visible_aliases = &["--to-dec", "td", "2d"])]
    ToDec(ToBaseArgs),

    /// Converts a number of one base to another
    #[command(
        visible_aliases = &["--to-base",
        "--to-radix",
        "to-radix",
        "tr",
        "2r"]
    )]
    ToBase {
        #[command(flatten)]
        base: ToBaseArgs,

        /// The output base.
        #[arg(value_name = "BASE")]
        base_out: Option<String>,
    },
    /// Create an access list for a transaction.
    #[command(visible_aliases = &["ac", "acl"])]
    AccessList(AccessListArgs),
    /// Get logs by signature or topic.
    #[command(visible_alias = "l")]
    Logs(LogsArgs),
    /// Get information about a block.
    #[command(visible_alias = "bl")]
    Block {
        /// The block height to query at.
        ///
        /// Can also be the tags earliest, finalized, safe, latest, or pending.
        block: Option<BlockId>,

        /// If specified, only get the given field of the block.
        #[arg(long, short)]
        field: Option<String>,

        #[arg(long, env = "CAST_FULL_BLOCK")]
        full: bool,

        /// Print the block as JSON.
        #[arg(long, short, help_heading = "Display options")]
        json: bool,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Get the latest block number.
    #[command(visible_alias = "bn")]
    BlockNumber {
        /// The hash or tag to query. If not specified, the latest number is returned.
        block: Option<BlockId>,
        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Perform a call on an account without publishing a transaction.
    #[command(visible_alias = "c")]
    Call(CallArgs),

    /// ABI-encode a function with arguments.
    #[command(name = "calldata", visible_alias = "cd")]
    CalldataEncode {
        /// The function signature in the format `<name>(<in-types>)(<out-types>)`
        sig: String,

        /// The arguments to encode.
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Get the symbolic name of the current chain.
    Chain {
        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Get the Ethereum chain ID.
    #[command(visible_aliases = &["ci", "cid"])]
    ChainId {
        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Get the current client version.
    #[command(visible_alias = "cl")]
    Client {
        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Compute the contract address from a given nonce and deployer address.
    #[command(visible_alias = "ca")]
    ComputeAddress {
        /// The deployer address.
        address: Option<String>,

        /// The nonce of the deployer address.
        #[arg(long)]
        nonce: Option<u64>,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Disassembles hex encoded bytecode into individual / human readable opcodes
    #[command(visible_alias = "da")]
    Disassemble {
        /// The hex encoded bytecode.
        bytecode: String,
    },

    /// Build and sign a transaction.
    #[command(name = "mktx", visible_alias = "m")]
    MakeTx(MakeTxArgs),

    /// Calculate the ENS namehash of a name.
    #[command(visible_aliases = &["na", "nh"])]
    Namehash { name: Option<String> },

    /// Get information about a transaction.
    #[command(visible_alias = "t")]
    Tx {
        /// The transaction hash.
        tx_hash: String,

        /// If specified, only get the given field of the transaction. If "raw", the RLP encoded
        /// transaction will be printed.
        field: Option<String>,

        /// Print the raw RLP encoded transaction.
        #[arg(long, conflicts_with = "field")]
        raw: bool,

        /// Print as JSON.
        #[arg(long, short, help_heading = "Display options")]
        json: bool,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Get the transaction receipt for a transaction.
    #[command(visible_alias = "re")]
    Receipt {
        /// The transaction hash.
        tx_hash: String,

        /// If specified, only get the given field of the transaction.
        field: Option<String>,

        /// The number of confirmations until the receipt is fetched
        #[arg(long, default_value = "1")]
        confirmations: u64,

        /// Exit immediately if the transaction was not found.
        #[arg(id = "async", long = "async", env = "CAST_ASYNC", alias = "cast-async")]
        cast_async: bool,

        /// Print as JSON.
        #[arg(long, short, help_heading = "Display options")]
        json: bool,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Sign and publish a transaction.
    #[command(name = "send", visible_alias = "s")]
    SendTx(SendTxArgs),

    /// Publish a raw transaction to the network.
    #[command(name = "publish", visible_alias = "p")]
    PublishTx {
        /// The raw transaction
        raw_tx: String,

        /// Only print the transaction hash and exit immediately.
        #[arg(id = "async", long = "async", env = "CAST_ASYNC", alias = "cast-async")]
        cast_async: bool,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Estimate the gas cost of a transaction.
    #[command(visible_alias = "e")]
    Estimate(EstimateArgs),

    /// Decode ABI-encoded input data.
    ///
    /// Similar to `abi-decode --input`, but function selector MUST be prefixed in `calldata`
    /// string
    #[command(visible_aliases = &["--calldata-decode","cdd"])]
    CalldataDecode {
        /// The function signature in the format `<name>(<in-types>)(<out-types>)`.
        sig: String,

        /// The ABI-encoded calldata.
        calldata: String,
    },

    /// Decode ABI-encoded input or output data.
    ///
    /// Defaults to decoding output data. To decode input data pass --input.
    ///
    /// When passing `--input`, function selector must NOT be prefixed in `calldata` string
    #[command(name = "abi-decode", visible_aliases = &["ad", "--abi-decode"])]
    AbiDecode {
        /// The function signature in the format `<name>(<in-types>)(<out-types>)`.
        sig: String,

        /// The ABI-encoded calldata.
        calldata: String,

        /// Whether to decode the input or output data.
        #[arg(long, short, help_heading = "Decode input data instead of output data")]
        input: bool,
    },

    /// ABI encode the given function argument, excluding the selector.
    #[command(visible_alias = "ae")]
    AbiEncode {
        /// The function signature.
        sig: String,

        /// Whether to use packed encoding.
        #[arg(long)]
        packed: bool,

        /// The arguments of the function.
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Compute the storage slot for an entry in a mapping.
    #[command(visible_alias = "in")]
    Index {
        /// The mapping key type.
        key_type: String,

        /// The mapping key.
        key: String,

        /// The storage slot of the mapping.
        slot_number: String,
    },

    /// Compute storage slots as specified by `ERC-7201: Namespaced Storage Layout`.
    #[command(name = "index-erc7201", alias = "index-erc-7201", visible_aliases = &["index7201", "in7201"])]
    IndexErc7201 {
        /// The arbitrary identifier.
        id: Option<String>,
        /// The formula ID. Currently the only supported formula is `erc7201`.
        #[arg(long, default_value = "erc7201")]
        formula_id: String,
    },

    /// Fetch the EIP-1967 implementation account
    #[command(visible_alias = "impl")]
    Implementation {
        /// The block height to query at.
        ///
        /// Can also be the tags earliest, finalized, safe, latest, or pending.
        #[arg(long, short = 'B')]
        block: Option<BlockId>,

        /// The address to get the nonce for.
        #[arg(value_parser = NameOrAddress::from_str)]
        who: NameOrAddress,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Fetch the EIP-1967 admin account
    #[command(visible_alias = "adm")]
    Admin {
        /// The block height to query at.
        ///
        /// Can also be the tags earliest, finalized, safe, latest, or pending.
        #[arg(long, short = 'B')]
        block: Option<BlockId>,

        /// The address to get the nonce for.
        #[arg(value_parser = NameOrAddress::from_str)]
        who: NameOrAddress,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Get the function signatures for the given selector from https://openchain.xyz.
    #[command(name = "4byte", visible_aliases = &["4", "4b"])]
    FourByte {
        /// The function selector.
        selector: Option<String>,
    },

    /// Decode ABI-encoded calldata using https://openchain.xyz.
    #[command(name = "4byte-decode", visible_aliases = &["4d", "4bd"])]
    FourByteDecode {
        /// The ABI-encoded calldata.
        calldata: Option<String>,
    },

    /// Get the event signature for a given topic 0 from https://openchain.xyz.
    #[command(name = "4byte-event", visible_aliases = &["4e", "4be", "topic0-event", "t0e"])]
    FourByteEvent {
        /// Topic 0
        #[arg(value_name = "TOPIC_0")]
        topic: Option<String>,
    },

    /// Upload the given signatures to https://openchain.xyz.
    ///
    /// Example inputs:
    /// - "transfer(address,uint256)"
    /// - "function transfer(address,uint256)"
    /// - "function transfer(address,uint256)" "event Transfer(address,address,uint256)"
    /// - "./out/Contract.sol/Contract.json"
    #[command(visible_aliases = &["ups"])]
    UploadSignature {
        /// The signatures to upload.
        ///
        /// Prefix with 'function', 'event', or 'error'. Defaults to function if no prefix given.
        /// Can also take paths to contract artifact JSON.
        signatures: Vec<String>,
    },

    /// Pretty print calldata.
    ///
    /// Tries to decode the calldata using https://openchain.xyz unless --offline is passed.
    #[command(visible_alias = "pc")]
    PrettyCalldata {
        /// The calldata.
        calldata: Option<String>,

        /// Skip the https://openchain.xyz lookup.
        #[arg(long, short)]
        offline: bool,
    },

    /// Get the timestamp of a block.
    #[command(visible_alias = "a")]
    Age {
        /// The block height to query at.
        ///
        /// Can also be the tags earliest, finalized, safe, latest, or pending.
        block: Option<BlockId>,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Get the balance of an account in wei.
    #[command(visible_alias = "b")]
    Balance {
        /// The block height to query at.
        ///
        /// Can also be the tags earliest, finalized, safe, latest, or pending.
        #[arg(long, short = 'B')]
        block: Option<BlockId>,

        /// The account to query.
        #[arg(value_parser = NameOrAddress::from_str)]
        who: NameOrAddress,

        /// Format the balance in ether.
        #[arg(long, short)]
        ether: bool,

        #[command(flatten)]
        rpc: RpcOpts,

        /// erc20 address to query, with the method `balanceOf(address) return (uint256)`, alias
        /// with '--erc721'
        #[arg(long, alias = "erc721")]
        erc20: Option<Address>,
    },

    /// Get the basefee of a block.
    #[command(visible_aliases = &["ba", "fee", "basefee"])]
    BaseFee {
        /// The block height to query at.
        ///
        /// Can also be the tags earliest, finalized, safe, latest, or pending.
        block: Option<BlockId>,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Get the runtime bytecode of a contract.
    #[command(visible_alias = "co")]
    Code {
        /// The block height to query at.
        ///
        /// Can also be the tags earliest, finalized, safe, latest, or pending.
        #[arg(long, short = 'B')]
        block: Option<BlockId>,

        /// The contract address.
        #[arg(value_parser = NameOrAddress::from_str)]
        who: NameOrAddress,

        /// Disassemble bytecodes into individual opcodes.
        #[arg(long, short)]
        disassemble: bool,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Get the runtime bytecode size of a contract.
    #[command(visible_alias = "cs")]
    Codesize {
        /// The block height to query at.
        ///
        /// Can also be the tags earliest, finalized, safe, latest, or pending.
        #[arg(long, short = 'B')]
        block: Option<BlockId>,

        /// The contract address.
        #[arg(value_parser = NameOrAddress::from_str)]
        who: NameOrAddress,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Get the current gas price.
    #[command(visible_alias = "g")]
    GasPrice {
        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Generate event signatures from event string.
    #[command(visible_alias = "se")]
    SigEvent {
        /// The event string.
        event_string: Option<String>,
    },

    /// Hash arbitrary data using Keccak-256.
    #[command(visible_aliases = &["k", "keccak256"])]
    Keccak {
        /// The data to hash.
        data: Option<String>,
    },

    /// Perform an ENS lookup.
    #[command(visible_alias = "rn")]
    ResolveName {
        /// The name to lookup.
        who: Option<String>,

        /// Perform a reverse lookup to verify that the name is correct.
        #[arg(long, short)]
        verify: bool,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Perform an ENS reverse lookup.
    #[command(visible_alias = "la")]
    LookupAddress {
        /// The account to perform the lookup for.
        who: Option<Address>,

        /// Perform a normal lookup to verify that the address is correct.
        #[arg(long, short)]
        verify: bool,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Get the raw value of a contract's storage slot.
    #[command(visible_alias = "st")]
    Storage(StorageArgs),

    /// Generate a storage proof for a given storage slot.
    #[command(visible_alias = "pr")]
    Proof {
        /// The contract address.
        #[arg(value_parser = NameOrAddress::from_str)]
        address: NameOrAddress,

        /// The storage slot numbers (hex or decimal).
        #[arg(value_parser = parse_slot)]
        slots: Vec<B256>,

        /// The block height to query at.
        ///
        /// Can also be the tags earliest, finalized, safe, latest, or pending.
        #[arg(long, short = 'B')]
        block: Option<BlockId>,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Get the nonce for an account.
    #[command(visible_alias = "n")]
    Nonce {
        /// The block height to query at.
        ///
        /// Can also be the tags earliest, finalized, safe, latest, or pending.
        #[arg(long, short = 'B')]
        block: Option<BlockId>,

        /// The address to get the nonce for.
        #[arg(value_parser = NameOrAddress::from_str)]
        who: NameOrAddress,

        #[command(flatten)]
        rpc: RpcOpts,
    },

    /// Get the source code of a contract from Etherscan.
    #[command(visible_aliases = &["et", "src"])]
    EtherscanSource {
        /// The contract's address.
        address: String,

        /// Whether to flatten the source code.
        #[arg(long, short)]
        flatten: bool,

        /// The output directory/file to expand source tree into.
        #[arg(short, value_hint = ValueHint::DirPath, alias = "path")]
        directory: Option<PathBuf>,

        #[command(flatten)]
        etherscan: EtherscanOpts,
    },

    /// Wallet management utilities.
    #[command(visible_alias = "w")]
    Wallet {
        #[command(subcommand)]
        command: WalletSubcommands,
    },

    /// Generate a Solidity interface from a given ABI.
    ///
    /// Currently does not support ABI encoder v2.
    #[command(visible_alias = "i")]
    Interface(InterfaceArgs),

    /// Generate a rust binding from a given ABI.
    #[command(visible_alias = "bi")]
    Bind(BindArgs),

    /// Get the selector for a function.
    #[command(visible_alias = "si")]
    Sig {
        /// The function signature, e.g. transfer(address,uint256).
        sig: Option<String>,

        /// Optimize signature to contain provided amount of leading zeroes in selector.
        optimize: Option<usize>,
    },

    /// Generate a deterministic contract address using CREATE2.
    #[command(visible_alias = "c2")]
    Create2(Create2Args),

    /// Get the block number closest to the provided timestamp.
    #[command(visible_alias = "f")]
    FindBlock(FindBlockArgs),

    /// Generate shell completions script.
    #[command(visible_alias = "com")]
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// Generate Fig autocompletion spec.
    #[command(visible_alias = "fig")]
    GenerateFigSpec,

    /// Runs a published transaction in a local environment and prints the trace.
    #[command(visible_alias = "r")]
    Run(RunArgs),

    /// Perform a raw JSON-RPC request.
    #[command(visible_alias = "rp")]
    Rpc(RpcArgs),

    /// Formats a string into bytes32 encoding.
    #[command(name = "format-bytes32-string", visible_aliases = &["--format-bytes32-string"])]
    FormatBytes32String {
        /// The string to format.
        string: Option<String>,
    },

    /// Parses a string from bytes32 encoding.
    #[command(name = "parse-bytes32-string", visible_aliases = &["--parse-bytes32-string"])]
    ParseBytes32String {
        /// The string to parse.
        bytes: Option<String>,
    },
    #[command(name = "parse-bytes32-address", visible_aliases = &["--parse-bytes32-address"])]
    #[command(about = "Parses a checksummed address from bytes32 encoding.")]
    ParseBytes32Address {
        #[arg(value_name = "BYTES")]
        bytes: Option<String>,
    },

    /// Decodes a raw signed EIP 2718 typed transaction
    #[command(visible_alias = "dt")]
    DecodeTransaction { tx: Option<String> },

    /// Extracts function selectors and arguments from bytecode
    #[command(visible_alias = "sel")]
    Selectors {
        /// The hex encoded bytecode.
        bytecode: String,

        /// Resolve the function signatures for the extracted selectors using https://openchain.xyz
        #[arg(long, short)]
        resolve: bool,
    },
}

impl CastSubcommand {
    pub async fn run(self) -> eyre::Result<()> {
        match self {
            // Constants
            Self::MaxInt { r#type } => {
                println!("{}", SimpleCast::max_int(&r#type)?);
            }
            Self::MinInt { r#type } => {
                println!("{}", SimpleCast::min_int(&r#type)?);
            }
            Self::MaxUint { r#type } => {
                println!("{}", SimpleCast::max_int(&r#type)?);
            }
            Self::AddressZero => {
                println!("{:?}", Address::ZERO);
            }
            Self::HashZero => {
                println!("{:?}", B256::ZERO);
            }

            // Conversions & transformations
            Self::FromUtf8 { text } => {
                let value = stdin::unwrap(text, false)?;
                println!("{}", SimpleCast::from_utf8(&value));
            }
            Self::ToAscii { hexdata } => {
                let value = stdin::unwrap(hexdata, false)?;
                println!("{}", SimpleCast::to_ascii(&value)?);
            }
            Self::ToUtf8 { hexdata } => {
                let value = stdin::unwrap(hexdata, false)?;
                println!("{}", SimpleCast::to_utf8(&value)?);
            }
            Self::FromFixedPoint { value, decimals } => {
                let (value, decimals) = stdin::unwrap2(value, decimals)?;
                println!("{}", SimpleCast::from_fixed_point(&value, &decimals)?);
            }
            Self::ToFixedPoint { value, decimals } => {
                let (value, decimals) = stdin::unwrap2(value, decimals)?;
                println!("{}", SimpleCast::to_fixed_point(&value, &decimals)?);
            }
            Self::ConcatHex { data } => {
                if data.is_empty() {
                    let s = stdin::read(true)?;
                    println!("{}", SimpleCast::concat_hex(s.split_whitespace()))
                } else {
                    println!("{}", SimpleCast::concat_hex(data))
                }
            }
            Self::FromBin => {
                let hex = stdin::read_bytes(false)?;
                println!("{}", hex::encode_prefixed(hex));
            }
            Self::ToHexdata { input } => {
                let value = stdin::unwrap_line(input)?;
                let output = match value {
                    s if s.starts_with('@') => hex::encode(std::env::var(&s[1..])?),
                    s if s.starts_with('/') => hex::encode(fs::read(s)?),
                    s => s.split(':').map(|s| s.trim_start_matches("0x").to_lowercase()).collect(),
                };
                println!("0x{output}");
            }
            Self::ToCheckSumAddress { address } => {
                let value = stdin::unwrap_line(address)?;
                println!("{}", value.to_checksum(None));
            }
            Self::ToUint256 { value } => {
                let value = stdin::unwrap_line(value)?;
                println!("{}", SimpleCast::to_uint256(&value)?);
            }
            Self::ToInt256 { value } => {
                let value = stdin::unwrap_line(value)?;
                println!("{}", SimpleCast::to_int256(&value)?);
            }
            Self::ToUnit { value, unit } => {
                let value = stdin::unwrap_line(value)?;
                println!("{}", SimpleCast::to_unit(&value, &unit)?);
            }
            Self::FromWei { value, unit } => {
                let value = stdin::unwrap_line(value)?;
                println!("{}", SimpleCast::from_wei(&value, &unit)?);
            }
            Self::ToWei { value, unit } => {
                let value = stdin::unwrap_line(value)?;
                println!("{}", SimpleCast::to_wei(&value, &unit)?);
            }
            Self::FromRlp { value } => {
                let value = stdin::unwrap_line(value)?;
                println!("{}", SimpleCast::from_rlp(value)?);
            }
            Self::ToRlp { value } => {
                let value = stdin::unwrap_line(value)?;
                println!("{}", SimpleCast::to_rlp(&value)?);
            }
            Self::ToHex(ToBaseArgs { value, base_in }) => {
                let value = stdin::unwrap_line(value)?;
                println!("{}", SimpleCast::to_base(&value, base_in.as_deref(), "hex")?);
            }
            Self::ToDec(ToBaseArgs { value, base_in }) => {
                let value = stdin::unwrap_line(value)?;
                println!("{}", SimpleCast::to_base(&value, base_in.as_deref(), "dec")?);
            }
            Self::ToBase { base: ToBaseArgs { value, base_in }, base_out } => {
                let (value, base_out) = stdin::unwrap2(value, base_out)?;
                println!("{}", SimpleCast::to_base(&value, base_in.as_deref(), &base_out)?);
            }
            Self::ToBytes32 { bytes } => {
                let value = stdin::unwrap_line(bytes)?;
                println!("{}", SimpleCast::to_bytes32(&value)?);
            }
            Self::FormatBytes32String { string } => {
                let value = stdin::unwrap_line(string)?;
                println!("{}", SimpleCast::format_bytes32_string(&value)?);
            }
            Self::ParseBytes32String { bytes } => {
                let value = stdin::unwrap_line(bytes)?;
                println!("{}", SimpleCast::parse_bytes32_string(&value)?);
            }
            Self::ParseBytes32Address { bytes } => {
                let value = stdin::unwrap_line(bytes)?;
                println!("{}", SimpleCast::parse_bytes32_address(&value)?);
            }

            // ABI encoding & decoding
            Self::AbiDecode { sig, calldata, input } => {
                let tokens = SimpleCast::abi_decode(&sig, &calldata, input)?;
                let tokens = format_tokens(&tokens);
                tokens.for_each(|t| println!("{t}"));
            }
            Self::AbiEncode { sig, packed, args } => {
                if !packed {
                    println!("{}", SimpleCast::abi_encode(&sig, &args)?);
                } else {
                    println!("{}", SimpleCast::abi_encode_packed(&sig, &args)?);
                }
            }
            Self::CalldataDecode { sig, calldata } => {
                let tokens = SimpleCast::calldata_decode(&sig, &calldata, true)?;
                let tokens = format_tokens(&tokens);
                tokens.for_each(|t| println!("{t}"));
            }
            Self::CalldataEncode { sig, args } => {
                println!("{}", SimpleCast::calldata_encode(sig, &args)?);
            }
            Self::Interface(cmd) => cmd.run().await?,
            Self::Bind(cmd) => cmd.run().await?,
            Self::PrettyCalldata { calldata, offline } => {
                let calldata = stdin::unwrap_line(calldata)?;
                println!("{}", pretty_calldata(&calldata, offline).await?);
            }
            Self::Sig { sig, optimize } => {
                let sig = stdin::unwrap_line(sig)?;
                match optimize {
                    Some(opt) => {
                        println!("Starting to optimize signature...");
                        let start_time = Instant::now();
                        let (selector, signature) = SimpleCast::get_selector(&sig, opt)?;
                        println!("Successfully generated in {:?}", start_time.elapsed());
                        println!("Selector: {selector}");
                        println!("Optimized signature: {signature}");
                    }
                    None => println!("{}", SimpleCast::get_selector(&sig, 0)?.0),
                }
            }

            // Blockchain & RPC queries
            Self::AccessList(cmd) => cmd.run().await?,
            Self::Age { block, rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                println!(
                    "{}",
                    Cast::new(provider).age(block.unwrap_or(BlockId::Number(Latest))).await?
                );
            }
            Self::Balance { block, who, ether, rpc, erc20 } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                let account_addr = who.resolve(&provider).await?;

                match erc20 {
                    Some(token) => {
                        let balance =
                            Cast::new(&provider).erc20_balance(token, account_addr, block).await?;
                        println!("{}", format_uint_exp(balance));
                    }
                    None => {
                        let value = Cast::new(&provider).balance(account_addr, block).await?;
                        if ether {
                            println!("{}", SimpleCast::from_wei(&value.to_string(), "eth")?);
                        } else {
                            println!("{value}");
                        }
                    }
                }
            }
            Self::BaseFee { block, rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                println!(
                    "{}",
                    Cast::new(provider).base_fee(block.unwrap_or(BlockId::Number(Latest))).await?
                );
            }
            Self::Block { block, full, field, json, rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                println!(
                    "{}",
                    Cast::new(provider)
                        .block(block.unwrap_or(BlockId::Number(Latest)), full, field, json)
                        .await?
                );
            }
            Self::BlockNumber { rpc, block } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                let number = match block {
                    Some(id) => provider
                        .get_block(id, false.into())
                        .await?
                        .ok_or_else(|| eyre::eyre!("block {id:?} not found"))?
                        .header
                        .number
                        .ok_or_else(|| eyre::eyre!("block {id:?} has no block number"))?,
                    None => Cast::new(provider).block_number().await?,
                };
                println!("{number}");
            }
            Self::Chain { rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                println!("{}", Cast::new(provider).chain().await?);
            }
            Self::ChainId { rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                println!("{}", Cast::new(provider).chain_id().await?);
            }
            Self::Client { rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                println!("{}", provider.get_client_version().await?);
            }
            Self::Code { block, who, disassemble, rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                let who = who.resolve(&provider).await?;
                println!("{}", Cast::new(provider).code(who, block, disassemble).await?);
            }
            Self::Codesize { block, who, rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                let who = who.resolve(&provider).await?;
                println!("{}", Cast::new(provider).codesize(who, block).await?);
            }
            Self::ComputeAddress { address, nonce, rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;

                let address: Address = stdin::unwrap_line(address)?.parse()?;
                let computed = Cast::new(provider).compute_address(address, nonce).await?;
                println!("Computed Address: {}", computed.to_checksum(None));
            }
            Self::Disassemble { bytecode } => {
                println!("{}", SimpleCast::disassemble(&bytecode)?);
            }
            Self::Selectors { bytecode, resolve } => {
                let selectors_and_args = SimpleCast::extract_selectors(&bytecode)?;
                if resolve {
                    let selectors_it = selectors_and_args.iter().map(|r| &r.0);
                    let resolve_results =
                        decode_selectors(SelectorType::Function, selectors_it).await?;

                    let max_args_len =
                        selectors_and_args.iter().map(|r| r.1.len()).max().unwrap_or(0);
                    for ((selector, arguments), func_names) in
                        selectors_and_args.into_iter().zip(resolve_results.into_iter())
                    {
                        let resolved = match func_names {
                            Some(v) => v.join("|"),
                            None => String::new(),
                        };
                        println!("{selector}\t{arguments:max_args_len$}\t{resolved}");
                    }
                } else {
                    for (selector, arguments) in selectors_and_args {
                        println!("{selector}\t{arguments}");
                    }
                }
            }
            Self::FindBlock(cmd) => cmd.run().await?,
            Self::GasPrice { rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                println!("{}", Cast::new(provider).gas_price().await?);
            }
            Self::Index { key_type, key, slot_number } => {
                println!("{}", SimpleCast::index(&key_type, &key, &slot_number)?);
            }
            Self::IndexErc7201 { id, formula_id } => {
                eyre::ensure!(formula_id == "erc7201", "unsupported formula ID: {formula_id}");
                let id = stdin::unwrap_line(id)?;
                println!("{}", foundry_common::erc7201(&id));
            }
            Self::Implementation { block, who, rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                let who = who.resolve(&provider).await?;
                println!("{}", Cast::new(provider).implementation(who, block).await?);
            }
            Self::Admin { block, who, rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                let who = who.resolve(&provider).await?;
                println!("{}", Cast::new(provider).admin(who, block).await?);
            }
            Self::Nonce { block, who, rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                let who = who.resolve(&provider).await?;
                println!("{}", Cast::new(provider).nonce(who, block).await?);
            }
            Self::Proof { address, slots, rpc, block } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                let address = address.resolve(&provider).await?;
                let value = provider
                    .get_proof(address, slots.into_iter().collect())
                    .block_id(block.unwrap_or_default())
                    .await?;
                println!("{}", serde_json::to_string(&value)?);
            }
            Self::Rpc(cmd) => cmd.run().await?,
            Self::Storage(cmd) => cmd.run().await?,

            // Calls & transactions
            Self::Call(cmd) => cmd.run().await?,
            Self::Estimate(cmd) => cmd.run().await?,
            Self::MakeTx(cmd) => cmd.run().await?,
            Self::PublishTx { raw_tx, cast_async, rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                let cast = Cast::new(&provider);
                let pending_tx = cast.publish(raw_tx).await?;
                let tx_hash = pending_tx.inner().tx_hash();

                if cast_async {
                    println!("{tx_hash:#x}");
                } else {
                    let receipt = pending_tx.get_receipt().await?;
                    println!("{}", serde_json::json!(receipt));
                }
            }
            Self::Receipt { tx_hash, field, json, cast_async, confirmations, rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;
                println!(
                    "{}",
                    Cast::new(provider)
                        .receipt(tx_hash, field, confirmations, cast_async, json)
                        .await?
                );
            }
            Self::Run(cmd) => cmd.run().await?,
            Self::SendTx(cmd) => cmd.run().await?,
            Self::Tx { tx_hash, field, raw, json, rpc } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;

                // Can use either --raw or specify raw as a field
                let raw = raw || field.as_ref().is_some_and(|f| f == "raw");

                println!("{}", Cast::new(&provider).transaction(tx_hash, field, raw, json).await?)
            }

            // 4Byte
            Self::FourByte { selector } => {
                let selector = stdin::unwrap_line(selector)?;
                let sigs = decode_function_selector(&selector).await?;
                if sigs.is_empty() {
                    eyre::bail!("No matching function signatures found for selector `{selector}`");
                }
                for sig in sigs {
                    println!("{sig}");
                }
            }
            Self::FourByteDecode { calldata } => {
                let calldata = stdin::unwrap_line(calldata)?;
                let sigs = decode_calldata(&calldata).await?;
                sigs.iter().enumerate().for_each(|(i, sig)| println!("{}) \"{sig}\"", i + 1));

                let sig = match sigs.len() {
                    0 => eyre::bail!("No signatures found"),
                    1 => sigs.first().unwrap(),
                    _ => {
                        let i: usize = prompt!("Select a function signature by number: ")?;
                        sigs.get(i - 1).ok_or_else(|| eyre::eyre!("Invalid signature index"))?
                    }
                };

                let tokens = SimpleCast::calldata_decode(sig, &calldata, true)?;
                for token in format_tokens(&tokens) {
                    println!("{token}");
                }
            }
            Self::FourByteEvent { topic } => {
                let topic = stdin::unwrap_line(topic)?;
                let sigs = decode_event_topic(&topic).await?;
                if sigs.is_empty() {
                    eyre::bail!("No matching event signatures found for topic `{topic}`");
                }
                for sig in sigs {
                    println!("{sig}");
                }
            }
            Self::UploadSignature { signatures } => {
                let signatures = stdin::unwrap_vec(signatures)?;
                let ParsedSignatures { signatures, abis } = parse_signatures(signatures);
                if !abis.is_empty() {
                    import_selectors(SelectorImportData::Abi(abis)).await?.describe();
                }
                if !signatures.is_empty() {
                    import_selectors(SelectorImportData::Raw(signatures)).await?.describe();
                }
            }

            // ENS
            Self::Namehash { name } => {
                let name = stdin::unwrap_line(name)?;
                println!("{}", namehash(&name));
            }
            Self::LookupAddress { who, rpc, verify } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;

                let who = stdin::unwrap_line(who)?;
                let name = provider.lookup_address(&who).await?;
                if verify {
                    let address = provider.resolve_name(&name).await?;
                    eyre::ensure!(
                        address == who,
                        "Reverse lookup verification failed: got `{address}`, expected `{who}`"
                    );
                }
                println!("{name}");
            }
            Self::ResolveName { who, rpc, verify } => {
                let config = Config::from(&rpc);
                let provider = utils::get_provider(&config)?;

                let who = stdin::unwrap_line(who)?;
                let address = provider.resolve_name(&who).await?;
                if verify {
                    let name = provider.lookup_address(&address).await?;
                    eyre::ensure!(
                        name == who,
                        "Forward lookup verification failed: got `{name}`, expected `{who}`"
                    );
                }
                println!("{address}");
            }

            // Misc
            Self::Keccak { data } => {
                let bytes = match data {
                    Some(data) => data.into_bytes(),
                    None => stdin::read_bytes(false)?,
                };
                match String::from_utf8(bytes) {
                    Ok(s) => {
                        let s = SimpleCast::keccak(&s)?;
                        println!("{s}");
                    }
                    Err(e) => {
                        let hash = keccak256(e.as_bytes());
                        let s = hex::encode(hash);
                        println!("0x{s}");
                    }
                };
            }
            Self::SigEvent { event_string } => {
                let event_string = stdin::unwrap_line(event_string)?;
                let parsed_event = get_event(&event_string)?;
                println!("{:?}", parsed_event.selector());
            }
            Self::LeftShift { value, bits, base_in, base_out } => {
                println!(
                    "{}",
                    SimpleCast::left_shift(&value, &bits, base_in.as_deref(), &base_out)?
                );
            }
            Self::RightShift { value, bits, base_in, base_out } => {
                println!(
                    "{}",
                    SimpleCast::right_shift(&value, &bits, base_in.as_deref(), &base_out)?
                );
            }
            Self::EtherscanSource { address, directory, etherscan, flatten } => {
                let config = Config::from(&etherscan);
                let chain = config.chain.unwrap_or_default();
                let api_key = config.get_etherscan_api_key(Some(chain)).unwrap_or_default();
                match (directory, flatten) {
                    (Some(dir), false) => {
                        SimpleCast::expand_etherscan_source_to_directory(
                            chain, address, api_key, dir,
                        )
                        .await?
                    }
                    (None, false) => {
                        println!(
                            "{}",
                            SimpleCast::etherscan_source(chain, address, api_key).await?
                        );
                    }
                    (dir, true) => {
                        SimpleCast::etherscan_source_flatten(chain, address, api_key, dir).await?;
                    }
                }
            }
            Self::Create2(cmd) => {
                cmd.run()?;
            }
            Self::Wallet { command } => command.run().await?,
            Self::Completions { shell } => {
                generate(shell, &mut CastArgs::command(), "cast", &mut std::io::stdout())
            }
            Self::GenerateFigSpec => clap_complete::generate(
                clap_complete_fig::Fig,
                &mut CastArgs::command(),
                "cast",
                &mut std::io::stdout(),
            ),
            Self::Logs(cmd) => cmd.run().await?,
            Self::DecodeTransaction { tx } => {
                let tx = stdin::unwrap_line(tx)?;
                let tx = SimpleCast::decode_raw_transaction(&tx)?;

                println!("{}", serde_json::to_string_pretty(&tx)?);
            }
        };
        Ok(())
    }
}

/// CLI arguments for `cast --to-base`.
#[derive(Debug, Parser)]
pub struct ToBaseArgs {
    /// The value to convert.
    #[arg(allow_hyphen_values = true)]
    pub value: Option<String>,

    /// The input base.
    #[arg(long, short = 'i')]
    pub base_in: Option<String>,
}

pub fn parse_slot(s: &str) -> Result<B256> {
    let slot = U256::from_str(s).map_err(|e| eyre::eyre!("Could not parse slot number: {e}"))?;
    Ok(B256::from(slot))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimpleCast;
    use alloy_rpc_types::{BlockNumberOrTag, RpcBlockHash};
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        CastArgs::command().debug_assert();
    }

    #[test]
    fn parse_proof_slot() {
        let args: CastArgs = CastArgs::parse_from([
            "foundry-cli",
            "proof",
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            "0",
            "1",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x1",
            "0x01",
        ]);
        match args.cmd {
            CastSubcommand::Proof { slots, .. } => {
                assert_eq!(
                    slots,
                    vec![
                        B256::ZERO,
                        U256::from(1).into(),
                        B256::ZERO,
                        U256::from(1).into(),
                        U256::from(1).into()
                    ]
                );
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn parse_call_data() {
        let args: CastArgs = CastArgs::parse_from([
            "foundry-cli",
            "calldata",
            "f()",
            "5c9d55b78febcc2061715ba4f57ecf8ea2711f2c",
            "2",
        ]);
        match args.cmd {
            CastSubcommand::CalldataEncode { args, .. } => {
                assert_eq!(
                    args,
                    vec!["5c9d55b78febcc2061715ba4f57ecf8ea2711f2c".to_string(), "2".to_string()]
                )
            }
            _ => unreachable!(),
        };
    }

    // <https://github.com/foundry-rs/book/issues/1019>
    #[test]
    fn parse_signature() {
        let args: CastArgs = CastArgs::parse_from([
            "foundry-cli",
            "sig",
            "__$_$__$$$$$__$$_$$$_$$__$$___$$(address,address,uint256)",
        ]);
        match args.cmd {
            CastSubcommand::Sig { sig, .. } => {
                let sig = sig.unwrap();
                assert_eq!(
                    sig,
                    "__$_$__$$$$$__$$_$$$_$$__$$___$$(address,address,uint256)".to_string()
                );

                let selector = SimpleCast::get_selector(&sig, 0).unwrap();
                assert_eq!(selector.0, "0x23b872dd".to_string());
            }
            _ => unreachable!(),
        };
    }

    #[test]
    fn parse_block_ids() {
        struct TestCase {
            input: String,
            expect: BlockId,
        }

        let test_cases = [
            TestCase {
                input: "0".to_string(),
                expect: BlockId::Number(BlockNumberOrTag::Number(0u64)),
            },
            TestCase {
                input: "0x56462c47c03df160f66819f0a79ea07def1569f8aac0fe91bb3a081159b61b4a"
                    .to_string(),
                expect: BlockId::Hash(RpcBlockHash::from_hash(
                    "0x56462c47c03df160f66819f0a79ea07def1569f8aac0fe91bb3a081159b61b4a"
                        .parse()
                        .unwrap(),
                    None,
                )),
            },
            TestCase {
                input: "latest".to_string(),
                expect: BlockId::Number(BlockNumberOrTag::Latest),
            },
            TestCase {
                input: "earliest".to_string(),
                expect: BlockId::Number(BlockNumberOrTag::Earliest),
            },
            TestCase {
                input: "pending".to_string(),
                expect: BlockId::Number(BlockNumberOrTag::Pending),
            },
            TestCase { input: "safe".to_string(), expect: BlockId::Number(BlockNumberOrTag::Safe) },
            TestCase {
                input: "finalized".to_string(),
                expect: BlockId::Number(BlockNumberOrTag::Finalized),
            },
        ];

        for test in test_cases {
            let result: BlockId = test.input.parse().unwrap();
            assert_eq!(result, test.expect);
        }
    }
}
