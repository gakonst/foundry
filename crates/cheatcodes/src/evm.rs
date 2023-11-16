//! Implementations of [`Evm`](crate::Group::Evm) cheatcodes.

use crate::{inspector::AddressState, Cheatcode, Cheatcodes, CheatsCtxt, Result, Vm::*};

use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::SolValue;
use ethers_core::utils::{Genesis, GenesisAccount};
use ethers_signers::Signer;
use foundry_common::fs::read_json_file;
use foundry_evm_core::backend::DatabaseExt;
use foundry_utils::types::ToAlloy;
use revm::{
    primitives::{Account, Bytecode, SpecId, KECCAK_EMPTY},
    EVMData,
};
use std::{collections::HashMap, path::Path};

mod fork;
pub(crate) mod mapping;
pub(crate) mod mock;
pub(crate) mod prank;

/// Records storage slots reads and writes.
#[derive(Clone, Debug, Default)]
pub struct RecordAccess {
    /// Storage slots reads.
    pub reads: HashMap<Address, Vec<U256>>,
    /// Storage slots writes.
    pub writes: HashMap<Address, Vec<U256>>,
}

/// Records `deal` cheatcodes
#[derive(Debug, Clone)]
pub struct DealRecord {
    /// Target of the deal.
    pub address: Address,
    /// The balance of the address before deal was applied
    pub old_balance: U256,
    /// Balance after deal was applied
    pub new_balance: U256,
}

impl Cheatcode for addrCall {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { privateKey } = self;
        let wallet = super::utils::parse_wallet(privateKey)?;
        Ok(wallet.address().to_alloy().abi_encode())
    }
}

impl Cheatcode for getNonce_0Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { account } = self;
        get_nonce(ccx, account)
    }
}

impl Cheatcode for loadCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { target, slot } = *self;
        ensure_not_precompile!(&target, ccx);
        ccx.data.journaled_state.load_account(target, ccx.data.db)?;
        let (val, _) = ccx.data.journaled_state.sload(target, slot.into(), ccx.data.db)?;
        Ok(val.abi_encode())
    }
}

impl Cheatcode for loadAllocsCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { pathToAllocsJson } = self;

        let path = Path::new(pathToAllocsJson);
        ensure!(path.exists(), "allocs file does not exist: {pathToAllocsJson}");

        // Let's first assume we're reading a genesis.json file.
        let allocs: HashMap<Address, GenesisAccount> = match read_json_file::<Genesis>(path) {
            Ok(genesis) => genesis.alloc.into_iter().map(|(k, g)| (k.to_alloy(), g)).collect(),
            // If that fails, let's try reading a file with just the genesis accounts.
            Err(_) => read_json_file(path)?,
        };

        // Then, load the allocs into the database.
        ccx.data
            .db
            .load_allocs(&allocs, &mut ccx.data.journaled_state)
            .map(|()| Vec::default())
            .map_err(|e| fmt_err!("failed to load allocs: {e}"))
    }
}

impl Cheatcode for sign_0Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { privateKey, digest } = self;
        super::utils::sign(privateKey, digest, ccx.data.env.cfg.chain_id)
    }
}

impl Cheatcode for recordCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        state.accesses = Some(Default::default());
        Ok(Default::default())
    }
}

impl Cheatcode for accessesCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self { target } = *self;
        let result = state
            .accesses
            .as_mut()
            .map(|accesses| {
                (
                    &accesses.reads.entry(target).or_default()[..],
                    &accesses.writes.entry(target).or_default()[..],
                )
            })
            .unwrap_or_default();
        Ok(result.abi_encode_params())
    }
}

impl Cheatcode for recordLogsCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        state.recorded_logs = Some(Default::default());
        Ok(Default::default())
    }
}

impl Cheatcode for getRecordedLogsCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        Ok(state.recorded_logs.replace(Default::default()).unwrap_or_default().abi_encode())
    }
}

impl Cheatcode for pauseGasMeteringCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        if state.gas_metering.is_none() {
            state.gas_metering = Some(None);
        }
        Ok(Default::default())
    }
}

impl Cheatcode for resumeGasMeteringCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        state.gas_metering = None;
        Ok(Default::default())
    }
}

