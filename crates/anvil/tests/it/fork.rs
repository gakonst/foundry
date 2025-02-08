//! various fork related test

use crate::{
    abi::{Greeter, ERC721},
    utils::{http_provider, http_provider_with_signer},
};
use alloy_chains::NamedChain;
use alloy_network::{EthereumWallet, ReceiptResponse, TransactionBuilder, TransactionResponse};
use alloy_primitives::{address, b256, bytes, uint, Address, Bytes, TxHash, TxKind, U256, U64};
use alloy_provider::Provider;
use alloy_rpc_types::{
    anvil::Forking,
    request::{TransactionInput, TransactionRequest},
    BlockId, BlockNumberOrTag, BlockTransactionsKind,
};
use alloy_serde::WithOtherFields;
use alloy_signer_local::PrivateKeySigner;
use anvil::{eth::EthApi, spawn, NodeConfig, NodeHandle};
use foundry_common::provider::get_http_provider;
use foundry_config::Config;
use foundry_test_utils::rpc::{self, next_http_rpc_endpoint, next_rpc_endpoint};
use futures::StreamExt;
use std::{sync::Arc, thread::sleep, time::Duration};

const BLOCK_NUMBER: u64 = 14_608_400u64;
const DEAD_BALANCE_AT_BLOCK_NUMBER: u128 = 12_556_069_338_441_120_059_867u128;

const BLOCK_TIMESTAMP: u64 = 1_650_274_250u64;

/// Represents an anvil fork of an anvil node
#[allow(unused)]
pub struct LocalFork {
    origin_api: EthApi,
    origin_handle: NodeHandle,
    fork_api: EthApi,
    fork_handle: NodeHandle,
}

#[allow(dead_code)]
impl LocalFork {
    /// Spawns two nodes with the test config
    pub async fn new() -> Self {
        Self::setup(NodeConfig::test(), NodeConfig::test()).await
    }

    /// Spawns two nodes where one is a fork of the other
    pub async fn setup(origin: NodeConfig, fork: NodeConfig) -> Self {
        let (origin_api, origin_handle) = spawn(origin).await;

        let (fork_api, fork_handle) =
            spawn(fork.with_eth_rpc_url(Some(origin_handle.http_endpoint()))).await;
        Self { origin_api, origin_handle, fork_api, fork_handle }
    }
}

