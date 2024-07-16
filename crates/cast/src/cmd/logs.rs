use crate::Cast;
use alloy_dyn_abi::{DynSolType, DynSolValue, Specifier};
use alloy_json_abi::Event;
use alloy_network::AnyNetwork;
use alloy_primitives::{hex::FromHex, Address, B256};
use alloy_rpc_types::{BlockId, BlockNumberOrTag, Filter, FilterBlockOption, FilterSet, Topic};
use clap::Parser;
use eyre::Result;
use foundry_cli::{opts::EthereumOpts, utils};
use foundry_common::ens::NameOrAddress;
use foundry_config::Config;
use itertools::Itertools;
use std::{io, str::FromStr};

/// CLI arguments for `cast logs`.
#[derive(Debug, Parser)]
pub struct LogsArgs {
    /// The block height to start query at.
    ///
    /// Can also be the tags earliest, finalized, safe, latest, or pending.
    #[arg(long)]
    from_block: Option<BlockId>,

    /// The block height to stop query at.
    ///
    /// Can also be the tags earliest, finalized, safe, latest, or pending.
    #[arg(long)]
    to_block: Option<BlockId>,

    /// The contract address to filter on.
    #[arg(
        long,
        value_parser = NameOrAddress::from_str
    )]
    address: Option<NameOrAddress>,

    /// The signature of the event to filter logs by which will be converted to the first topic or
    /// a topic to filter on.
    #[arg(value_name = "SIG_OR_TOPIC")]
    sig_or_topic: Option<String>,

    /// If used with a signature, the indexed fields of the event to filter by. Otherwise, the
    /// remaining topics of the filter.
    #[arg(value_name = "TOPICS_OR_ARGS")]
    topics_or_args: Vec<String>,

    /// If the RPC type and endpoints supports `eth_subscribe` stream logs instead of printing and
    /// exiting. Will continue until interrupted or TO_BLOCK is reached.
    #[arg(long)]
    subscribe: bool,

    /// Print the logs as JSON.s
    #[arg(long, short, help_heading = "Display options")]
    json: bool,

    #[command(flatten)]
    eth: EthereumOpts,
}

impl LogsArgs {
    pub async fn run(self) -> Result<()> {
        let Self {
            from_block,
            to_block,
            address,
            sig_or_topic,
            topics_or_args,
            subscribe,
            json,
            eth,
        } = self;

        let config = Config::from(&eth);
        let provider = utils::get_provider(&config)?;

        let cast = Cast::new(&provider);

        let address = match address {
            Some(address) => Some(address.resolve(&provider).await?),
            None => None,
        };

        let from_block =
            cast.convert_block_number(Some(from_block.unwrap_or_else(BlockId::earliest))).await?;
        let to_block =
            cast.convert_block_number(Some(to_block.unwrap_or_else(BlockId::latest))).await?;

        let filter = build_filter(from_block, to_block, address, sig_or_topic, topics_or_args)?;

        if !subscribe {
            let logs = cast.filter_logs(filter, json).await?;

            println!("{logs}");

            return Ok(())
        }

        // FIXME: this is a hotfix for <https://github.com/foundry-rs/foundry/issues/7682>
        //  currently the alloy `eth_subscribe` impl does not work with all transports, so we use
        // the builtin transport here for now
        let url = config.get_rpc_url_or_localhost_http()?;
        let provider = alloy_provider::ProviderBuilder::<_, _, AnyNetwork>::default()
            .on_builtin(url.as_ref())
            .await?;
        let cast = Cast::new(&provider);
        let mut stdout = io::stdout();
        cast.subscribe(filter, &mut stdout, json).await?;

        Ok(())
    }
}

/// Builds a Filter by first trying to parse the `sig_or_topic` as an event signature. If
/// successful, `topics_or_args` is parsed as indexed inputs and converted to topics. Otherwise,
/// `sig_or_topic` is prepended to `topics_or_args` and used as raw topics.
fn build_filter(
    from_block: Option<BlockNumberOrTag>,
    to_block: Option<BlockNumberOrTag>,
    address: Option<Address>,
    sig_or_topic: Option<String>,
    topics_or_args: Vec<String>,
) -> Result<Filter, eyre::Error> {
    let block_option = FilterBlockOption::Range { from_block, to_block };
    let filter = match sig_or_topic {
        // Try and parse the signature as an event signature
        Some(sig_or_topic) => match foundry_common::abi::get_event(sig_or_topic.as_str()) {
            Ok(event) => build_filter_event_sig(event, topics_or_args)?,
            Err(_) => {
                let topics = [vec![sig_or_topic], topics_or_args].concat();
                build_filter_topics(topics)?
            }
        },
        None => Filter::default(),
    };

    let mut filter = filter.select(block_option);

    if let Some(address) = address {
        filter = filter.address(address)
    }

    Ok(filter)
}

