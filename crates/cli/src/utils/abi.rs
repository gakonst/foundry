use alloy_chains::Chain;
use alloy_json_abi::Function;
use alloy_primitives::{hex, Address};
use alloy_provider::{network::AnyNetwork, Provider};
use alloy_transport::Transport;
use eyre::{OptionExt, Result};
use foundry_common::{
    abi::{encode_function_args, get_func, get_func_etherscan},
    ens::NameOrAddress,
};
use futures::future::join_all;

async fn resolve_name_args<T: Transport + Clone, P: Provider<T, AnyNetwork>>(
    args: &[String],
    provider: &P,
) -> Vec<String> {
    join_all(args.iter().map(|arg| async {
        if arg.contains('.') {
            let addr = NameOrAddress::Name(arg.to_string()).resolve(provider).await;
            match addr {
                Ok(addr) => addr.to_string(),
                Err(_) => arg.to_string(),
            }
        } else {
            arg.to_string()
        }
    }))
    .await
}

pub async fn parse_function_args<T: Transport + Clone, P: Provider<T, AnyNetwork>>(
    sig: &str,
    args: Vec<String>,
    to: Option<Address>,
    chain: Chain,
    provider: &P,
    etherscan_api_key: Option<&str>,
) -> Result<(Vec<u8>, Option<Function>)> {
    if sig.trim().is_empty() {
        eyre::bail!("Function signature or calldata must be provided.")
    }

    let args = resolve_name_args(&args, provider).await;

    if let Ok(data) = hex::decode(sig) {
        return Ok((data, None));
    }

    let func = if sig.contains('(') {
        // a regular function signature with parentheses
        get_func(sig)?
    } else {
        let etherscan_api_key = etherscan_api_key.ok_or_eyre(
            "Function signature does not contain parentheses. If you wish to fetch function data from Etherscan, please provide an API key.",
        )?;
        let to = to.ok_or_eyre("A 'to' address must be provided to fetch function data.")?;
        get_func_etherscan(sig, to, &args, chain, etherscan_api_key).await?
    };

    Ok((encode_function_args(&func, &args)?, Some(func)))
}