impl Cheatcode for chainIdCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { newChainId } = self;
        ensure!(*newChainId <= U256::from(u64::MAX), "chain ID must be less than 2^64 - 1");
        ccx.data.env.cfg.chain_id = newChainId.to();
        Ok(Default::default())
    }
}

impl Cheatcode for coinbaseCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { newCoinbase } = self;
        ccx.data.env.block.coinbase = *newCoinbase;
        Ok(Default::default())
    }
}

impl Cheatcode for difficultyCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { newDifficulty } = self;
        ensure!(
            ccx.data.env.cfg.spec_id < SpecId::MERGE,
            "`difficulty` is not supported after the Paris hard fork, use `prevrandao` instead; \
             see EIP-4399: https://eips.ethereum.org/EIPS/eip-4399"
        );
        ccx.data.env.block.difficulty = *newDifficulty;
        Ok(Default::default())
    }
}

impl Cheatcode for feeCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { newBasefee } = self;
        ccx.data.env.block.basefee = *newBasefee;
        Ok(Default::default())
    }
}

impl Cheatcode for prevrandaoCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { newPrevrandao } = self;
        ensure!(
            ccx.data.env.cfg.spec_id >= SpecId::MERGE,
            "`prevrandao` is not supported before the Paris hard fork, use `difficulty` instead; \
             see EIP-4399: https://eips.ethereum.org/EIPS/eip-4399"
        );
        ccx.data.env.block.prevrandao = Some(*newPrevrandao);
        Ok(Default::default())
    }
}

impl Cheatcode for rollCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { newHeight } = self;
        ccx.data.env.block.number = *newHeight;
        Ok(Default::default())
    }
}

impl Cheatcode for txGasPriceCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { newGasPrice } = self;
        ccx.data.env.tx.gas_price = *newGasPrice;
        Ok(Default::default())
    }
}

impl Cheatcode for warpCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { newTimestamp } = self;
        ccx.data.env.block.timestamp = *newTimestamp;
        Ok(Default::default())
    }
}

impl Cheatcode for dealCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { account: address, newBalance: new_balance } = *self;
        let account = journaled_account(ccx.data, address)?;
        let old_balance = std::mem::replace(&mut account.info.balance, new_balance);
        let record = DealRecord { address, old_balance, new_balance };
        ccx.state.eth_deals.push(record);
        Ok(Default::default())
    }
}

impl Cheatcode for etchCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { target, newRuntimeBytecode } = self;
        ensure_not_precompile!(target, ccx);
        ccx.data.journaled_state.load_account(*target, ccx.data.db)?;
        let bytecode = Bytecode::new_raw(Bytes::copy_from_slice(newRuntimeBytecode)).to_checked();
        ccx.data.journaled_state.set_code(*target, bytecode);
        Ok(Default::default())
    }
}

impl Cheatcode for resetNonceCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { account } = self;
        let account = journaled_account(ccx.data, *account)?;
        // Per EIP-161, EOA nonces start at 0, but contract nonces
        // start at 1. Comparing by code_hash instead of code
        // to avoid hitting the case where account's code is None.
        let empty = account.info.code_hash == KECCAK_EMPTY;
        let nonce = if empty { 0 } else { 1 };
        account.info.nonce = nonce;
        debug!(target: "cheatcodes", nonce, "reset");
        Ok(Default::default())
    }
}

impl Cheatcode for setNonceCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { account, newNonce } = *self;
        let account = journaled_account(ccx.data, account)?;
        // nonce must increment only
        let current = account.info.nonce;
        ensure!(
            newNonce >= current,
            "new nonce ({newNonce}) must be strictly equal to or higher than the \
             account's current nonce ({current})"
        );
        account.info.nonce = newNonce;
        Ok(Default::default())
    }
}

impl Cheatcode for setNonceUnsafeCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { account, newNonce } = *self;
        let account = journaled_account(ccx.data, account)?;
        account.info.nonce = newNonce;
        Ok(Default::default())
    }
}