/// Creates a [Filter] from the given event signature and arguments.
fn build_filter_event_sig(event: Event, args: Vec<String>) -> Result<Filter, eyre::Error> {
    let args = args.iter().map(|arg| arg.as_str()).collect::<Vec<_>>();

    // Match the args to indexed inputs. Enumerate so that the ordering can be restored
    // when merging the inputs with arguments and without arguments
    let (with_args, without_args): (Vec<_>, Vec<_>) = event
        .inputs
        .iter()
        .zip(args)
        .filter(|(input, _)| input.indexed)
        .map(|(input, arg)| {
            let kind = input.resolve()?;
            Ok((kind, arg))
        })
        .collect::<Result<Vec<(DynSolType, &str)>>>()?
        .into_iter()
        .enumerate()
        .partition(|(_, (_, arg))| !arg.is_empty());

    // Only parse the inputs with arguments
    let indexed_tokens = with_args
        .iter()
        .map(|(_, (kind, arg))| kind.coerce_str(arg))
        .collect::<Result<Vec<DynSolValue>, _>>()?;

    // Merge the inputs restoring the original ordering
    let mut topics = with_args
        .into_iter()
        .zip(indexed_tokens)
        .map(|((i, _), t)| (i, Some(t)))
        .chain(without_args.into_iter().map(|(i, _)| (i, None)))
        .sorted_by(|(i1, _), (i2, _)| i1.cmp(i2))
        .map(|(_, token)| {
            token
                .map(|token| Topic::from(B256::from_slice(token.abi_encode().as_slice())))
                .unwrap_or(Topic::default())
        })
        .collect::<Vec<Topic>>();

    topics.resize(3, Topic::default());

    let filter = Filter::new()
        .event_signature(event.selector())
        .topic1(topics[0].clone())
        .topic2(topics[1].clone())
        .topic3(topics[2].clone());

    Ok(filter)
}

