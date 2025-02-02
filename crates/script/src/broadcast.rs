use crate::{
    build::LinkedBuildData, progress::ScriptProgress, sequence::ScriptSequenceKind,
    verify::BroadcastedState, ScriptArgs, ScriptConfig,
};
use alloy_chains::Chain;
use alloy_consensus::TxEnvelope;
use alloy_eips::{eip2718::Encodable2718, BlockId};
use alloy_network::{
    primitives::BlockTransactionsKind, AnyNetwork, EthereumWallet, TransactionBuilder,
};
use alloy_primitives::{
    map::{AddressHashMap, AddressHashSet},
    utils::format_units,
    Address, TxHash,
};
use alloy_provider::{utils::Eip1559Estimation, Provider};
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use alloy_transport::Transport;
use eyre::{bail, Context, Result};
use forge_verify::provider::VerificationProviderType;
use foundry_cheatcodes::Wallets;
use foundry_cli::utils::{has_batch_support, has_different_gas_calc};
use foundry_common::{
    provider::{get_http_provider, try_get_http_provider, RetryProvider},
    shell, TransactionMaybeSigned,
};
use foundry_config::Config;
use futures::{future::join_all, StreamExt};
use itertools::Itertools;
use std::{cmp::Ordering, sync::Arc};

pub async fn estimate_gas<P, T>(
    tx: &mut WithOtherFields<TransactionRequest>,
    provider: &P,
    estimate_multiplier: u64,
) -> Result<()>
where
    P: Provider<T, AnyNetwork>,
    T: Transport + Clone,
{
    // if already set, some RPC endpoints might simply return the gas value that is already
    // set in the request and omit the estimate altogether, so we remove it here
    tx.gas = None;

    tx.set_gas_limit(
        provider.estimate_gas(tx).await.wrap_err("Failed to estimate gas for tx")? *
            estimate_multiplier /
            100,
    );
    Ok(())
}

pub async fn get_block_gas_limit(provider_url: &str, block_number: Option<u64>) -> Result<u64> {
    let provider = try_get_http_provider(provider_url)
        .wrap_err_with(|| format!("bad fork_url provider: {provider_url}"))?;

    let block_id = block_number.map_or(BlockId::latest(), BlockId::number);
    let block = provider.get_block(block_id, BlockTransactionsKind::Hashes).await?;

    if let Some(block) = block {
        Ok(block.header.gas_limit)
    } else {
        bail!("Failed to get block for block number: {}", block_number.unwrap_or_default())
    }
}

pub async fn next_nonce(
    caller: Address,
    provider_url: &str,
    block_number: Option<u64>,
) -> eyre::Result<u64> {
    let provider = try_get_http_provider(provider_url)
        .wrap_err_with(|| format!("bad fork_url provider: {provider_url}"))?;

    let block_id = block_number.map_or(BlockId::latest(), BlockId::number);
    Ok(provider.get_transaction_count(caller).block_id(block_id).await?)
}

pub async fn send_transaction(
    provider: Arc<RetryProvider>,
    mut kind: SendTransactionKind<'_>,
    sequential_broadcast: bool,
    is_fixed_gas_limit: bool,
    estimate_via_rpc: bool,
    estimate_multiplier: u64,
) -> Result<TxHash> {
    if let SendTransactionKind::Raw(tx, _) | SendTransactionKind::Unlocked(tx) = &mut kind {
        if sequential_broadcast {
            let from = tx.from.expect("no sender");

            let tx_nonce = tx.nonce.expect("no nonce");
            for attempt in 0..5 {
                let nonce = provider.get_transaction_count(from).await?;
                match nonce.cmp(&tx_nonce) {
                    Ordering::Greater => {
                        bail!("EOA nonce changed unexpectedly while sending transactions. Expected {tx_nonce} got {nonce} from provider.")
                    }
                    Ordering::Less => {
                        if attempt == 4 {
                            bail!("After 5 attempts, provider nonce ({nonce}) is still behind expected nonce ({tx_nonce}).")
                        }
                        warn!("Expected nonce ({tx_nonce}) is ahead of provider nonce ({nonce}). Retrying in 1 second...");
                        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                    }
                    Ordering::Equal => {
                        // Nonces are equal, we can proceed
                        break;
                    }
                }
            }
        }

        // Chains which use `eth_estimateGas` are being sent sequentially and require their
        // gas to be re-estimated right before broadcasting.
        if !is_fixed_gas_limit && estimate_via_rpc {
            estimate_gas(tx, &provider, estimate_multiplier).await?;
        }
    }

    let pending = match kind {
        SendTransactionKind::Unlocked(tx) => {
            debug!("sending transaction from unlocked account {:?}", tx);

            // Submit the transaction
            provider.send_transaction(tx).await?
        }
        SendTransactionKind::Raw(tx, signer) => {
            debug!("sending transaction: {:?}", tx);
            let signed = tx.build(signer).await?;

            // Submit the raw transaction
            provider.send_raw_transaction(signed.encoded_2718().as_ref()).await?
        }
        SendTransactionKind::Signed(tx) => {
            debug!("sending transaction: {:?}", tx);
            provider.send_raw_transaction(tx.encoded_2718().as_ref()).await?
        }
    };

    Ok(*pending.tx_hash())
}

