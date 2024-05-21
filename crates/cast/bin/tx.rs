use alloy_consensus::{SidecarBuilder, SimpleCoder};
use alloy_network::{AnyNetwork, TransactionBuilder};
use alloy_primitives::{Address, TxKind};
use alloy_provider::Provider;
use alloy_rpc_types::{TransactionRequest, WithOtherFields};
use alloy_transport::Transport;
use eyre::Result;
use foundry_cli::{
    opts::TransactionOpts,
    utils::{self, parse_function_args},
};
use foundry_common::ens::NameOrAddress;
use foundry_config::{Chain, Config};

/// Prevents a misconfigured hwlib from sending a transaction that defies user-specified --from
pub fn validate_from_address(
    specified_from: Option<Address>,
    signer_address: Address,
) -> Result<()> {
    if let Some(specified_from) = specified_from {
        if specified_from != signer_address {
            eyre::bail!(
                "\
The specified sender via CLI/env vars does not match the sender configured via
the hardware wallet's HD Path.
Please use the `--hd-path <PATH>` parameter to specify the BIP32 Path which
corresponds to the sender, or let foundry automatically detect it by not specifying any sender address."
            )
        }
    }
    Ok(())
}

/// Ensures the transaction is either a contract deployment or a recipient address is specified
pub async fn resolve_tx_kind<P: Provider<T, AnyNetwork>, T: Transport + Clone>(
    provider: &P,
    code: &Option<String>,
    to: &Option<NameOrAddress>,
) -> Result<TxKind> {
    if code.is_some() {
        Ok(TxKind::Create)
    } else if let Some(to) = to {
        Ok(TxKind::Call(to.resolve(provider).await?))
    } else {
        eyre::bail!("Must specify a recipient address or contract code to deploy");
    }
}

/// Initial state.
pub struct InitState;

/// State with known [TxKind].
pub struct TxKindState {
    kind: TxKind,
}

/// State with known input for the transaction.
pub struct InputState {
    kind: TxKind,
    input: Vec<u8>,
}

pub struct CastTxBuilder<T, P, S> {
    provider: P,
    tx: WithOtherFields<TransactionRequest>,
    legacy: bool,
    blob: bool,
    chain: Chain,
    etherscan_api_key: Option<String>,
    state: S,
    _t: std::marker::PhantomData<T>,
}

impl<T, P> CastTxBuilder<T, P, InitState>
where
    P: Provider<T, AnyNetwork>,
    T: Transport + Clone,
{
    pub async fn new(provider: P, tx_opts: TransactionOpts, config: &Config) -> Result<Self> {
        let mut tx = WithOtherFields::<TransactionRequest>::default();

        let chain = utils::get_chain(config.chain, &provider).await?;
        let etherscan_api_key = config.get_etherscan_api_key(Some(chain));

        if let Some(gas_limit) = tx_opts.gas_limit {
            tx.set_gas_limit(gas_limit.to());
        }

        if let Some(value) = tx_opts.value {
            tx.set_value(value);
        }

        if let Some(gas_price) = tx_opts.gas_price {
            if tx_opts.legacy {
                tx.set_gas_price(gas_price.to());
            } else {
                tx.set_max_fee_per_gas(gas_price.to());
            }
        }

        if !tx_opts.legacy {
            if let Some(priority_fee) = tx_opts.priority_gas_price {
                tx.set_max_priority_fee_per_gas(priority_fee.to());
            }
        }

        if let Some(max_blob_fee) = tx_opts.blob_gas_price {
            tx.set_max_fee_per_blob_gas(max_blob_fee.to())
        }

        if let Some(nonce) = tx_opts.nonce {
            tx.set_nonce(nonce.to());
        }

        Ok(Self {
            provider,
            tx,
            legacy: tx_opts.legacy || chain.is_legacy(),
            blob: tx_opts.blob,
            chain,
            etherscan_api_key,
            state: InitState,
            _t: std::marker::PhantomData,
        })
    }

    pub fn with_tx_kind(self, kind: TxKind) -> CastTxBuilder<T, P, TxKindState> {
        CastTxBuilder {
            provider: self.provider,
            tx: self.tx,
            legacy: self.legacy,
            blob: self.blob,
            chain: self.chain,
            etherscan_api_key: self.etherscan_api_key,
            state: TxKindState { kind },
            _t: self._t,
        }
    }
}

