//! ABI related helper functions
use alloy_json_abi::{Function, Event};
use alloy_dyn_abi::{DynSolValue, DynSolType, JsonAbiExt, FunctionExt};
use alloy_primitives::{Address, I256, U256, Log, hex};
use ethers_core::types::Chain;
use foundry_block_explorers::{contract::ContractMetadata, errors::EtherscanError, Client};
use eyre::{ContextCompat, Result, WrapErr};
use std::{future::Future, pin::Pin, str::FromStr};
use yansi::Paint;

use crate::calc::to_exponential_notation;

/// Given a function and a vector of string arguments, it proceeds to convert the args to ethabi
/// Tokens and then ABI encode them.
pub fn encode_function_args(func: &Function, args: &[impl AsRef<str>]) -> Result<Vec<u8>> {
    let params = func
        .inputs
        .iter()
        .zip(args)
        .map(|(input, arg)| (&input.ty, arg.as_ref()))
        .collect::<Vec<_>>();
    let args = params.iter().map(|(_, arg)| DynSolValue::from(arg.to_owned().to_string())).collect::<Vec<_>>();
    Ok(func.abi_encode_input(&args)?)
}

/// Decodes the calldata of the function
///
/// # Panics
///
/// If the `sig` is an invalid function signature
pub fn abi_decode_calldata(sig: &str, calldata: &str, input: bool, fn_selector: bool) -> Result<Vec<DynSolValue>> {
    let func = Function::parse(sig)?;
    let calldata = hex::decode(calldata)?;
    let res = if input {
        // If function selector is prefixed in "calldata", remove it (first 4 bytes)
        if fn_selector {
            func.abi_decode_input(&calldata[4..], false)?
        } else {
            func.abi_decode_input(&calldata, false)?
        }
    } else {
        func.abi_decode_output(&calldata, false)?
    };

    // in case the decoding worked but nothing was decoded
    if res.is_empty() {
        eyre::bail!("no data was decoded")
    }

    Ok(res)
}

/// Parses string input as Token against the expected ParamType
pub fn parse_tokens<'a, I: IntoIterator<Item = (&'a ParamType, &'a str)>>(
    params: I,
    lenient: bool,
) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();

    for (param, value) in params.into_iter() {
        let mut token = if lenient {
            LenientTokenizer::tokenize(param, value)
        } else {
            StrictTokenizer::tokenize(param, value)
        };
        if token.is_err() && value.starts_with("0x") {
            match param {
                ParamType::FixedBytes(32) => {
                    if value.len() < 66 {
                        let padded_value = [value, &"0".repeat(66 - value.len())].concat();
                        token = if lenient {
                            LenientTokenizer::tokenize(param, &padded_value)
                        } else {
                            StrictTokenizer::tokenize(param, &padded_value)
                        };
                    }
                }
                ParamType::Uint(_) => {
                    // try again if value is hex
                    if let Ok(value) = U256::from_str(value).map(|v| v.to_string()) {
                        token = if lenient {
                            LenientTokenizer::tokenize(param, &value)
                        } else {
                            StrictTokenizer::tokenize(param, &value)
                        };
                    }
                }
                // TODO: Not sure what to do here. Put the no effect in for now, but that is not
                // ideal. We could attempt massage for every value type?
                _ => {}
            }
        }

        let token = token.map(sanitize_token).wrap_err_with(|| {
            format!("Failed to parse `{value}`, expected value of type: {param}")
        })?;
        tokens.push(token);
    }
    Ok(tokens)
}