/// How to send a single transaction
#[derive(Clone)]
pub enum SendTransactionKind<'a> {
    Unlocked(WithOtherFields<TransactionRequest>),
    Raw(WithOtherFields<TransactionRequest>, &'a EthereumWallet),
    Signed(TxEnvelope),
}

/// Represents how to send _all_ transactions
pub enum SendTransactionsKind {
    /// Send via `eth_sendTransaction` and rely on the  `from` address being unlocked.
    Unlocked(AddressHashSet),
    /// Send a signed transaction via `eth_sendRawTransaction`
    Raw(AddressHashMap<EthereumWallet>),
}

impl SendTransactionsKind {
    /// Returns the [`SendTransactionKind`] for the given address
    ///
    /// Returns an error if no matching signer is found or the address is not unlocked
    pub fn for_sender(
        &self,
        addr: &Address,
        tx: WithOtherFields<TransactionRequest>,
    ) -> Result<SendTransactionKind<'_>> {
        match self {
            Self::Unlocked(unlocked) => {
                if !unlocked.contains(addr) {
                    bail!("Sender address {:?} is not unlocked", addr)
                }
                Ok(SendTransactionKind::Unlocked(tx))
            }
            Self::Raw(wallets) => {
                if let Some(wallet) = wallets.get(addr) {
                    Ok(SendTransactionKind::Raw(tx, wallet))
                } else {
                    bail!("No matching signer for {:?} found", addr)
                }
            }
        }
    }
}

/// State after we have bundled all
/// [`TransactionWithMetadata`](forge_script_sequence::TransactionWithMetadata) objects into a
/// single [`ScriptSequenceKind`] object containing one or more script sequences.
pub struct BundledState {
    pub args: ScriptArgs,
    pub script_config: ScriptConfig,
    pub script_wallets: Wallets,
    pub build_data: LinkedBuildData,
    pub sequence: ScriptSequenceKind,
}

impl BundledState {
    pub async fn wait_for_pending(mut self) -> Result<Self> {
        let progress = ScriptProgress::default();
        let progress_ref = &progress;
        let futs = self
            .sequence
            .sequences_mut()
            .iter_mut()
            .enumerate()
            .map(|(sequence_idx, sequence)| async move {
                let rpc_url = sequence.rpc_url();
                let provider = Arc::new(get_http_provider(rpc_url));
                progress_ref
                    .wait_for_pending(
                        sequence_idx,
                        sequence,
                        &provider,
                        self.script_config.config.transaction_timeout,
                    )
                    .await
            })
            .collect::<Vec<_>>();

        let errors = join_all(futs).await.into_iter().filter_map(Result::err).collect::<Vec<_>>();

        self.sequence.save(true, false)?;

        if !errors.is_empty() {
            return Err(eyre::eyre!("{}", errors.iter().format("\n")));
        }

        Ok(self)
    }