impl<T, P> CastTxBuilder<T, P, TxKindState>
where
    P: Provider<T, AnyNetwork>,
    T: Transport + Clone,
{
    pub async fn with_code_sig_and_args(
        self,
        code: Option<String>,
        sig: Option<String>,
        args: Vec<String>,
    ) -> Result<CastTxBuilder<T, P, InputState>> {
        let mut args = if let Some(sig) = sig {
            parse_function_args(
                &sig,
                args,
                self.state.kind.to().cloned(),
                self.chain,
                &self.provider,
                self.etherscan_api_key.as_deref(),
            )
            .await?
            .0
        } else {
            Vec::new()
        };

        let input = if let Some(code) = code {
            let mut code = hex::decode(code)?;
            code.append(&mut args);
            code
        } else {
            args
        };

        Ok(CastTxBuilder {
            provider: self.provider,
            tx: self.tx,
            legacy: self.legacy,
            blob: self.blob,
            chain: self.chain,
            etherscan_api_key: self.etherscan_api_key,
            state: InputState { kind: self.state.kind, input },
            _t: self._t,
        })
    }
}

impl<T, P> CastTxBuilder<T, P, InputState>
where
    P: Provider<T, AnyNetwork>,
    T: Transport + Clone,
{
    pub async fn build(
        mut self,
        from: impl Into<NameOrAddress>,
    ) -> Result<WithOtherFields<TransactionRequest>> {
        let from = from.into().resolve(&self.provider).await?;

        self.tx.set_kind(self.state.kind);
        self.tx.set_input(self.state.input);
        self.tx.set_from(from);
        self.tx.set_chain_id(self.chain.id());

        if self.legacy && self.tx.gas_price.is_none() {
            self.tx.gas_price = Some(self.provider.get_gas_price().await?);
        }

        if self.blob && self.tx.max_fee_per_blob_gas.is_none() {
            self.tx.max_fee_per_blob_gas = Some(self.provider.get_blob_base_fee().await?)
        }

        if !self.legacy &&
            (self.tx.max_fee_per_gas.is_none() || self.tx.max_priority_fee_per_gas.is_none())
        {
            let estimate = self.provider.estimate_eip1559_fees(None).await?;

            if !self.legacy {
                if self.tx.max_fee_per_gas.is_none() {
                    self.tx.max_fee_per_gas = Some(estimate.max_fee_per_gas);
                }

                if self.tx.max_priority_fee_per_gas.is_none() {
                    self.tx.max_priority_fee_per_gas = Some(estimate.max_priority_fee_per_gas);
                }
            }
        }

        if self.tx.gas.is_none() {
            self.tx.gas = Some(self.provider.estimate_gas(&self.tx).await?);
        }

        if self.tx.nonce.is_none() {
            self.tx.nonce = Some(self.provider.get_transaction_count(from).await?);
        }

        Ok(self.tx)
    }
}

impl<T, P, S> CastTxBuilder<T, P, S>
where
    P: Provider<T, AnyNetwork>,
    T: Transport + Clone,
{
    pub fn with_blob_data(mut self, blob_data: Option<Vec<u8>>) -> Result<Self> {
        let Some(blob_data) = blob_data else { return Ok(self) };

        let mut coder = SidecarBuilder::<SimpleCoder>::default();
        coder.ingest(&blob_data);
        let sidecar = coder.build()?;

        self.tx.set_blob_sidecar(sidecar);
        self.tx.populate_blob_hashes();

        Ok(self)
    }
}
