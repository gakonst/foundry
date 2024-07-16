use crate::Cast;
use clap::Parser;
use eyre::Result;
use foundry_cli::{opts::RpcOpts, utils};
use foundry_config::Config;
use itertools::Itertools;

/// CLI arguments for `cast rpc`.
#[derive(Clone, Debug, Parser)]
pub struct RpcArgs {
    /// RPC method name
    method: String,

    /// RPC parameters
    ///
    /// Interpreted as JSON:
    ///
    /// cast rpc eth_getBlockByNumber 0x123 false
    /// => {"method": "eth_getBlockByNumber", "params": ["0x123", false] ... }
    params: Vec<String>,

    /// Send raw JSON parameters
    ///
    /// The first param will be interpreted as a raw JSON array of params.
    /// If no params are given, stdin will be used. For example:
    ///
    /// cast rpc eth_getBlockByNumber '["0x123", false]' --raw
    ///     => {"method": "eth_getBlockByNumber", "params": ["0x123", false] ... }
    #[arg(long, short = 'w')]
    raw: bool,

    #[command(flatten)]
    rpc: RpcOpts,
}

impl RpcArgs {
    pub async fn run(self) -> Result<()> {
        let Self { raw, method, params, rpc } = self;

        let config = Config::from(&rpc);
        let provider = utils::get_provider(&config)?;

        let params = if raw {
            if params.is_empty() {
                serde_json::Deserializer::from_reader(std::io::stdin())
                    .into_iter()
                    .next()
                    .transpose()?
                    .ok_or_else(|| eyre::format_err!("Empty JSON parameters"))?
            } else {
                value_or_string(params.into_iter().join(" "))
            }
        } else {
            serde_json::Value::Array(params.into_iter().map(value_or_string).collect())
        };
        println!("{}", Cast::new(provider).rpc(&method, params).await?);
        Ok(())
    }
}

fn value_or_string(value: String) -> serde_json::Value {
    serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value))
}