    /// Broadcasts transactions from all sequences.
    pub async fn broadcast(mut self) -> Result<BroadcastedState> {
        let required_addresses = self
            .sequence
            .sequences()
            .iter()
            .flat_map(|sequence| {
                sequence
                    .transactions()
                    .filter(|tx| tx.is_unsigned())
                    .map(|tx| tx.from().expect("missing from"))
            })
            .collect::<AddressHashSet>();

        if required_addresses.contains(&Config::DEFAULT_SENDER) {
            eyre::bail!(
                "You seem to be using Foundry's default sender. Be sure to set your own --sender."
            );
        }

        let send_kind = if self.args.unlocked {
            SendTransactionsKind::Unlocked(required_addresses.clone())
        } else {
            let signers = self.script_wallets.into_multi_wallet().into_signers()?;
            let mut missing_addresses = Vec::new();

            for addr in &required_addresses {
                if !signers.contains_key(addr) {
                    missing_addresses.push(addr);
                }
            }

            if !missing_addresses.is_empty() {
                eyre::bail!(
                    "No associated wallet for addresses: {:?}. Unlocked wallets: {:?}",
                    missing_addresses,
                    signers.keys().collect::<Vec<_>>()
                );
            }

            let signers = signers
                .into_iter()
                .map(|(addr, signer)| (addr, EthereumWallet::new(signer)))
                .collect();

            SendTransactionsKind::Raw(signers)
        };

        let progress = ScriptProgress::default();

        for i in 0..self.sequence.sequences().len() {
            let mut sequence = self.sequence.sequences_mut().get_mut(i).unwrap();

            let provider = Arc::new(try_get_http_provider(sequence.rpc_url())?);
            let already_broadcasted = sequence.receipts.len();

            let seq_progress = progress.get_sequence_progress(i, sequence);

            if already_broadcasted < sequence.transactions.len() {
                let is_legacy = Chain::from(sequence.chain).is_legacy() || self.args.legacy;
                // Make a one-time gas price estimation
                let (gas_price, eip1559_fees) = match (
                    is_legacy,
                    self.args.with_gas_price,
                    self.args.priority_gas_price,
                ) {
                    (true, Some(gas_price), _) => (Some(gas_price.to()), None),
                    (true, None, _) => (Some(provider.get_gas_price().await?), None),
                    (false, Some(max_fee_per_gas), Some(max_priority_fee_per_gas)) => (
                        None,
                        Some(Eip1559Estimation {
                            max_fee_per_gas: max_fee_per_gas.to(),
                            max_priority_fee_per_gas: max_priority_fee_per_gas.to(),
                        }),
                    ),
                    (false, _, _) => {
                        let mut fees = provider.estimate_eip1559_fees(None).await.wrap_err("Failed to estimate EIP1559 fees. This chain might not support EIP1559, try adding --legacy to your command.")?;

                        if let Some(gas_price) = self.args.with_gas_price {
                            fees.max_fee_per_gas = gas_price.to();
                        }

                        if let Some(priority_gas_price) = self.args.priority_gas_price {
                            fees.max_priority_fee_per_gas = priority_gas_price.to();
                        }

                        (None, Some(fees))
                    }
                };

                // Iterate through transactions, matching the `from` field with the associated
                // wallet. Then send the transaction. Panics if we find a unknown `from`
                let transactions = sequence
                    .transactions
                    .iter()
                    .skip(already_broadcasted)
                    .map(|tx_with_metadata| {
                        let is_fixed_gas_limit = tx_with_metadata.is_fixed_gas_limit;

                        let kind = match tx_with_metadata.tx().clone() {
                            TransactionMaybeSigned::Signed { tx, .. } => {
                                SendTransactionKind::Signed(tx)
                            }
                            TransactionMaybeSigned::Unsigned(mut tx) => {
                                let from = tx.from.expect("No sender for onchain transaction!");

                                tx.set_chain_id(sequence.chain);

                                // Set TxKind::Create explicitly to satisfy `check_reqd_fields` in
                                // alloy
                                if tx.to.is_none() {
                                    tx.set_create();
                                }

                                if let Some(gas_price) = gas_price {
                                    tx.set_gas_price(gas_price);
                                } else {
                                    let eip1559_fees = eip1559_fees.expect("was set above");
                                    tx.set_max_priority_fee_per_gas(
                                        eip1559_fees.max_priority_fee_per_gas,
                                    );
                                    tx.set_max_fee_per_gas(eip1559_fees.max_fee_per_gas);
                                }

                                send_kind.for_sender(&from, tx)?
                            }
                        };

                        Ok((kind, is_fixed_gas_limit))
                    })
                    .collect::<Result<Vec<_>>>()?;

                let estimate_via_rpc =
                    has_different_gas_calc(sequence.chain) || self.args.skip_simulation;

                // We only wait for a transaction receipt before sending the next transaction, if
                // there is more than one signer. There would be no way of assuring
                // their order otherwise.
                // Or if the chain does not support batched transactions (eg. Arbitrum).
                // Or if we need to invoke eth_estimateGas before sending transactions.
                let sequential_broadcast = estimate_via_rpc ||
                    self.args.slow ||
                    required_addresses.len() != 1 ||
                    !has_batch_support(sequence.chain);

                // We send transactions and wait for receipts in batches.
                let batch_size = if sequential_broadcast { 1 } else { self.args.batch_size };
                let mut index = already_broadcasted;

                for (batch_number, batch) in transactions.chunks(batch_size).enumerate() {
                    let mut pending_transactions = vec![];

                    seq_progress.inner.write().set_status(&format!(
                        "Sending transactions [{} - {}]",
                        batch_number * batch_size,
                        batch_number * batch_size + std::cmp::min(batch_size, batch.len()) - 1
                    ));
                    for (kind, is_fixed_gas_limit) in batch {
                        let fut = send_transaction(
                            provider.clone(),
                            kind.clone(),
                            sequential_broadcast,
                            *is_fixed_gas_limit,
                            estimate_via_rpc,
                            self.args.gas_estimate_multiplier,
                        );
                        pending_transactions.push(fut);
                    }

                    if !pending_transactions.is_empty() {
                        let mut buffer = futures::stream::iter(pending_transactions).buffered(7);

                        while let Some(tx_hash) = buffer.next().await {
                            let tx_hash = tx_hash.wrap_err("Failed to send transaction")?;
                            sequence.add_pending(index, tx_hash);

                            // Checkpoint save
                            self.sequence.save(true, false)?;
                            sequence = self.sequence.sequences_mut().get_mut(i).unwrap();

                            seq_progress.inner.write().tx_sent(tx_hash);
                            index += 1;
                        }

                        // Checkpoint save
                        self.sequence.save(true, false)?;
                        sequence = self.sequence.sequences_mut().get_mut(i).unwrap();

                        progress
                            .wait_for_pending(
                                i,
                                sequence,
                                &provider,
                                self.script_config.config.transaction_timeout,
                            )
                            .await?
                    }
                    // Checkpoint save
                    self.sequence.save(true, false)?;
                    sequence = self.sequence.sequences_mut().get_mut(i).unwrap();
                }
            }

            let (total_gas, total_gas_price, total_paid) =
                sequence.receipts.iter().fold((0, 0, 0), |acc, receipt| {
                    let gas_used = receipt.gas_used;
                    let gas_price = receipt.effective_gas_price as u64;
                    (acc.0 + gas_used, acc.1 + gas_price, acc.2 + gas_used * gas_price)
                });
            let paid = format_units(total_paid, 18).unwrap_or_else(|_| "N/A".to_string());
            let avg_gas_price = format_units(total_gas_price / sequence.receipts.len() as u64, 9)
                .unwrap_or_else(|_| "N/A".to_string());

            seq_progress.inner.write().set_status(&format!(
                "Total Paid: {} ETH ({} gas * avg {} gwei)\n",
                paid.trim_end_matches('0'),
                total_gas,
                avg_gas_price.trim_end_matches('0').trim_end_matches('.')
            ));
            seq_progress.inner.write().finish();
        }

        if !shell::is_json() {
            sh_println!("\n\n==========================")?;
            sh_println!("\nONCHAIN EXECUTION COMPLETE & SUCCESSFUL.")?;
        }

        Ok(BroadcastedState {
            args: self.args,
            script_config: self.script_config,
            build_data: self.build_data,
            sequence: self.sequence,
        })
    }

    pub fn verify_preflight_check(&self) -> Result<()> {
        for sequence in self.sequence.sequences() {
            if self.args.verifier.verifier == VerificationProviderType::Etherscan &&
                self.script_config
                    .config
                    .get_etherscan_api_key(Some(sequence.chain.into()))
                    .is_none()
            {
                eyre::bail!("Missing etherscan key for chain {}", sequence.chain);
            }
        }

        Ok(())
    }
}