/// Cleans up potential shortcomings of the ethabi Tokenizer.
///
/// For example: parsing a string array with a single empty string: `[""]`, is returned as
///
/// ```text
///     [
///        String(
///            "\"\"",
///        ),
///    ],
/// ```
///
/// But should just be
///
/// ```text
///     [
///        String(
///            "",
///        ),
///    ],
/// ```
///
/// This will handle this edge case
pub fn sanitize_token(token: DynSolValue) -> DynSolValue {
    match token {
        DynSolValue::Array(tokens) => {
            let mut sanitized = Vec::with_capacity(tokens.len());
            for token in tokens {
                let token = match token {
                    DynSolValue::String(val) => {
                        let val = match val.as_str() {
                            // this is supposed to be an empty string
                            "\"\"" | "''" => "".to_string(),
                            _ => val,
                        };
                        DynSolValue::String(val)
                    }
                    _ => sanitize_token(token),
                };
                sanitized.push(token)
            }
            DynSolValue::Array(sanitized)
        }
        _ => token,
    }
}

/// Pretty print a slice of tokens.
pub fn format_tokens(tokens: &[DynSolValue]) -> impl Iterator<Item = String> + '_ {
    tokens.iter().map(format_token)
}

/// Gets pretty print strings for tokens
pub fn format_token(param: &DynSolValue) -> String {
    match param {
        DynSolValue::Address(addr) => addr.to_checksum(None),
        DynSolValue::FixedBytes(bytes, _) => hex::encode_prefixed(bytes),
        DynSolValue::Bytes(bytes) => hex::encode_prefixed(bytes),
        DynSolValue::Int(num, _) => format!("{}", num),
        DynSolValue::Uint(num, _) => format_uint_with_exponential_notation_hint(*num),
        DynSolValue::Bool(b) => format!("{b}"),
        DynSolValue::String(s) => s.to_string(),
        DynSolValue::FixedArray(tokens) => {
            let string = tokens.iter().map(format_token).collect::<Vec<String>>().join(", ");
            format!("[{string}]")
        }
        DynSolValue::Array(tokens) => {
            let string = tokens.iter().map(format_token).collect::<Vec<String>>().join(", ");
            format!("[{string}]")
        }
        DynSolValue::Tuple(tokens) => {
            let string = tokens.iter().map(format_token).collect::<Vec<String>>().join(", ");
            format!("({string})")
        },
        DynSolValue::Function(_) => unimplemented!()
    }
}

/// Gets pretty print strings for tokens, without adding
/// exponential notation hints for large numbers (e.g. [1e7] for 10000000)
pub fn format_token_raw(param: &DynSolValue) -> String {
    match param {
        DynSolValue::Uint(num, _) => format!("{}", num),
        DynSolValue::FixedArray(tokens) | DynSolValue::Array(tokens) => {
            let string = tokens.iter().map(format_token_raw).collect::<Vec<String>>().join(", ");
            format!("[{string}]")
        }
        DynSolValue::Tuple(tokens) => {
            let string = tokens.iter().map(format_token_raw).collect::<Vec<String>>().join(", ");
            format!("({string})")
        }
        _ => format_token(param),
    }
}

/// Formats a U256 number to string, adding an exponential notation _hint_ if it
/// is larger than `10_000`, with a precision of `4` figures, and trimming the
/// trailing zeros.
///
/// Examples:
///
/// ```text
///   0 -> "0"
///   1234 -> "1234"
///   1234567890 -> "1234567890 [1.234e9]"
///   1000000000000000000 -> "1000000000000000000 [1e18]"
///   10000000000000000000000 -> "10000000000000000000000 [1e22]"
/// ```
pub fn format_uint_with_exponential_notation_hint(num: U256) -> String {
    if num.lt(&U256::from(10_000)) {
        return num.to_string()
    }

    let exp = to_exponential_notation(num, 4, true);
    format!("{} {}", num, Paint::default(format!("[{}]", exp)).dimmed())
}

/// Helper trait for converting types to Functions. Helpful for allowing the `call`
/// function on the EVM to be generic over `String`, `&str` and `Function`.
pub trait IntoFunction {
    /// Consumes self and produces a function
    ///
    /// # Panic
    ///
    /// This function does not return a Result, so it is expected that the consumer
    /// uses it correctly so that it does not panic.
    fn into(self) -> Function;
}