pub fn fork_config() -> NodeConfig {
    NodeConfig::test()
        .with_eth_rpc_url(Some(rpc::next_http_archive_rpc_url()))
        .with_fork_block_number(Some(BLOCK_NUMBER))
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_gas_limit_applied_from_config() {
    let (api, _handle) = spawn(fork_config().with_gas_limit(Some(10_000_000_u128))).await;

    assert_eq!(api.gas_limit(), uint!(10_000_000_U256));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_gas_limit_disabled_from_config() {
    let (api, handle) = spawn(fork_config().disable_block_gas_limit(true)).await;

    // see https://github.com/foundry-rs/foundry/pull/8933
    assert_eq!(api.gas_limit(), U256::from(U64::MAX));

    // try to mine a couple blocks
    let provider = handle.http_provider();
    let tx = TransactionRequest::default()
        .to(Address::random())
        .value(U256::from(1337u64))
        .from(handle.dev_wallets().next().unwrap().address());
    let tx = WithOtherFields::new(tx);
    let _ = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();

    let tx = TransactionRequest::default()
        .to(Address::random())
        .value(U256::from(1337u64))
        .from(handle.dev_wallets().next().unwrap().address());
    let tx = WithOtherFields::new(tx);
    let _ = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_spawn_fork() {
    let (api, _handle) = spawn(fork_config()).await;
    assert!(api.is_fork());

    let head = api.block_number().unwrap();
    assert_eq!(head, U256::from(BLOCK_NUMBER))
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_eth_get_balance() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();
    for _ in 0..10 {
        let addr = Address::random();
        let balance = api.balance(addr, None).await.unwrap();
        let provider_balance = provider.get_balance(addr).await.unwrap();
        assert_eq!(balance, provider_balance)
    }
}

// <https://github.com/foundry-rs/foundry/issues/4082>
#[tokio::test(flavor = "multi_thread")]
async fn test_fork_eth_get_balance_after_mine() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();
    let info = api.anvil_node_info().await.unwrap();
    let number = info.fork_config.fork_block_number.unwrap();
    assert_eq!(number, BLOCK_NUMBER);

    let address = Address::random();

    let _balance = provider.get_balance(address).await.unwrap();

    api.evm_mine(None).await.unwrap();

    let _balance = provider.get_balance(address).await.unwrap();
}

// <https://github.com/foundry-rs/foundry/issues/4082>
#[tokio::test(flavor = "multi_thread")]
async fn test_fork_eth_get_code_after_mine() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();
    let info = api.anvil_node_info().await.unwrap();
    let number = info.fork_config.fork_block_number.unwrap();
    assert_eq!(number, BLOCK_NUMBER);

    let address = Address::random();

    let _code = provider.get_code_at(address).block_id(BlockId::number(1)).await.unwrap();

    api.evm_mine(None).await.unwrap();

    let _code = provider.get_code_at(address).block_id(BlockId::number(1)).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_eth_get_code() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();
    for _ in 0..10 {
        let addr = Address::random();
        let code = api.get_code(addr, None).await.unwrap();
        let provider_code = provider.get_code_at(addr).await.unwrap();
        assert_eq!(code, provider_code)
    }

    let addresses: Vec<Address> = vec![
        "0x6b175474e89094c44da98b954eedeac495271d0f".parse().unwrap(),
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".parse().unwrap(),
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse().unwrap(),
        "0x1F98431c8aD98523631AE4a59f267346ea31F984".parse().unwrap(),
        "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45".parse().unwrap(),
    ];
    for address in addresses {
        let prev_code = api
            .get_code(address, Some(BlockNumberOrTag::Number(BLOCK_NUMBER - 10).into()))
            .await
            .unwrap();
        let code = api.get_code(address, None).await.unwrap();
        let provider_code = provider.get_code_at(address).await.unwrap();
        assert_eq!(code, prev_code);
        assert_eq!(code, provider_code);
        assert!(!code.as_ref().is_empty());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_eth_get_nonce() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();

    for _ in 0..10 {
        let addr = Address::random();
        let api_nonce = api.transaction_count(addr, None).await.unwrap().to::<u64>();
        let provider_nonce = provider.get_transaction_count(addr).await.unwrap();
        assert_eq!(api_nonce, provider_nonce);
    }

    let addr = Config::DEFAULT_SENDER;
    let api_nonce = api.transaction_count(addr, None).await.unwrap().to::<u64>();
    let provider_nonce = provider.get_transaction_count(addr).await.unwrap();
    assert_eq!(api_nonce, provider_nonce);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_optimism_with_transaction_hash() {
    use std::str::FromStr;

    // Fork to a block with a specific transaction
    let fork_tx_hash =
        TxHash::from_str("fcb864b5a50f0f0b111dbbf9e9167b2cb6179dfd6270e1ad53aac6049c0ec038")
            .unwrap();
    let (api, _handle) = spawn(
        NodeConfig::test()
            .with_eth_rpc_url(Some(rpc::next_rpc_endpoint(NamedChain::Optimism)))
            .with_fork_transaction_hash(Some(fork_tx_hash)),
    )
    .await;

    // Make sure the fork starts from previous block
    let block_number = api.block_number().unwrap().to::<u64>();
    assert_eq!(block_number, 125777954 - 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_eth_fee_history() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();

    let count = 10u64;
    let _history =
        api.fee_history(U256::from(count), BlockNumberOrTag::Latest, vec![]).await.unwrap();
    let _provider_history =
        provider.get_fee_history(count, BlockNumberOrTag::Latest, &[]).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_reset() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();

    let accounts: Vec<_> = handle.dev_wallets().collect();
    let from = accounts[0].address();
    let to = accounts[1].address();
    let block_number = provider.get_block_number().await.unwrap();
    let balance_before = provider.get_balance(to).await.unwrap();
    let amount = handle.genesis_balance().checked_div(U256::from(2u64)).unwrap();

    let initial_nonce = provider.get_transaction_count(from).await.unwrap();

    let tx = TransactionRequest::default().to(to).value(amount).from(from);
    let tx = WithOtherFields::new(tx);
    let tx = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();
    assert_eq!(tx.transaction_index, Some(0));

    let nonce = provider.get_transaction_count(from).await.unwrap();

    assert_eq!(nonce, initial_nonce + 1);
    let to_balance = provider.get_balance(to).await.unwrap();
    assert_eq!(balance_before.saturating_add(amount), to_balance);
    api.anvil_reset(Some(Forking { json_rpc_url: None, block_number: Some(block_number) }))
        .await
        .unwrap();

    // reset block number
    assert_eq!(block_number, provider.get_block_number().await.unwrap());

    let nonce = provider.get_transaction_count(from).await.unwrap();
    assert_eq!(nonce, initial_nonce);
    let balance = provider.get_balance(from).await.unwrap();
    assert_eq!(balance, handle.genesis_balance());
    let balance = provider.get_balance(to).await.unwrap();
    assert_eq!(balance, handle.genesis_balance());

    // reset to latest
    api.anvil_reset(Some(Forking::default())).await.unwrap();

    let new_block_num = provider.get_block_number().await.unwrap();
    assert!(new_block_num > block_number);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_reset_setup() {
    let (api, handle) = spawn(NodeConfig::test()).await;
    let provider = handle.http_provider();

    let dead_addr: Address = "000000000000000000000000000000000000dEaD".parse().unwrap();

    let block_number = provider.get_block_number().await.unwrap();
    assert_eq!(block_number, 0);

    let local_balance = provider.get_balance(dead_addr).await.unwrap();
    assert_eq!(local_balance, U256::ZERO);

    api.anvil_reset(Some(Forking {
        json_rpc_url: Some(rpc::next_http_archive_rpc_url()),
        block_number: Some(BLOCK_NUMBER),
    }))
    .await
    .unwrap();

    let block_number = provider.get_block_number().await.unwrap();
    assert_eq!(block_number, BLOCK_NUMBER);

    let remote_balance = provider.get_balance(dead_addr).await.unwrap();
    assert_eq!(remote_balance, U256::from(DEAD_BALANCE_AT_BLOCK_NUMBER));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_state_snapshotting() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();
    let state_snapshot = api.evm_snapshot().await.unwrap();

    let accounts: Vec<_> = handle.dev_wallets().collect();
    let from = accounts[0].address();
    let to = accounts[1].address();
    let block_number = provider.get_block_number().await.unwrap();

    let initial_nonce = provider.get_transaction_count(from).await.unwrap();
    let balance_before = provider.get_balance(to).await.unwrap();
    let amount = handle.genesis_balance().checked_div(U256::from(2u64)).unwrap();

    let provider = handle.http_provider();
    let tx = TransactionRequest::default().to(to).value(amount).from(from);
    let tx = WithOtherFields::new(tx);

    let _ = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();

    let provider = handle.http_provider();

    let nonce = provider.get_transaction_count(from).await.unwrap();
    assert_eq!(nonce, initial_nonce + 1);
    let to_balance = provider.get_balance(to).await.unwrap();
    assert_eq!(balance_before.saturating_add(amount), to_balance);

    assert!(api.evm_revert(state_snapshot).await.unwrap());

    let nonce = provider.get_transaction_count(from).await.unwrap();
    assert_eq!(nonce, initial_nonce);
    let balance = provider.get_balance(from).await.unwrap();
    assert_eq!(balance, handle.genesis_balance());
    let balance = provider.get_balance(to).await.unwrap();
    assert_eq!(balance, handle.genesis_balance());
    assert_eq!(block_number, provider.get_block_number().await.unwrap());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_state_snapshotting_repeated() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();

    let state_snapshot = api.evm_snapshot().await.unwrap();

    let accounts: Vec<_> = handle.dev_wallets().collect();
    let from = accounts[0].address();
    let to = accounts[1].address();
    let block_number = provider.get_block_number().await.unwrap();

    let initial_nonce = provider.get_transaction_count(from).await.unwrap();
    let balance_before = provider.get_balance(to).await.unwrap();
    let amount = handle.genesis_balance().checked_div(U256::from(92u64)).unwrap();

    let tx = TransactionRequest::default().to(to).value(amount).from(from);
    let tx = WithOtherFields::new(tx);
    let tx_provider = handle.http_provider();
    let _ = tx_provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();

    let nonce = provider.get_transaction_count(from).await.unwrap();
    assert_eq!(nonce, initial_nonce + 1);
    let to_balance = provider.get_balance(to).await.unwrap();
    assert_eq!(balance_before.saturating_add(amount), to_balance);

    let _second_state_snapshot = api.evm_snapshot().await.unwrap();

    assert!(api.evm_revert(state_snapshot).await.unwrap());

    let nonce = provider.get_transaction_count(from).await.unwrap();
    assert_eq!(nonce, initial_nonce);
    let balance = provider.get_balance(from).await.unwrap();
    assert_eq!(balance, handle.genesis_balance());
    let balance = provider.get_balance(to).await.unwrap();
    assert_eq!(balance, handle.genesis_balance());
    assert_eq!(block_number, provider.get_block_number().await.unwrap());

    // invalidated
    // TODO enable after <https://github.com/foundry-rs/foundry/pull/6366>
    // assert!(!api.evm_revert(second_snapshot).await.unwrap());

    // nothing is reverted, snapshot gone
    assert!(!api.evm_revert(state_snapshot).await.unwrap());
}

// <https://github.com/foundry-rs/foundry/issues/6463>
#[tokio::test(flavor = "multi_thread")]
async fn test_fork_state_snapshotting_blocks() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();

    let state_snapshot = api.evm_snapshot().await.unwrap();

    let accounts: Vec<_> = handle.dev_wallets().collect();
    let from = accounts[0].address();
    let to = accounts[1].address();
    let block_number = provider.get_block_number().await.unwrap();

    let initial_nonce = provider.get_transaction_count(from).await.unwrap();
    let balance_before = provider.get_balance(to).await.unwrap();
    let amount = handle.genesis_balance().checked_div(U256::from(2u64)).unwrap();

    // send the transaction
    let tx = TransactionRequest::default().to(to).value(amount).from(from);
    let tx = WithOtherFields::new(tx);
    let _ = provider.send_transaction(tx.clone()).await.unwrap().get_receipt().await.unwrap();

    let block_number_after = provider.get_block_number().await.unwrap();
    assert_eq!(block_number_after, block_number + 1);

    let nonce = provider.get_transaction_count(from).await.unwrap();
    assert_eq!(nonce, initial_nonce + 1);
    let to_balance = provider.get_balance(to).await.unwrap();
    assert_eq!(balance_before.saturating_add(amount), to_balance);

    assert!(api.evm_revert(state_snapshot).await.unwrap());

    assert_eq!(initial_nonce, provider.get_transaction_count(from).await.unwrap());
    let block_number_after = provider.get_block_number().await.unwrap();
    assert_eq!(block_number_after, block_number);

    // repeat transaction
    let _ = provider.send_transaction(tx.clone()).await.unwrap().get_receipt().await.unwrap();
    let nonce = provider.get_transaction_count(from).await.unwrap();
    assert_eq!(nonce, initial_nonce + 1);

    // revert again: nothing to revert since state snapshot gone
    assert!(!api.evm_revert(state_snapshot).await.unwrap());
    let nonce = provider.get_transaction_count(from).await.unwrap();
    assert_eq!(nonce, initial_nonce + 1);
    let block_number_after = provider.get_block_number().await.unwrap();
    assert_eq!(block_number_after, block_number + 1);
}

/// tests that the remote state and local state are kept separate.
/// changes don't make into the read only Database that holds the remote state, which is flushed to
/// a cache file.
#[tokio::test(flavor = "multi_thread")]
async fn test_separate_states() {
    let (api, handle) = spawn(fork_config().with_fork_block_number(Some(14723772u64))).await;
    let provider = handle.http_provider();

    let addr: Address = "000000000000000000000000000000000000dEaD".parse().unwrap();

    let remote_balance = provider.get_balance(addr).await.unwrap();
    assert_eq!(remote_balance, U256::from(12556104082473169733500u128));

    api.anvil_set_balance(addr, U256::from(1337u64)).await.unwrap();
    let balance = provider.get_balance(addr).await.unwrap();
    assert_eq!(balance, U256::from(1337u64));

    let fork = api.get_fork().unwrap();
    let fork_db = fork.database.read().await;
    let acc = fork_db
        .maybe_inner()
        .expect("could not get fork db inner")
        .db()
        .accounts
        .read()
        .get(&addr)
        .cloned()
        .unwrap();

    assert_eq!(acc.balance, remote_balance);
}

#[tokio::test(flavor = "multi_thread")]
async fn can_deploy_greeter_on_fork() {
    let (_api, handle) = spawn(fork_config().with_fork_block_number(Some(14723772u64))).await;

    let wallet = handle.dev_wallets().next().unwrap();
    let signer: EthereumWallet = wallet.into();

    let provider = http_provider_with_signer(&handle.http_endpoint(), signer);

    let greeter_contract = Greeter::deploy(&provider, "Hello World!".to_string()).await.unwrap();

    let greeting = greeter_contract.greet().call().await.unwrap();
    assert_eq!("Hello World!", greeting._0);

    let greeter_contract = Greeter::deploy(&provider, "Hello World!".to_string()).await.unwrap();

    let greeting = greeter_contract.greet().call().await.unwrap();
    assert_eq!("Hello World!", greeting._0);
}

#[tokio::test(flavor = "multi_thread")]
async fn can_reset_properly() {
    let (origin_api, origin_handle) = spawn(NodeConfig::test()).await;
    let account = origin_handle.dev_accounts().next().unwrap();
    let origin_provider = origin_handle.http_provider();
    let origin_nonce = 1u64;
    origin_api.anvil_set_nonce(account, U256::from(origin_nonce)).await.unwrap();

    assert_eq!(origin_nonce, origin_provider.get_transaction_count(account).await.unwrap());

    let (fork_api, fork_handle) =
        spawn(NodeConfig::test().with_eth_rpc_url(Some(origin_handle.http_endpoint()))).await;

    let fork_provider = fork_handle.http_provider();
    let fork_tx_provider = http_provider(&fork_handle.http_endpoint());
    assert_eq!(origin_nonce, fork_provider.get_transaction_count(account).await.unwrap());

    let to = Address::random();
    let to_balance = fork_provider.get_balance(to).await.unwrap();
    let tx = TransactionRequest::default().from(account).to(to).value(U256::from(1337u64));
    let tx = WithOtherFields::new(tx);
    let tx = fork_tx_provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();

    // nonce incremented by 1
    assert_eq!(origin_nonce + 1, fork_provider.get_transaction_count(account).await.unwrap());

    // resetting to origin state
    fork_api.anvil_reset(Some(Forking::default())).await.unwrap();

    // nonce reset to origin
    assert_eq!(origin_nonce, fork_provider.get_transaction_count(account).await.unwrap());

    // balance is reset
    assert_eq!(to_balance, fork_provider.get_balance(to).await.unwrap());

    // tx does not exist anymore
    assert!(fork_tx_provider.get_transaction_by_hash(tx.transaction_hash).await.unwrap().is_none())
}

// Ref: <https://github.com/foundry-rs/foundry/issues/8684>
#[tokio::test(flavor = "multi_thread")]
async fn can_reset_fork_to_new_fork() {
    let eth_rpc_url = next_rpc_endpoint(NamedChain::Mainnet);
    let (api, handle) = spawn(NodeConfig::test().with_eth_rpc_url(Some(eth_rpc_url))).await;
    let provider = handle.http_provider();

    let op = address!("C0d3c0d3c0D3c0D3C0d3C0D3C0D3c0d3c0d30007"); // L2CrossDomainMessenger - Dead on mainnet.

    let tx = TransactionRequest::default().with_to(op).with_input("0x54fd4d50");

    let tx = WithOtherFields::new(tx);

    let mainnet_call_output = provider.call(&tx).await.unwrap();

    assert_eq!(mainnet_call_output, Bytes::new()); // 0x

    let optimism = next_rpc_endpoint(NamedChain::Optimism);

    api.anvil_reset(Some(Forking {
        json_rpc_url: Some(optimism.to_string()),
        block_number: Some(124659890),
    }))
    .await
    .unwrap();

    let code = provider.get_code_at(op).await.unwrap();

    assert_ne!(code, Bytes::new());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_timestamp() {
    let start = std::time::Instant::now();

    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();

    let block = provider
        .get_block(BlockId::Number(BLOCK_NUMBER.into()), false.into())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(block.header.timestamp, BLOCK_TIMESTAMP);

    let accounts: Vec<_> = handle.dev_wallets().collect();
    let from = accounts[0].address();

    let tx =
        TransactionRequest::default().to(Address::random()).value(U256::from(1337u64)).from(from);
    let tx = WithOtherFields::new(tx);
    let tx = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();
    let status = tx.inner.inner.inner.receipt.status.coerce_status();
    assert!(status);

    let block = provider.get_block(BlockId::latest(), false.into()).await.unwrap().unwrap();

    let elapsed = start.elapsed().as_secs() + 1;

    // ensure the diff between the new mined block and the original block is within the elapsed time
    let diff = block.header.timestamp - BLOCK_TIMESTAMP;
    assert!(diff <= elapsed, "diff={diff}, elapsed={elapsed}");

    let start = std::time::Instant::now();
    // reset to check timestamp works after resetting
    api.anvil_reset(Some(Forking { json_rpc_url: None, block_number: Some(BLOCK_NUMBER) }))
        .await
        .unwrap();
    let block = provider
        .get_block(BlockId::Number(BLOCK_NUMBER.into()), false.into())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(block.header.timestamp, BLOCK_TIMESTAMP);

    let tx =
        TransactionRequest::default().to(Address::random()).value(U256::from(1337u64)).from(from);
    let tx = WithOtherFields::new(tx);
    let _ = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap(); // FIXME: Awaits endlessly here.

    let block = provider.get_block(BlockId::latest(), false.into()).await.unwrap().unwrap();
    let elapsed = start.elapsed().as_secs() + 1;
    let diff = block.header.timestamp - BLOCK_TIMESTAMP;
    assert!(diff <= elapsed);

    // ensure that after setting a timestamp manually, then next block time is correct
    let start = std::time::Instant::now();
    api.anvil_reset(Some(Forking { json_rpc_url: None, block_number: Some(BLOCK_NUMBER) }))
        .await
        .unwrap();
    api.evm_set_next_block_timestamp(BLOCK_TIMESTAMP + 1).unwrap();
    let tx =
        TransactionRequest::default().to(Address::random()).value(U256::from(1337u64)).from(from);
    let tx = WithOtherFields::new(tx);
    let _tx = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();

    let block = provider.get_block(BlockId::latest(), false.into()).await.unwrap().unwrap();
    assert_eq!(block.header.timestamp, BLOCK_TIMESTAMP + 1);

    let tx =
        TransactionRequest::default().to(Address::random()).value(U256::from(1337u64)).from(from);
    let tx = WithOtherFields::new(tx);
    let _ = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();

    let block = provider.get_block(BlockId::latest(), false.into()).await.unwrap().unwrap();
    let elapsed = start.elapsed().as_secs() + 1;
    let diff = block.header.timestamp - (BLOCK_TIMESTAMP + 1);
    assert!(diff <= elapsed);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_set_empty_code() {
    let (api, _handle) = spawn(fork_config()).await;
    let addr = "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984".parse().unwrap();
    let code = api.get_code(addr, None).await.unwrap();
    assert!(!code.as_ref().is_empty());
    api.anvil_set_code(addr, Vec::new().into()).await.unwrap();
    let code = api.get_code(addr, None).await.unwrap();
    assert!(code.as_ref().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_can_send_tx() {
    let (api, handle) =
        spawn(fork_config().with_blocktime(Some(std::time::Duration::from_millis(800)))).await;

    let wallet = PrivateKeySigner::random();
    let signer = wallet.address();
    let provider = handle.http_provider();
    // let provider = SignerMiddleware::new(provider, wallet);

    api.anvil_set_balance(signer, U256::MAX).await.unwrap();
    api.anvil_impersonate_account(signer).await.unwrap(); // Added until WalletFiller for alloy-provider is fixed.
    let balance = provider.get_balance(signer).await.unwrap();
    assert_eq!(balance, U256::MAX);

    let addr = Address::random();
    let val = U256::from(1337u64);
    let tx = TransactionRequest::default().to(addr).value(val).from(signer);
    let tx = WithOtherFields::new(tx);
    // broadcast it via the eth_sendTransaction API
    let _ = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();

    let balance = provider.get_balance(addr).await.unwrap();
    assert_eq!(balance, val);
}

// <https://github.com/foundry-rs/foundry/issues/1920>
#[tokio::test(flavor = "multi_thread")]
async fn test_fork_nft_set_approve_all() {
    let (api, handle) = spawn(
        fork_config()
            .with_fork_block_number(Some(14812197u64))
            .with_blocktime(Some(Duration::from_secs(5)))
            .with_chain_id(1u64.into()),
    )
    .await;

    // create and fund a random wallet
    let wallet = PrivateKeySigner::random();
    let signer = wallet.address();
    api.anvil_set_balance(signer, U256::from(1000e18)).await.unwrap();

    let provider = handle.http_provider();

    // pick a random nft <https://opensea.io/assets/ethereum/0x9c8ff314c9bc7f6e59a9d9225fb22946427edc03/154>
    let nouns_addr: Address = "0x9c8ff314c9bc7f6e59a9d9225fb22946427edc03".parse().unwrap();

    let owner: Address = "0x052564eb0fd8b340803df55def89c25c432f43f4".parse().unwrap();
    let token_id: U256 = U256::from(154u64);

    let nouns = ERC721::new(nouns_addr, provider.clone());

    let real_owner = nouns.ownerOf(token_id).call().await.unwrap();
    assert_eq!(real_owner._0, owner);
    let approval = nouns.setApprovalForAll(nouns_addr, true);
    let tx = TransactionRequest::default()
        .from(owner)
        .to(nouns_addr)
        .with_input(approval.calldata().to_owned());
    let tx = WithOtherFields::new(tx);
    api.anvil_impersonate_account(owner).await.unwrap();
    let tx = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();
    let status = tx.inner.inner.inner.receipt.status.coerce_status();
    assert!(status);

    // transfer: impersonate real owner and transfer nft
    api.anvil_impersonate_account(real_owner._0).await.unwrap();

    api.anvil_set_balance(real_owner._0, U256::from(10000e18 as u64)).await.unwrap();

    let call = nouns.transferFrom(real_owner._0, signer, token_id);
    let tx = TransactionRequest::default()
        .from(real_owner._0)
        .to(nouns_addr)
        .with_input(call.calldata().to_owned());
    let tx = WithOtherFields::new(tx);
    let tx = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();
    let status = tx.inner.inner.inner.receipt.status.coerce_status();
    assert!(status);

    let real_owner = nouns.ownerOf(token_id).call().await.unwrap();
    assert_eq!(real_owner._0, wallet.address());
}

// <https://github.com/foundry-rs/foundry/issues/2261>
#[tokio::test(flavor = "multi_thread")]
async fn test_fork_with_custom_chain_id() {
    // spawn a forked node with some random chainId
    let (api, handle) = spawn(
        fork_config()
            .with_fork_block_number(Some(14812197u64))
            .with_blocktime(Some(Duration::from_secs(5)))
            .with_chain_id(3145u64.into()),
    )
    .await;

    // get the eth chainId and the txn chainId
    let eth_chain_id = api.eth_chain_id();
    let txn_chain_id = api.chain_id();

    // get the chainId in the config
    let config_chain_id = handle.config().chain_id;

    // check that the chainIds are the same
    assert_eq!(eth_chain_id.unwrap().unwrap().to::<u64>(), 3145u64);
    assert_eq!(txn_chain_id, 3145u64);
    assert_eq!(config_chain_id, Some(3145u64));
}

// <https://github.com/foundry-rs/foundry/issues/1920>
#[tokio::test(flavor = "multi_thread")]
async fn test_fork_can_send_opensea_tx() {
    let (api, handle) = spawn(
        fork_config()
            .with_fork_block_number(Some(14983338u64))
            .with_blocktime(Some(Duration::from_millis(5000))),
    )
    .await;

    let sender: Address = "0x8fdbae54b6d9f3fc2c649e3dd4602961967fd42f".parse().unwrap();

    // transfer: impersonate real sender
    api.anvil_impersonate_account(sender).await.unwrap();

    let provider = handle.http_provider();

    let input: Bytes = "0xfb0f3ee1000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003ff2e795f5000000000000000000000000000023f28ae3e9756ba982a6290f9081b6a84900b758000000000000000000000000004c00500000ad104d7dbd00e3ae0a5c00560c0000000000000000000000000003235b597a78eabcb08ffcb4d97411073211dbcb0000000000000000000000000000000000000000000000000000000000000e72000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000062ad47c20000000000000000000000000000000000000000000000000000000062d43104000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000df44e65d2a2cf40000007b02230091a7ed01230072f7006a004d60a8d4e71d599b8104250f00000000007b02230091a7ed01230072f7006a004d60a8d4e71d599b8104250f00000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002e000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000001c6bf526340000000000000000000000000008de9c5a032463c561423387a9648c5c7bcc5bc900000000000000000000000000000000000000000000000000005543df729c0000000000000000000000000006eb234847a9e3a546539aac57a071c01dc3f398600000000000000000000000000000000000000000000000000000000000000416d39b5352353a22cf2d44faa696c2089b03137a13b5acfee0366306f2678fede043bc8c7e422f6f13a3453295a4a063dac7ee6216ab7bade299690afc77397a51c00000000000000000000000000000000000000000000000000000000000000".parse().unwrap();
    let to: Address = "0x00000000006c3852cbef3e08e8df289169ede581".parse().unwrap();
    let tx = TransactionRequest::default()
        .from(sender)
        .to(to)
        .value(U256::from(20000000000000000u64))
        .with_input(input)
        .with_gas_price(22180711707u128)
        .with_gas_limit(150_000);
    let tx = WithOtherFields::new(tx);

    let tx = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();
    let status = tx.inner.inner.inner.receipt.status.coerce_status();
    assert!(status);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_base_fee() {
    let (api, handle) = spawn(fork_config()).await;

    let accounts: Vec<_> = handle.dev_wallets().collect();
    let from = accounts[0].address();

    let provider = handle.http_provider();

    api.anvil_set_next_block_base_fee_per_gas(U256::ZERO).await.unwrap();

    let addr = Address::random();
    let val = U256::from(1337u64);
    let tx = TransactionRequest::default().from(from).to(addr).value(val);
    let tx = WithOtherFields::new(tx);
    let _res = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_init_base_fee() {
    let (api, handle) = spawn(fork_config().with_fork_block_number(Some(13184859u64))).await;

    let provider = handle.http_provider();

    let block = provider.get_block(BlockId::latest(), false.into()).await.unwrap().unwrap();
    // <https://etherscan.io/block/13184859>
    assert_eq!(block.header.number, 13184859u64);
    let init_base_fee = block.header.base_fee_per_gas.unwrap();
    assert_eq!(init_base_fee, 63739886069);

    api.mine_one().await;

    let block = provider.get_block(BlockId::latest(), false.into()).await.unwrap().unwrap();

    let next_base_fee = block.header.base_fee_per_gas.unwrap();
    assert!(next_base_fee < init_base_fee);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reset_fork_on_new_blocks() {
    let (api, handle) =
        spawn(NodeConfig::test().with_eth_rpc_url(Some(rpc::next_http_archive_rpc_url()))).await;

    let anvil_provider = handle.http_provider();
    let endpoint = next_http_rpc_endpoint();
    let provider = Arc::new(get_http_provider(&endpoint));

    let current_block = anvil_provider.get_block_number().await.unwrap();

    handle.task_manager().spawn_reset_on_new_polled_blocks(provider.clone(), api);

    let mut stream = provider
        .watch_blocks()
        .await
        .unwrap()
        .with_poll_interval(Duration::from_secs(2))
        .into_stream()
        .flat_map(futures::stream::iter);
    // the http watcher may fetch multiple blocks at once, so we set a timeout here to offset edge
    // cases where the stream immediately returns a block
    tokio::time::sleep(Duration::from_secs(12)).await;
    stream.next().await.unwrap();
    stream.next().await.unwrap();

    let next_block = anvil_provider.get_block_number().await.unwrap();

    assert!(next_block > current_block, "nextblock={next_block} currentblock={current_block}")
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_call() {
    let input: Bytes = "0x77c7b8fc".parse().unwrap();
    let to: Address = "0x99d1Fa417f94dcD62BfE781a1213c092a47041Bc".parse().unwrap();
    let block_number = 14746300u64;

    let provider = http_provider(rpc::next_http_archive_rpc_url().as_str());
    let tx = TransactionRequest::default().to(to).with_input(input.clone());
    let tx = WithOtherFields::new(tx);
    let res0 = provider.call(&tx).block(BlockId::Number(block_number.into())).await.unwrap();

    let (api, _) = spawn(fork_config().with_fork_block_number(Some(block_number))).await;

    let res1 = api
        .call(
            WithOtherFields::new(TransactionRequest {
                to: Some(TxKind::from(to)),
                input: input.into(),
                ..Default::default()
            }),
            None,
            None,
        )
        .await
        .unwrap();

    assert_eq!(res0, res1);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_block_timestamp() {
    let (api, _) = spawn(fork_config()).await;

    let initial_block = api.block_by_number(BlockNumberOrTag::Latest).await.unwrap().unwrap();
    api.anvil_mine(Some(U256::from(1)), None).await.unwrap();
    let latest_block = api.block_by_number(BlockNumberOrTag::Latest).await.unwrap().unwrap();

    assert!(initial_block.header.timestamp <= latest_block.header.timestamp);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_snapshot_block_timestamp() {
    let (api, _) = spawn(fork_config()).await;

    let snapshot_id = api.evm_snapshot().await.unwrap();
    api.anvil_mine(Some(U256::from(1)), None).await.unwrap();
    let initial_block = api.block_by_number(BlockNumberOrTag::Latest).await.unwrap().unwrap();
    api.evm_revert(snapshot_id).await.unwrap();
    api.evm_set_next_block_timestamp(initial_block.header.timestamp).unwrap();
    api.anvil_mine(Some(U256::from(1)), None).await.unwrap();
    let latest_block = api.block_by_number(BlockNumberOrTag::Latest).await.unwrap().unwrap();

    assert_eq!(initial_block.header.timestamp, latest_block.header.timestamp);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_uncles_fetch() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();

    // Block on ETH mainnet with 2 uncles
    let block_with_uncles = 190u64;

    let block =
        api.block_by_number(BlockNumberOrTag::Number(block_with_uncles)).await.unwrap().unwrap();

    assert_eq!(block.uncles.len(), 2);

    let count = provider.get_uncle_count(block_with_uncles.into()).await.unwrap();
    assert_eq!(count as usize, block.uncles.len());

    let hash = BlockId::hash(block.header.hash);
    let count = provider.get_uncle_count(hash).await.unwrap();
    assert_eq!(count as usize, block.uncles.len());

    for (uncle_idx, uncle_hash) in block.uncles.iter().enumerate() {
        // Try with block number
        let uncle = provider
            .get_uncle(BlockId::number(block_with_uncles), uncle_idx as u64)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(*uncle_hash, uncle.header.hash);

        // Try with block hash
        let uncle = provider
            .get_uncle(BlockId::hash(block.header.hash), uncle_idx as u64)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(*uncle_hash, uncle.header.hash);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_block_transaction_count() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();

    let accounts: Vec<_> = handle.dev_wallets().collect();
    let sender = accounts[0].address();

    // disable automine (so there are pending transactions)
    api.anvil_set_auto_mine(false).await.unwrap();
    // transfer: impersonate real sender
    api.anvil_impersonate_account(sender).await.unwrap();

    let tx =
        TransactionRequest::default().from(sender).value(U256::from(42u64)).with_gas_limit(100_000);
    let tx = WithOtherFields::new(tx);
    let _ = provider.send_transaction(tx).await.unwrap();

    let pending_txs =
        api.block_transaction_count_by_number(BlockNumberOrTag::Pending).await.unwrap().unwrap();
    assert_eq!(pending_txs.to::<u64>(), 1);

    // mine a new block
    api.anvil_mine(None, None).await.unwrap();

    let pending_txs =
        api.block_transaction_count_by_number(BlockNumberOrTag::Pending).await.unwrap().unwrap();
    assert_eq!(pending_txs.to::<u64>(), 0);
    let latest_txs =
        api.block_transaction_count_by_number(BlockNumberOrTag::Latest).await.unwrap().unwrap();
    assert_eq!(latest_txs.to::<u64>(), 1);
    let latest_block = api.block_by_number(BlockNumberOrTag::Latest).await.unwrap().unwrap();
    let latest_txs =
        api.block_transaction_count_by_hash(latest_block.header.hash).await.unwrap().unwrap();
    assert_eq!(latest_txs.to::<u64>(), 1);

    // check txs count on an older block: 420000 has 3 txs on mainnet
    let count_txs = api
        .block_transaction_count_by_number(BlockNumberOrTag::Number(420000))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(count_txs.to::<u64>(), 3);
    let count_txs = api
        .block_transaction_count_by_hash(
            "0xb3b0e3e0c64e23fb7f1ccfd29245ae423d2f6f1b269b63b70ff882a983ce317c".parse().unwrap(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(count_txs.to::<u64>(), 3);
}

// <https://github.com/foundry-rs/foundry/issues/2931>
#[tokio::test(flavor = "multi_thread")]
async fn can_impersonate_in_fork() {
    let (api, handle) = spawn(fork_config().with_fork_block_number(Some(15347924u64))).await;
    let provider = handle.http_provider();

    let token_holder: Address = "0x2f0b23f53734252bda2277357e97e1517d6b042a".parse().unwrap();
    let to = Address::random();
    let val = U256::from(1337u64);

    // fund the impersonated account
    api.anvil_set_balance(token_holder, U256::from(1e18)).await.unwrap();

    let tx = TransactionRequest::default().from(token_holder).to(to).value(val);
    let tx = WithOtherFields::new(tx);
    let res = provider.send_transaction(tx.clone()).await;
    res.unwrap_err();

    api.anvil_impersonate_account(token_holder).await.unwrap();

    let res = provider.send_transaction(tx.clone()).await.unwrap().get_receipt().await.unwrap();
    assert_eq!(res.from, token_holder);
    let status = res.inner.inner.inner.receipt.status.coerce_status();
    assert!(status);

    let balance = provider.get_balance(to).await.unwrap();
    assert_eq!(balance, val);

    api.anvil_stop_impersonating_account(token_holder).await.unwrap();
    let res = provider.send_transaction(tx).await;
    res.unwrap_err();
}

// <https://etherscan.io/block/14608400>
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn test_total_difficulty_fork() {
    let (api, handle) = spawn(fork_config()).await;

    let total_difficulty = U256::from(46_673_965_560_973_856_260_636u128);
    let difficulty = U256::from(13_680_435_288_526_144u128);

    let provider = handle.http_provider();
    let block = provider.get_block(BlockId::latest(), false.into()).await.unwrap().unwrap();
    assert_eq!(block.header.total_difficulty, Some(total_difficulty));
    assert_eq!(block.header.difficulty, difficulty);

    api.mine_one().await;
    api.mine_one().await;

    let next_total_difficulty = total_difficulty + difficulty;

    let block = provider.get_block(BlockId::latest(), false.into()).await.unwrap().unwrap();
    assert_eq!(block.header.total_difficulty, Some(next_total_difficulty));
    assert_eq!(block.header.difficulty, U256::ZERO);
}

// <https://etherscan.io/block/14608400>
#[tokio::test(flavor = "multi_thread")]
async fn test_transaction_receipt() {
    let (api, _) = spawn(fork_config()).await;

    // A transaction from the forked block (14608400)
    let receipt = api
        .transaction_receipt(
            "0xce495d665e9091613fd962351a5cbca27a992b919d6a87d542af97e2723ec1e4".parse().unwrap(),
        )
        .await
        .unwrap();
    assert!(receipt.is_some());

    // A transaction from a block in the future (14608401)
    let receipt = api
        .transaction_receipt(
            "0x1a15472088a4a97f29f2f9159511dbf89954b58d9816e58a32b8dc17171dc0e8".parse().unwrap(),
        )
        .await
        .unwrap();
    assert!(receipt.is_none());
}

// <https://etherscan.io/block/14608400>
#[tokio::test(flavor = "multi_thread")]
async fn test_block_receipts() {
    let (api, _) = spawn(fork_config()).await;

    // Receipts from the forked block (14608400)
    let receipts = api.block_receipts(BlockNumberOrTag::Number(BLOCK_NUMBER).into()).await.unwrap();
    assert!(receipts.is_some());

    // Receipts from a block in the future (14608401)
    let receipts =
        api.block_receipts(BlockNumberOrTag::Number(BLOCK_NUMBER + 1).into()).await.unwrap();
    assert!(receipts.is_none());

    // Receipts from a block hash (14608400)
    let hash = b256!("4c1c76f89cfe4eb503b09a0993346dd82865cac9d76034efc37d878c66453f0a");
    let receipts = api.block_receipts(BlockId::Hash(hash.into())).await.unwrap();
    assert!(receipts.is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn can_override_fork_chain_id() {
    let chain_id_override = 5u64;
    let (_api, handle) = spawn(
        fork_config()
            .with_fork_block_number(Some(16506610u64))
            .with_chain_id(Some(chain_id_override)),
    )
    .await;

    let wallet = handle.dev_wallets().next().unwrap();
    let signer: EthereumWallet = wallet.into();
    let provider = http_provider_with_signer(&handle.http_endpoint(), signer);

    let greeter_contract =
        Greeter::deploy(provider.clone(), "Hello World!".to_string()).await.unwrap();
    let greeting = greeter_contract.greet().call().await.unwrap();

    assert_eq!("Hello World!", greeting._0);
    let greeter_contract =
        Greeter::deploy(provider.clone(), "Hello World!".to_string()).await.unwrap();
    let greeting = greeter_contract.greet().call().await.unwrap();
    assert_eq!("Hello World!", greeting._0);

    let provider = handle.http_provider();
    let chain_id = provider.get_chain_id().await.unwrap();
    assert_eq!(chain_id, chain_id_override);
}

// <https://github.com/foundry-rs/foundry/issues/6485>
#[tokio::test(flavor = "multi_thread")]
async fn test_fork_reset_moonbeam() {
    crate::init_tracing();
    let (api, handle) = spawn(
        fork_config()
            .with_eth_rpc_url(Some("https://rpc.api.moonbeam.network".to_string()))
            .with_fork_block_number(None::<u64>),
    )
    .await;
    let provider = handle.http_provider();

    let accounts: Vec<_> = handle.dev_wallets().collect();
    let from = accounts[0].address();

    let tx =
        TransactionRequest::default().to(Address::random()).value(U256::from(1337u64)).from(from);
    let tx = WithOtherFields::new(tx);
    api.anvil_impersonate_account(from).await.unwrap();
    let tx = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();
    let status = tx.inner.inner.inner.receipt.status.coerce_status();
    assert!(status);

    // reset to check timestamp works after resetting
    api.anvil_reset(Some(Forking {
        json_rpc_url: Some("https://rpc.api.moonbeam.network".to_string()),
        block_number: None,
    }))
    .await
    .unwrap();

    let tx =
        TransactionRequest::default().to(Address::random()).value(U256::from(1337u64)).from(from);
    let tx = WithOtherFields::new(tx);
    let tx = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();
    let status = tx.inner.inner.inner.receipt.status.coerce_status();
    assert!(status);
}

// <https://github.com/foundry-rs/foundry/issues/6640
#[tokio::test(flavor = "multi_thread")]
async fn test_fork_reset_basefee() {
    // <https://etherscan.io/block/18835000>
    let (api, _handle) = spawn(fork_config().with_fork_block_number(Some(18835000u64))).await;

    api.mine_one().await;
    let latest = api.block_by_number(BlockNumberOrTag::Latest).await.unwrap().unwrap();

    // basefee of +1 block: <https://etherscan.io/block/18835001>
    assert_eq!(latest.header.base_fee_per_gas.unwrap(), 59455969592u64);

    // now reset to block 18835000 -1
    api.anvil_reset(Some(Forking { json_rpc_url: None, block_number: Some(18835000u64 - 1) }))
        .await
        .unwrap();

    api.mine_one().await;
    let latest = api.block_by_number(BlockNumberOrTag::Latest).await.unwrap().unwrap();

    // basefee of the forked block: <https://etherscan.io/block/18835000>
    assert_eq!(latest.header.base_fee_per_gas.unwrap(), 59017001138);
}

// <https://github.com/foundry-rs/foundry/issues/6795>
#[tokio::test(flavor = "multi_thread")]
async fn test_arbitrum_fork_dev_balance() {
    let (api, handle) = spawn(
        fork_config()
            .with_fork_block_number(None::<u64>)
            .with_eth_rpc_url(Some(next_rpc_endpoint(NamedChain::Arbitrum))),
    )
    .await;

    let accounts: Vec<_> = handle.dev_wallets().collect();
    for acc in accounts {
        let balance = api.balance(acc.address(), Some(Default::default())).await.unwrap();
        assert_eq!(balance, U256::from(100000000000000000000u128));
    }
}

// <https://github.com/foundry-rs/foundry/issues/9152>
#[tokio::test(flavor = "multi_thread")]
async fn test_arb_fork_mining() {
    let fork_block_number = 266137031u64;
    let fork_rpc = next_rpc_endpoint(NamedChain::Arbitrum);
    let (api, _handle) = spawn(
        fork_config()
            .with_fork_block_number(Some(fork_block_number))
            .with_eth_rpc_url(Some(fork_rpc)),
    )
    .await;

    let init_blk_num = api.block_number().unwrap().to::<u64>();

    // Mine one
    api.mine_one().await;
    let mined_blk_num = api.block_number().unwrap().to::<u64>();

    assert_eq!(mined_blk_num, init_blk_num + 1);
}

// <https://github.com/foundry-rs/foundry/issues/6749>
#[tokio::test(flavor = "multi_thread")]
async fn test_arbitrum_fork_block_number() {
    // fork to get initial block for test
    let (_, handle) = spawn(
        fork_config()
            .with_fork_block_number(None::<u64>)
            .with_eth_rpc_url(Some(next_rpc_endpoint(NamedChain::Arbitrum))),
    )
    .await;
    let provider = handle.http_provider();
    let initial_block_number = provider.get_block_number().await.unwrap();

    // fork again at block number returned by `eth_blockNumber`
    // if wrong block number returned (e.g. L1) then fork will fail with error code -32000: missing
    // trie node
    let (api, _) = spawn(
        fork_config()
            .with_fork_block_number(Some(initial_block_number))
            .with_eth_rpc_url(Some(next_rpc_endpoint(NamedChain::Arbitrum))),
    )
    .await;
    let block_number = api.block_number().unwrap().to::<u64>();
    assert_eq!(block_number, initial_block_number);

    // take snapshot at initial block number
    let snapshot_state = api.evm_snapshot().await.unwrap();

    // mine new block and check block number returned by `eth_blockNumber`
    api.mine_one().await;
    let block_number = api.block_number().unwrap().to::<u64>();
    assert_eq!(block_number, initial_block_number + 1);

    // test block by number API call returns proper block number and `l1BlockNumber` is set
    let block_by_number = api.block_by_number(BlockNumberOrTag::Latest).await.unwrap().unwrap();
    assert_eq!(block_by_number.header.number, initial_block_number + 1);
    assert!(block_by_number.other.get("l1BlockNumber").is_some());

    // revert to recorded snapshot and check block number
    assert!(api.evm_revert(snapshot_state).await.unwrap());
    let block_number = api.block_number().unwrap().to::<u64>();
    assert_eq!(block_number, initial_block_number);

    // reset fork to different block number and compare with block returned by `eth_blockNumber`
    api.anvil_reset(Some(Forking {
        json_rpc_url: Some(next_rpc_endpoint(NamedChain::Arbitrum)),
        block_number: Some(initial_block_number - 2),
    }))
    .await
    .unwrap();
    let block_number = api.block_number().unwrap().to::<u64>();
    assert_eq!(block_number, initial_block_number - 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_base_fork_gas_limit() {
    // fork to get initial block for test
    let (api, handle) = spawn(
        fork_config()
            .with_fork_block_number(None::<u64>)
            .with_eth_rpc_url(Some(next_rpc_endpoint(NamedChain::Base))),
    )
    .await;

    let provider = handle.http_provider();
    let block = provider
        .get_block(BlockId::Number(BlockNumberOrTag::Latest), BlockTransactionsKind::Hashes)
        .await
        .unwrap()
        .unwrap();

    assert!(api.gas_limit() >= uint!(96_000_000_U256));
    assert!(block.header.gas_limit >= 96_000_000_u64);
}

// <https://github.com/foundry-rs/foundry/issues/7023>
#[tokio::test(flavor = "multi_thread")]
async fn test_fork_execution_reverted() {
    let target = 16681681u64;
    let (api, _handle) = spawn(fork_config().with_fork_block_number(Some(target + 1))).await;

    let resp = api
        .call(
            WithOtherFields::new(TransactionRequest {
                to: Some(TxKind::from(address!("Fd6CC4F251eaE6d02f9F7B41D1e80464D3d2F377"))),
                input: TransactionInput::new(bytes!("8f283b3c")),
                ..Default::default()
            }),
            Some(target.into()),
            None,
        )
        .await;

    assert!(resp.is_err());
    let err = resp.unwrap_err();
    assert!(err.to_string().contains("execution reverted"));
}

// <https://github.com/foundry-rs/foundry/issues/8227>
#[tokio::test(flavor = "multi_thread")]
async fn test_immutable_fork_transaction_hash() {
    use std::str::FromStr;

    // Fork to a block with a specific transaction
    let fork_tx_hash =
        TxHash::from_str("39d64ebf9eb3f07ede37f8681bc3b61928817276c4c4680b6ef9eac9f88b6786")
            .unwrap();
    let (api, _) = spawn(
        fork_config()
            .with_blocktime(Some(Duration::from_millis(500)))
            .with_fork_transaction_hash(Some(fork_tx_hash))
            .with_eth_rpc_url(Some("https://rpc.immutable.com".to_string())),
    )
    .await;

    let fork_block_number = 8521008;

    // Make sure the fork starts from previous block
    let mut block_number = api.block_number().unwrap().to::<u64>();
    assert_eq!(block_number, fork_block_number - 1);

    // Wait for fork to pass the target block
    while block_number < fork_block_number {
        sleep(Duration::from_millis(250));
        block_number = api.block_number().unwrap().to::<u64>();
    }

    let block = api
        .block_by_number(BlockNumberOrTag::Number(fork_block_number - 1))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(block.transactions.len(), 14);
    let block = api
        .block_by_number_full(BlockNumberOrTag::Number(fork_block_number))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(block.transactions.len(), 3);

    // Validate the transactions preceding the target transaction exist
    let expected_transactions = [
        TxHash::from_str("1bfe33136edc3d26bd01ce75c8f5ae14fffe8b142d30395cb4b6d3dc3043f400")
            .unwrap(),
        TxHash::from_str("8c0ce5fb9ec2c8e03f7fcc69c7786393c691ce43b58a06d74d6733679308fc01")
            .unwrap(),
        fork_tx_hash,
    ];
    for expected in [
        (expected_transactions[0], address!("8C1aB379E7263d37049505626D2F975288F5dF12")),
        (expected_transactions[1], address!("df918d9D02d5C7Df6825a7046dBF3D10F705Aa76")),
        (expected_transactions[2], address!("5Be88952ce249024613e0961eB437f5E9424A90c")),
    ] {
        let tx = api.backend.mined_transaction_by_hash(expected.0).unwrap();
        assert_eq!(tx.inner.from, expected.1);
    }

    // Validate the order of transactions in the new block
    for expected in [
        (expected_transactions[0], 0),
        (expected_transactions[1], 1),
        (expected_transactions[2], 2),
    ] {
        let tx = api
            .backend
            .mined_block_by_number(BlockNumberOrTag::Number(fork_block_number))
            .map(|b| b.header.hash)
            .and_then(|hash| {
                api.backend.mined_transaction_by_block_hash_and_index(hash, expected.1.into())
            })
            .unwrap();
        assert_eq!(tx.tx_hash().to_string(), expected.0.to_string());
    }
}

// <https://github.com/foundry-rs/foundry/issues/4700>
#[tokio::test(flavor = "multi_thread")]
async fn test_fork_query_at_fork_block() {
    let (api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();
    let info = api.anvil_node_info().await.unwrap();
    let number = info.fork_config.fork_block_number.unwrap();
    assert_eq!(number, BLOCK_NUMBER);

    let address = Address::random();

    let balance = provider.get_balance(address).await.unwrap();
    api.evm_mine(None).await.unwrap();
    api.anvil_set_balance(address, balance + U256::from(1)).await.unwrap();

    let balance_before =
        provider.get_balance(address).block_id(BlockId::number(number)).await.unwrap();

    assert_eq!(balance_before, balance);
}

// <https://github.com/foundry-rs/foundry/issues/4173>
#[tokio::test(flavor = "multi_thread")]
async fn test_reset_dev_account_nonce() {
    let config: NodeConfig = fork_config();
    let address = config.genesis_accounts[0].address();
    let (api, handle) = spawn(config).await;
    let provider = handle.http_provider();
    let info = api.anvil_node_info().await.unwrap();
    let number = info.fork_config.fork_block_number.unwrap();
    assert_eq!(number, BLOCK_NUMBER);

    let nonce_before = provider.get_transaction_count(address).await.unwrap();

    // Reset to older block with other nonce
    api.anvil_reset(Some(Forking {
        json_rpc_url: None,
        block_number: Some(BLOCK_NUMBER - 1_000_000),
    }))
    .await
    .unwrap();

    let nonce_after = provider.get_transaction_count(address).await.unwrap();

    assert!(nonce_before > nonce_after);

    let receipt = provider
        .send_transaction(WithOtherFields::new(
            TransactionRequest::default()
                .from(address)
                .to(address)
                .nonce(nonce_after)
                .gas_limit(21000),
        ))
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap();

    assert!(receipt.status());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reset_updates_cache_path_when_rpc_url_not_provided() {
    let config: NodeConfig = fork_config();

    let (mut api, _handle) = spawn(config).await;
    let info = api.anvil_node_info().await.unwrap();
    let number = info.fork_config.fork_block_number.unwrap();
    assert_eq!(number, BLOCK_NUMBER);

    async fn get_block_from_cache_path(api: &mut EthApi) -> u64 {
        let db = api.backend.get_db().read().await;
        let cache_debug = format!("{:?}", db.maybe_inner().unwrap().cache());
        let re = regex::Regex::new(r#"JsonBlockCacheDB \{ cache_path: Some\("([^"]+)"\)"#).unwrap();
        let cache_path = re
            .captures_iter(&cache_debug)
            .next()
            .expect("must have JsonBlockCacheDB match")
            .get(1)
            .expect("must have matching path")
            .as_str();

        std::path::PathBuf::from(cache_path)
            .parent()
            .expect("must have filename")
            .file_name()
            .expect("must have block number as dir name")
            .to_str()
            .expect("must be valid string")
            .parse::<u64>()
            .expect("must be valid number")
    }

    assert_eq!(BLOCK_NUMBER, get_block_from_cache_path(&mut api).await);

    // Reset to older block without specifying a new rpc url
    api.anvil_reset(Some(Forking {
        json_rpc_url: None,
        block_number: Some(BLOCK_NUMBER - 1_000_000),
    }))
    .await
    .unwrap();

    assert_eq!(BLOCK_NUMBER - 1_000_000, get_block_from_cache_path(&mut api).await);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fork_get_account() {
    let (_api, handle) = spawn(fork_config()).await;
    let provider = handle.http_provider();

    let accounts = handle.dev_accounts().collect::<Vec<_>>();

    let alice = accounts[0];
    let bob = accounts[1];

    let init_block = provider.get_block_number().await.unwrap();
    let alice_bal = provider.get_balance(alice).await.unwrap();
    let alice_nonce = provider.get_transaction_count(alice).await.unwrap();
    let alice_acc_init = provider.get_account(alice).await.unwrap();

    assert_eq!(alice_acc_init.balance, alice_bal);
    assert_eq!(alice_acc_init.nonce, alice_nonce);

    let tx = TransactionRequest::default().from(alice).to(bob).value(U256::from(142));

    let tx = WithOtherFields::new(tx);
    let receipt = provider.send_transaction(tx).await.unwrap().get_receipt().await.unwrap();

    assert!(receipt.status());
    assert_eq!(init_block + 1, receipt.block_number.unwrap());

    let alice_acc = provider.get_account(alice).await.unwrap();

    assert_eq!(
        alice_acc.balance,
        alice_bal -
            (U256::from(142) +
                U256::from(receipt.gas_used as u128 * receipt.effective_gas_price)),
    );
    assert_eq!(alice_acc.nonce, alice_nonce + 1);

    let alice_acc_prev_block = provider.get_account(alice).number(init_block).await.unwrap();

    assert_eq!(alice_acc_init, alice_acc_prev_block);
}