/// Creates a [Filter] from raw topic hashes.
fn build_filter_topics(topics: Vec<String>) -> Result<Filter, eyre::Error> {
    let mut topics = topics
        .into_iter()
        .map(|topic| {
            if topic.is_empty() {
                Ok(Topic::default())
            } else {
                Ok(Topic::from(B256::from_hex(topic.as_str())?))
            }
        })
        .collect::<Result<Vec<FilterSet<_>>>>()?;

    topics.resize(4, Topic::default());

    let filter = Filter::new()
        .event_signature(topics[0].clone())
        .topic1(topics[1].clone())
        .topic2(topics[2].clone())
        .topic3(topics[3].clone());

    Ok(filter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{U160, U256};
    use alloy_rpc_types::ValueOrArray;

    const ADDRESS: &str = "0x4D1A2e2bB4F88F0250f26Ffff098B0b30B26BF38";
    const TRANSFER_SIG: &str = "Transfer(address indexed,address indexed,uint256)";
    const TRANSFER_TOPIC: &str =
        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

    #[test]
    fn test_build_filter_basic() {
        let from_block = Some(BlockNumberOrTag::from(1337));
        let to_block = Some(BlockNumberOrTag::Latest);
        let address = Address::from_str(ADDRESS).ok();
        let expected = Filter {
            block_option: FilterBlockOption::Range { from_block, to_block },
            address: ValueOrArray::Value(address.unwrap()).into(),
            topics: [vec![].into(), vec![].into(), vec![].into(), vec![].into()],
        };
        let filter = build_filter(from_block, to_block, address, None, vec![]).unwrap();
        assert_eq!(filter, expected)
    }

    #[test]
    fn test_build_filter_sig() {
        let expected = Filter {
            block_option: FilterBlockOption::Range { from_block: None, to_block: None },
            address: vec![].into(),
            topics: [
                B256::from_str(TRANSFER_TOPIC).unwrap().into(),
                vec![].into(),
                vec![].into(),
                vec![].into(),
            ],
        };
        let filter =
            build_filter(None, None, None, Some(TRANSFER_SIG.to_string()), vec![]).unwrap();
        assert_eq!(filter, expected)
    }

    #[test]
    fn test_build_filter_mismatch() {
        let expected = Filter {
            block_option: FilterBlockOption::Range { from_block: None, to_block: None },
            address: vec![].into(),
            topics: [
                B256::from_str(TRANSFER_TOPIC).unwrap().into(),
                vec![].into(),
                vec![].into(),
                vec![].into(),
            ],
        };
        let filter = build_filter(
            None,
            None,
            None,
            Some("Swap(address indexed from, address indexed to, uint256 value)".to_string()), // Change signature, should result in error
            vec![],
        )
        .unwrap();
        assert_ne!(filter, expected)
    }

    #[test]
    fn test_build_filter_sig_with_arguments() {
        let addr = Address::from_str(ADDRESS).unwrap();
        let addr = U256::from(U160::from_be_bytes(addr.0 .0));
        let expected = Filter {
            block_option: FilterBlockOption::Range { from_block: None, to_block: None },
            address: vec![].into(),
            topics: [
                B256::from_str(TRANSFER_TOPIC).unwrap().into(),
                addr.into(),
                vec![].into(),
                vec![].into(),
            ],
        };
        let filter = build_filter(
            None,
            None,
            None,
            Some(TRANSFER_SIG.to_string()),
            vec![ADDRESS.to_string()],
        )
        .unwrap();
        assert_eq!(filter, expected)
    }

    #[test]
    fn test_build_filter_sig_with_skipped_arguments() {
        let addr = Address::from_str(ADDRESS).unwrap();
        let addr = U256::from(U160::from_be_bytes(addr.0 .0));
        let expected = Filter {
            block_option: FilterBlockOption::Range { from_block: None, to_block: None },
            address: vec![].into(),
            topics: [
                vec![B256::from_str(TRANSFER_TOPIC).unwrap()].into(),
                vec![].into(),
                addr.into(),
                vec![].into(),
            ],
        };
        let filter = build_filter(
            None,
            None,
            None,
            Some(TRANSFER_SIG.to_string()),
            vec![String::new(), ADDRESS.to_string()],
        )
        .unwrap();
        assert_eq!(filter, expected)
    }

    #[test]
    fn test_build_filter_with_topics() {
        let expected = Filter {
            block_option: FilterBlockOption::Range { from_block: None, to_block: None },
            address: vec![].into(),
            topics: [
                vec![B256::from_str(TRANSFER_TOPIC).unwrap()].into(),
                vec![B256::from_str(TRANSFER_TOPIC).unwrap()].into(),
                vec![].into(),
                vec![].into(),
            ],
        };
        let filter = build_filter(
            None,
            None,
            None,
            Some(TRANSFER_TOPIC.to_string()),
            vec![TRANSFER_TOPIC.to_string()],
        )
        .unwrap();

        assert_eq!(filter, expected)
    }

    #[test]
    fn test_build_filter_with_skipped_topic() {
        let expected = Filter {
            block_option: FilterBlockOption::Range { from_block: None, to_block: None },
            address: vec![].into(),
            topics: [
                vec![B256::from_str(TRANSFER_TOPIC).unwrap()].into(),
                vec![].into(),
                vec![B256::from_str(TRANSFER_TOPIC).unwrap()].into(),
                vec![].into(),
            ],
        };
        let filter = build_filter(
            None,
            None,
            None,
            Some(TRANSFER_TOPIC.to_string()),
            vec![String::new(), TRANSFER_TOPIC.to_string()],
        )
        .unwrap();

        assert_eq!(filter, expected)
    }

    #[test]
    fn test_build_filter_sig_with_mismatched_argument() {
        let err = build_filter(
            None,
            None,
            None,
            Some(TRANSFER_SIG.to_string()),
            vec!["1234".to_string()],
        )
        .err()
        .unwrap()
        .to_string();

        assert_eq!(err, "parser error:\n1234\n^\nInvalid string length");
    }

    #[test]
    fn test_build_filter_with_invalid_sig_or_topic() {
        let err = build_filter(None, None, None, Some("asdasdasd".to_string()), vec![])
            .err()
            .unwrap()
            .to_string();

        assert_eq!(err, "Odd number of digits");
    }

    #[test]
    fn test_build_filter_with_invalid_sig_or_topic_hex() {
        let err = build_filter(None, None, None, Some(ADDRESS.to_string()), vec![])
            .err()
            .unwrap()
            .to_string();

        assert_eq!(err, "Invalid string length");
    }

    #[test]
    fn test_build_filter_with_invalid_topic() {
        let err = build_filter(
            None,
            None,
            None,
            Some(TRANSFER_TOPIC.to_string()),
            vec!["1234".to_string()],
        )
        .err()
        .unwrap()
        .to_string();

        assert_eq!(err, "Invalid string length");
    }
}