impl IntoFunction for Function {
    fn into(self) -> Function {
        self
    }
}

impl IntoFunction for String {
    fn into(self) -> Function {
        IntoFunction::into(self.as_str())
    }
}

impl<'a> IntoFunction for &'a str {
    fn into(self) -> Function {
        Function::parse(self).expect("could not parse function")
    }
}

/// Given a function signature string, it tries to parse it as a `Function`
pub fn get_func(sig: &str) -> Result<Function> {
    Ok(match Function::parse(sig) {
        Ok(func) => func,
        Err(err) => {
                // we return the `Function` parse error as this case is more likely
                return Err(err.into())
        }
    })
}

/// Given an event signature string, it tries to parse it as a `Event`
pub fn get_event(sig: &str) -> Result<Event> {
    Ok(Event::parse(sig)?)
}

/// Given an event without indexed parameters and a rawlog, it tries to return the event with the
/// proper indexed parameters. Otherwise, it returns the original event.
pub fn get_indexed_event(mut event: Event, raw_log: &Log) -> Event {
    if !event.anonymous && raw_log.topics().len() > 1 {
        let indexed_params = raw_log.topics().len() - 1;
        let num_inputs = event.inputs.len();
        let num_address_params =
            event.inputs.iter().filter(|p| p.ty == "address").count();

        event.inputs.iter_mut().enumerate().for_each(|(index, param)| {
            if param.name.is_empty() {
                param.name = format!("param{index}");
            }
            if num_inputs == indexed_params ||
                (num_address_params == indexed_params && param.ty == "address")
            {
                param.indexed = true;
            }
        })
    }
    event
}

/// Given a function name, address, and args, tries to parse it as a `Function` by fetching the
/// abi from etherscan. If the address is a proxy, fetches the ABI of the implementation contract.
pub async fn get_func_etherscan(
    function_name: &str,
    contract: Address,
    args: &[String],
    chain: Chain,
    etherscan_api_key: &str,
) -> Result<Function> {
    let client = Client::new(chain, etherscan_api_key)?;
    let source = find_source(client, contract).await?;
    let metadata = source.items.first().wrap_err("etherscan returned empty metadata")?;

    let mut abi = metadata.abi()?;
    let funcs = abi.functions.remove(function_name).unwrap_or_default();

    for func in funcs {
        let res = encode_function_args(&func, args);
        if res.is_ok() {
            return Ok(func)
        }
    }

    Err(eyre::eyre!("Function not found in abi"))
}

