use crate::{tx::CastTxBuilder, Cast};
use alloy_primitives::TxKind;
use alloy_rpc_types::BlockId;
use clap::Parser;
use eyre::Result;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils,
};
use foundry_common::ens::NameOrAddress;
use foundry_config::Config;
use std::str::FromStr;

/// CLI arguments for `cast access-list`.
#[derive(Debug, Parser)]
pub struct AccessListArgs {
    /// The destination of the transaction.
    #[arg(
        value_name = "TO",
        value_parser = NameOrAddress::from_str
    )]
    to: Option<NameOrAddress>,

    /// The signature of the function to call.
    #[arg(value_name = "SIG")]
    sig: Option<String>,

    /// The arguments of the function to call.
    #[arg(value_name = "ARGS")]
    args: Vec<String>,

    /// The block height to query at.
    ///
    /// Can also be the tags earliest, finalized, safe, latest, or pending.
    #[arg(long, short = 'B')]
    block: Option<BlockId>,

    /// Print the access list as JSON.
    #[arg(long, short, help_heading = "Display options")]
    json: bool,

    #[command(flatten)]
    tx: TransactionOpts,

    #[command(flatten)]
    eth: EthereumOpts,
}

impl AccessListArgs {
    pub async fn run(self) -> Result<()> {
        let Self { to, sig, args, tx, eth, block, json: to_json } = self;

        let config = Config::from(&eth);
        let provider = utils::get_provider(&config)?;
        let sender = eth.wallet.sender().await;

        let tx_kind = if let Some(to) = to {
            TxKind::Call(to.resolve(&provider).await?)
        } else {
            TxKind::Create
        };

        let (tx, _) = CastTxBuilder::new(&provider, tx, &config)
            .await?
            .with_tx_kind(tx_kind)
            .with_code_sig_and_args(None, sig, args)
            .await?
            .build_raw(sender)
            .await?;

        let cast = Cast::new(&provider);

        let access_list: String = cast.access_list(&tx, block, to_json).await?;

        println!("{access_list}");

        Ok(())
    }
}