impl Cheatcode for storeCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { target, slot, value } = *self;
        ensure_not_precompile!(&target, ccx);
        // ensure the account is touched
        let _ = journaled_account(ccx.data, target)?;
        ccx.data.journaled_state.sstore(target, slot.into(), value.into(), ccx.data.db)?;
        Ok(Default::default())
    }
}

impl Cheatcode for coolCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { target } = *self;
        ensure_not_precompile!(&target, ccx);
        // TODO: prevent or warn about cooling the to/from address in a tx
        ccx.state.addresses.insert(target, (AddressState::Cool, HashMap::new()));
        Ok(Default::default())
    }
}

impl Cheatcode for readCallersCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self {} = self;
        read_callers(ccx.state, &ccx.data.env.tx.caller)
    }
}

impl Cheatcode for snapshotCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self {} = self;
        Ok(ccx.data.db.snapshot(&ccx.data.journaled_state, ccx.data.env).abi_encode())
    }
}

impl Cheatcode for revertToCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { snapshotId } = self;
        let result = if let Some(journaled_state) =
            ccx.data.db.revert(*snapshotId, &ccx.data.journaled_state, ccx.data.env)
        {
            // we reset the evm's journaled_state to the state of the snapshot previous state
            ccx.data.journaled_state = journaled_state;
            true
        } else {
            false
        };
        Ok(result.abi_encode())
    }
}

pub(super) fn get_nonce<DB: DatabaseExt>(ccx: &mut CheatsCtxt<DB>, address: &Address) -> Result {
    super::script::correct_sender_nonce(ccx)?;
    let (account, _) = ccx.data.journaled_state.load_account(*address, ccx.data.db)?;
    Ok(account.info.nonce.abi_encode())
}

/// Reads the current caller information and returns the current [CallerMode], `msg.sender` and
/// `tx.origin`.
///
/// Depending on the current caller mode, one of the following results will be returned:
/// - If there is an active prank:
///     - caller_mode will be equal to:
///         - [CallerMode::Prank] if the prank has been set with `vm.prank(..)`.
///         - [CallerMode::RecurrentPrank] if the prank has been set with `vm.startPrank(..)`.
///     - `msg.sender` will be equal to the address set for the prank.
///     - `tx.origin` will be equal to the default sender address unless an alternative one has been
///       set when configuring the prank.
///
/// - If there is an active broadcast:
///     - caller_mode will be equal to:
///         - [CallerMode::Broadcast] if the broadcast has been set with `vm.broadcast(..)`.
///         - [CallerMode::RecurrentBroadcast] if the broadcast has been set with
///           `vm.startBroadcast(..)`.
///     - `msg.sender` and `tx.origin` will be equal to the address provided when setting the
///       broadcast.
///
/// - If no caller modification is active:
///     - caller_mode will be equal to [CallerMode::None],
///     - `msg.sender` and `tx.origin` will be equal to the default sender address.
fn read_callers(state: &Cheatcodes, default_sender: &Address) -> Result {
    let Cheatcodes { prank, broadcast, .. } = state;

    let mut mode = CallerMode::None;
    let mut new_caller = default_sender;
    let mut new_origin = default_sender;
    if let Some(prank) = prank {
        mode = if prank.single_call { CallerMode::Prank } else { CallerMode::RecurrentPrank };
        new_caller = &prank.new_caller;
        if let Some(new) = &prank.new_origin {
            new_origin = new;
        }
    } else if let Some(broadcast) = broadcast {
        mode = if broadcast.single_call {
            CallerMode::Broadcast
        } else {
            CallerMode::RecurrentBroadcast
        };
        new_caller = &broadcast.new_origin;
        new_origin = &broadcast.new_origin;
    }

    Ok((mode, new_caller, new_origin).abi_encode_params())
}

/// Ensures the `Account` is loaded and touched.
pub(super) fn journaled_account<'a, DB: DatabaseExt>(
    data: &'a mut EVMData<'_, DB>,
    addr: Address,
) -> Result<&'a mut Account> {
    data.journaled_state.load_account(addr, data.db)?;
    data.journaled_state.touch(&addr);
    Ok(data.journaled_state.state.get_mut(&addr).expect("account is loaded"))
}