/// If the code at `address` is a proxy, recurse until we find the implementation.
pub fn find_source(
    client: Client,
    address: Address,
) -> Pin<Box<dyn Future<Output = Result<ContractMetadata>>>> {
    Box::pin(async move {
        tracing::trace!("find etherscan source for: {:?}", address);
        let source = client.contract_source_code(address).await?;
        let metadata = source.items.first().wrap_err("Etherscan returned no data")?;
        if metadata.proxy == 0 {
            Ok(source)
        } else {
            let implementation = metadata.implementation.unwrap();
            println!(
                "Contract at {address} is a proxy, trying to fetch source at {implementation:?}..."
            );
            match find_source(client, implementation).await {
                impl_source @ Ok(_) => impl_source,
                Err(e) => {
                    let err = EtherscanError::ContractCodeNotVerified(address).to_string();
                    if e.to_string() == err {
                        tracing::error!("{}", err);
                        Ok(source)
                    } else {
                        Err(e)
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_dyn_abi::EventExt;
    use alloy_primitives::B256;

    #[test]
    fn can_sanitize_token() {
        let token =
            Token::Array(LenientTokenizer::tokenize_array("[\"\"]", &ParamType::String).unwrap());
        let sanitized = sanitize_token(token);
        assert_eq!(sanitized, Token::Array(vec![Token::String("".to_string())]));

        let token =
            Token::Array(LenientTokenizer::tokenize_array("['']", &ParamType::String).unwrap());
        let sanitized = sanitize_token(token);
        assert_eq!(sanitized, Token::Array(vec![Token::String("".to_string())]));

        let token = Token::Array(
            LenientTokenizer::tokenize_array("[\"\",\"\"]", &ParamType::String).unwrap(),
        );
        let sanitized = sanitize_token(token);
        assert_eq!(
            sanitized,
            Token::Array(vec![Token::String("".to_string()), Token::String("".to_string())])
        );

        let token =
            Token::Array(LenientTokenizer::tokenize_array("['','']", &ParamType::String).unwrap());
        let sanitized = sanitize_token(token);
        assert_eq!(
            sanitized,
            Token::Array(vec![Token::String("".to_string()), Token::String("".to_string())])
        );
    }

    #[test]
    fn parse_hex_uint_tokens() {
        let param = DynSolType::Uint(256);

        let tokens = parse_tokens(std::iter::once((&param, "100")), true).unwrap();
        assert_eq!(tokens, vec![DynSolValue::Uint(U256::from(100), 256)]);

        let val: U256 = U256::from(100u64);
        let hex_val = format!("0x{val:x}");
        let tokens = parse_tokens(std::iter::once((&param, hex_val.as_str())), true).unwrap();
        assert_eq!(tokens, vec![DynSolValue::Uint(U256::from(100), 256)]);
    }

    #[test]
    fn test_indexed_only_address() {
        let event = get_event("event Ev(address,uint256,address)").unwrap();

        let param0 = B256::random();
        let param1 = vec![3; 32];
        let param2 = B256::random();
        let log = Log::new_unchecked(vec![event.selector(), param0, param2], param1.clone().into());
        let event = get_indexed_event(event, &log);

        assert_eq!(event.inputs.len(), 3);

        // Only the address fields get indexed since total_params > num_indexed_params
        let parsed = event.decode_log(&log, false).unwrap();

        assert_eq!(event.inputs.iter().filter(|param| param.indexed).count(), 2);
        assert_eq!(parsed.body[0], DynSolValue::Address(Address::from_word(param0)));
        assert_eq!(parsed.body[1], DynSolValue::Uint(U256::from_be_bytes([3; 32]), 256));
        assert_eq!(parsed.body[2], DynSolValue::Address(Address::from_word(param2)));
    }

    #[test]
    fn test_indexed_all() {
        let event = get_event("event Ev(address,uint256,address)").unwrap();

        let param0 = B256::random();
        let param1 = vec![3; 32];
        let param2 = B256::random();
        let log = Log::new_unchecked(
            vec![event.selector(), param0, B256::from_slice(&param1), param2],
            vec![].into(),
        );
        let event = get_indexed_event(event, &log);

        assert_eq!(event.inputs.len(), 3);

        // All parameters get indexed since num_indexed_params == total_params
        assert_eq!(event.inputs.iter().filter(|param| param.indexed).count(), 3);
        let parsed = event.decode_log(&log, false).unwrap();

        assert_eq!(parsed.body[0], DynSolValue::Address(Address::from_word(param0)));
        assert_eq!(parsed.body[1], DynSolValue::Uint(U256::from_be_bytes([3; 32]), 256));
        assert_eq!(parsed.body[2], DynSolValue::Address(Address::from_word(param2)));
    }

    #[test]
    fn test_format_token_addr() {
        // copied from testcases in https://github.com/ethereum/EIPs/blob/master/EIPS/eip-55.md
        let eip55 = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";
        assert_eq!(
            format_token(&DynSolValue::Address(Address::from_str(&eip55.to_lowercase()).unwrap())),
            eip55.to_string()
        );

        // copied from testcases in https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1191.md
        let eip1191 = "0xFb6916095cA1Df60bb79ce92cE3EA74c37c5d359";
        assert_ne!(
            format_token(&DynSolValue::Address(Address::from_str(&eip1191.to_lowercase()).unwrap())),
            eip1191.to_string()
        );
    }
}
