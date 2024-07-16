use crate::Cast;
use alloy_provider::Provider;
use clap::Parser;
use eyre::Result;
use foundry_cli::{opts::RpcOpts, utils};
use foundry_config::Config;
use futures::join;

/// CLI arguments for `cast find-block`.
#[derive(Clone, Debug, Parser)]
pub struct FindBlockArgs {
    /// The UNIX timestamp to search for, in seconds.
    timestamp: u64,

    #[command(flatten)]
    rpc: RpcOpts,
}

impl FindBlockArgs {
    pub async fn run(self) -> Result<()> {
        let Self { timestamp, rpc } = self;

        let ts_target = timestamp;
        let config = Config::from(&rpc);
        let provider = utils::get_provider(&config)?;

        let last_block_num = provider.get_block_number().await?;
        let cast_provider = Cast::new(provider);

        let res = join!(cast_provider.timestamp(last_block_num), cast_provider.timestamp(1));
        let ts_block_latest: u64 = res.0?.to();
        let ts_block_1: u64 = res.1?.to();

        let block_num = if ts_block_latest < ts_target {
            // If the most recent block's timestamp is below the target, return it
            last_block_num
        } else if ts_block_1 > ts_target {
            // If the target timestamp is below block 1's timestamp, return that
            1
        } else {
            // Otherwise, find the block that is closest to the timestamp
            let mut low_block = 1_u64; // block 0 has a timestamp of 0: https://github.com/ethereum/go-ethereum/issues/17042#issuecomment-559414137
            let mut high_block = last_block_num;
            let mut matching_block = None;
            while high_block > low_block && matching_block.is_none() {
                // Get timestamp of middle block (this approach approach to avoids overflow)
                let high_minus_low_over_2 = high_block
                    .checked_sub(low_block)
                    .ok_or_else(|| eyre::eyre!("unexpected underflow"))
                    .unwrap()
                    .checked_div(2_u64)
                    .unwrap();
                let mid_block = high_block.checked_sub(high_minus_low_over_2).unwrap();
                let ts_mid_block = cast_provider.timestamp(mid_block).await?.to::<u64>();

                // Check if we've found a match or should keep searching
                if ts_mid_block == ts_target {
                    matching_block = Some(mid_block)
                } else if high_block.checked_sub(low_block).unwrap() == 1_u64 {
                    // The target timestamp is in between these blocks. This rounds to the
                    // highest block if timestamp is equidistant between blocks
                    let res = join!(
                        cast_provider.timestamp(high_block),
                        cast_provider.timestamp(low_block)
                    );
                    let ts_high: u64 = res.0.unwrap().to();
                    let ts_low: u64 = res.1.unwrap().to();
                    let high_diff = ts_high.checked_sub(ts_target).unwrap();
                    let low_diff = ts_target.checked_sub(ts_low).unwrap();
                    let is_low = low_diff < high_diff;
                    matching_block = if is_low { Some(low_block) } else { Some(high_block) }
                } else if ts_mid_block < ts_target {
                    low_block = mid_block;
                } else {
                    high_block = mid_block;
                }
            }
            matching_block.unwrap_or(low_block)
        };
        println!("{block_num}");

        Ok(())
    }
}
