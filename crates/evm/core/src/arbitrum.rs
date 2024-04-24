use self::ArbSys::ArbSysCalls;
use crate::backend::{DatabaseError, ForkDB};
use alloy_chains::{Chain, NamedChain};
use alloy_primitives::{address, Address, Bytes, U256};
use alloy_sol_types::{sol, SolInterface, SolValue};
use revm::{
    primitives::{Env, PrecompileResult},
    ContextStatefulPrecompile,
};

pub const ARB_SYS_ADDRESS: Address = address!("0000000000000000000000000000000000000064");

#[derive(Debug, thiserror::Error)]
pub enum ArbitrumError {
    #[error("missing L1 block field in L2 block data")]
    MissingL1BlockField,

    #[error(transparent)]
    Database(#[from] DatabaseError),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

sol! {
    interface ArbSys {
        function arbBlockNumber() external view returns (uint);
    }
}

#[derive(Clone, Debug)]
pub struct ArbSysPrecompile;

impl<DB: revm::Database> ContextStatefulPrecompile<DB> for ArbSysPrecompile {
    fn call(
        &self,
        bytes: &Bytes,
        _: u64,
        evmctx: &mut revm::InnerEvmContext<DB>,
    ) -> PrecompileResult {
        match ArbSysCalls::abi_decode(bytes.as_ref(), false).map_err(|_| {
            revm::precompile::Error::other("failed to decode ArbSys precompile call")
        })? {
            ArbSysCalls::arbBlockNumber(_) => {
                Ok((0, U256::from(evmctx.env.block.number).abi_encode().into()))
            }
            _ => unimplemented!(),
        }
    }
}

pub fn get_block_number(active_fork_db: &ForkDB, env: &Env) -> Result<U256, ArbitrumError> {
    let block = active_fork_db.db.get_full_block(env.block.number.to::<u64>())?;
    let l1_block_number = block
        .other
        .get_deserialized::<U256>("l1BlockNumber")
        .ok_or(ArbitrumError::MissingL1BlockField)??;

    Ok(l1_block_number)
}

pub fn is_arbitrum(chain_id: u64) -> bool {
    let chain = Chain::from(chain_id);
    matches!(
        chain.named(),
        Some(
            NamedChain::Arbitrum |
                NamedChain::ArbitrumGoerli |
                NamedChain::ArbitrumNova |
                NamedChain::ArbitrumTestnet
        )
    )
}
