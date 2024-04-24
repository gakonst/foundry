use crate::{Cheatcode, Cheatcodes, CheatsCtxt, DatabaseExt, Result, Vm::*};
use alloy_primitives::{B256, U256};
use alloy_provider::Provider;
use alloy_rpc_types::Filter;
use alloy_sol_types::SolValue;
use eyre::WrapErr;
use foundry_common::provider::alloy::ProviderBuilder;
use foundry_compilers::utils::RuntimeOrHandle;
use foundry_evm_core::fork::CreateFork;

impl Cheatcode for activeForkCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self {} = self;
        ccx.ecx
            .db
            .active_fork_id()
            .map(|id| id.abi_encode())
            .ok_or_else(|| fmt_err!("no active fork"))
    }
}

impl Cheatcode for createFork_0Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { urlOrAlias } = self;
        create_fork(ccx, urlOrAlias, None)
    }
}

impl Cheatcode for createFork_1Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { urlOrAlias, blockNumber } = self;
        create_fork(ccx, urlOrAlias, Some(blockNumber.saturating_to()))
    }
}

impl Cheatcode for createFork_2Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { urlOrAlias, txHash } = self;
        create_fork_at_transaction(ccx, urlOrAlias, txHash)
    }
}

impl Cheatcode for createSelectFork_0Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { urlOrAlias } = self;
        create_select_fork(ccx, urlOrAlias, None)
    }
}

impl Cheatcode for createSelectFork_1Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { urlOrAlias, blockNumber } = self;
        create_select_fork(ccx, urlOrAlias, Some(blockNumber.saturating_to()))
    }
}

impl Cheatcode for createSelectFork_2Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { urlOrAlias, txHash } = self;
        create_select_fork_at_transaction(ccx, urlOrAlias, txHash)
    }
}

impl Cheatcode for rollFork_0Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { blockNumber } = self;
        ccx.ecx.db.roll_fork(
            None,
            (*blockNumber).to(),
            &mut ccx.ecx.env,
            &mut ccx.ecx.journaled_state,
        )?;
        Ok(Default::default())
    }
}

impl Cheatcode for rollFork_1Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { txHash } = self;
        ccx.ecx.db.roll_fork_to_transaction(
            None,
            *txHash,
            &mut ccx.ecx.env,
            &mut ccx.ecx.journaled_state,
        )?;
        Ok(Default::default())
    }
}

impl Cheatcode for rollFork_2Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { forkId, blockNumber } = self;
        ccx.ecx.db.roll_fork(
            Some(*forkId),
            (*blockNumber).to(),
            &mut ccx.ecx.env,
            &mut ccx.ecx.journaled_state,
        )?;
        Ok(Default::default())
    }
}

impl Cheatcode for rollFork_3Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { forkId, txHash } = self;
        ccx.ecx.db.roll_fork_to_transaction(
            Some(*forkId),
            *txHash,
            &mut ccx.ecx.env,
            &mut ccx.ecx.journaled_state,
        )?;
        Ok(Default::default())
    }
}

impl Cheatcode for selectForkCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { forkId } = self;
        check_broadcast(ccx.state)?;

        // No need to correct since the sender's nonce does not get incremented when selecting a
        // fork.
        ccx.state.corrected_nonce = true;

        ccx.ecx.db.select_fork(
            *forkId,
            &mut ccx.ecx.env,
            ccx.precompiles,
            &mut ccx.ecx.journaled_state,
        )?;
        Ok(Default::default())
    }
}

impl Cheatcode for transact_0Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { txHash } = *self;
        ccx.ecx.db.transact(
            None,
            txHash,
            &mut ccx.ecx.env,
            &mut ccx.ecx.journaled_state,
            ccx.state,
        )?;
        Ok(Default::default())
    }
}

impl Cheatcode for transact_1Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { forkId, txHash } = *self;
        ccx.ecx.db.transact(
            Some(forkId),
            txHash,
            &mut ccx.ecx.env,
            &mut ccx.ecx.journaled_state,
            ccx.state,
        )?;
        Ok(Default::default())
    }
}

impl Cheatcode for allowCheatcodesCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { account } = self;
        ccx.ecx.db.allow_cheatcode_access(*account);
        Ok(Default::default())
    }
}

impl Cheatcode for makePersistent_0Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { account } = self;
        ccx.ecx.db.add_persistent_account(*account);
        Ok(Default::default())
    }
}

impl Cheatcode for makePersistent_1Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { account0, account1 } = self;
        ccx.ecx.db.add_persistent_account(*account0);
        ccx.ecx.db.add_persistent_account(*account1);
        Ok(Default::default())
    }
}

impl Cheatcode for makePersistent_2Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { account0, account1, account2 } = self;
        ccx.ecx.db.add_persistent_account(*account0);
        ccx.ecx.db.add_persistent_account(*account1);
        ccx.ecx.db.add_persistent_account(*account2);
        Ok(Default::default())
    }
}

impl Cheatcode for makePersistent_3Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { accounts } = self;
        ccx.ecx.db.extend_persistent_accounts(accounts.iter().copied());
        Ok(Default::default())
    }
}

impl Cheatcode for revokePersistent_0Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { account } = self;
        ccx.ecx.db.remove_persistent_account(account);
        Ok(Default::default())
    }
}

impl Cheatcode for revokePersistent_1Call {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { accounts } = self;
        ccx.ecx.db.remove_persistent_accounts(accounts.iter().copied());
        Ok(Default::default())
    }
}

impl Cheatcode for isPersistentCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { account } = self;
        Ok(ccx.ecx.db.is_persistent(account).abi_encode())
    }
}

impl Cheatcode for rpcCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { method, params } = self;
        let url =
            ccx.ecx.db.active_fork_url().ok_or_else(|| fmt_err!("no active fork URL found"))?;
        let provider = ProviderBuilder::new(&url).build()?;
        let method: &'static str = Box::new(method.clone()).leak();
        let params_json: serde_json::Value = serde_json::from_str(params)?;
        let result = RuntimeOrHandle::new()
            .block_on(provider.raw_request(method.into(), params_json))
            .map_err(|err| fmt_err!("{method:?}: {err}"))?;

        let result_as_tokens = crate::json::json_value_to_token(&result)
            .map_err(|err| fmt_err!("failed to parse result: {err}"))?;

        Ok(result_as_tokens.abi_encode())
    }
}

impl Cheatcode for eth_getLogsCall {
    fn apply_full<DB: DatabaseExt>(&self, ccx: &mut CheatsCtxt<DB>) -> Result {
        let Self { fromBlock, toBlock, target, topics } = self;
        let (Ok(from_block), Ok(to_block)) = (u64::try_from(fromBlock), u64::try_from(toBlock))
        else {
            bail!("blocks in block range must be less than 2^64 - 1")
        };

        if topics.len() > 4 {
            bail!("topics array must contain at most 4 elements")
        }

        let url =
            ccx.ecx.db.active_fork_url().ok_or_else(|| fmt_err!("no active fork URL found"))?;
        let provider = ProviderBuilder::new(&url).build()?;
        let mut filter = Filter::new().address(*target).from_block(from_block).to_block(to_block);
        for (i, topic) in topics.iter().enumerate() {
            // todo: needed because rust wants to convert FixedBytes<32> to U256 to convert it back
            // to FixedBytes<32> and then to Topic for some reason removing the
            // From<U256> impl in alloy does not fix the situation, and it is not possible to impl
            // From<FixedBytes<32>> either because of a conflicting impl
            match i {
                0 => filter = filter.event_signature(*topic),
                1 => filter = filter.topic1(*topic),
                2 => filter = filter.topic2(*topic),
                3 => filter = filter.topic3(*topic),
                _ => unreachable!(),
            };
        }

        // todo: handle the errors somehow
        let logs = RuntimeOrHandle::new()
            .block_on(provider.get_logs(&filter))
            .wrap_err("failed to get logs")?;

        let eth_logs = logs
            .into_iter()
            .map(|log| EthGetLogs {
                emitter: log.address(),
                topics: log.topics().to_vec(),
                data: log.inner.data.data,
                blockHash: log.block_hash.unwrap_or_default(),
                blockNumber: log.block_number.unwrap_or_default(),
                transactionHash: log.transaction_hash.unwrap_or_default(),
                transactionIndex: log.transaction_index.unwrap_or_default(),
                logIndex: U256::from(log.log_index.unwrap_or_default()),
                removed: log.removed,
            })
            .collect::<Vec<_>>();

        Ok(eth_logs.abi_encode())
    }
}

/// Creates and then also selects the new fork
fn create_select_fork<DB: DatabaseExt>(
    ccx: &mut CheatsCtxt<DB>,
    url_or_alias: &str,
    block: Option<u64>,
) -> Result {
    check_broadcast(ccx.state)?;

    // No need to correct since the sender's nonce does not get incremented when selecting a fork.
    ccx.state.corrected_nonce = true;

    let fork = create_fork_request(ccx, url_or_alias, block)?;
    let id = ccx.ecx.db.create_select_fork(
        fork,
        &mut ccx.ecx.env,
        ccx.precompiles,
        &mut ccx.ecx.journaled_state,
    )?;
    Ok(id.abi_encode())
}

/// Creates a new fork
fn create_fork<DB: DatabaseExt>(
    ccx: &mut CheatsCtxt<DB>,
    url_or_alias: &str,
    block: Option<u64>,
) -> Result {
    let fork = create_fork_request(ccx, url_or_alias, block)?;
    let id = ccx.ecx.db.create_fork(fork)?;
    Ok(id.abi_encode())
}

/// Creates and then also selects the new fork at the given transaction
fn create_select_fork_at_transaction<DB: DatabaseExt>(
    ccx: &mut CheatsCtxt<DB>,
    url_or_alias: &str,
    transaction: &B256,
) -> Result {
    check_broadcast(ccx.state)?;

    // No need to correct since the sender's nonce does not get incremented when selecting a fork.
    ccx.state.corrected_nonce = true;

    let fork = create_fork_request(ccx, url_or_alias, None)?;
    let id = ccx.ecx.db.create_select_fork_at_transaction(
        fork,
        &mut ccx.ecx.env,
        &mut ccx.ecx.journaled_state,
        ccx.precompiles,
        *transaction,
    )?;
    Ok(id.abi_encode())
}

/// Creates a new fork at the given transaction
fn create_fork_at_transaction<DB: DatabaseExt>(
    ccx: &mut CheatsCtxt<DB>,
    url_or_alias: &str,
    transaction: &B256,
) -> Result {
    let fork = create_fork_request(ccx, url_or_alias, None)?;
    let id = ccx.ecx.db.create_fork_at_transaction(fork, *transaction)?;
    Ok(id.abi_encode())
}

/// Creates the request object for a new fork request
fn create_fork_request<DB: DatabaseExt>(
    ccx: &mut CheatsCtxt<DB>,
    url_or_alias: &str,
    block: Option<u64>,
) -> Result<CreateFork> {
    let url = ccx.state.config.rpc_url(url_or_alias)?;
    let mut evm_opts = ccx.state.config.evm_opts.clone();
    evm_opts.fork_block_number = block;
    let fork = CreateFork {
        enable_caching: ccx.state.config.rpc_storage_caching.enable_for_endpoint(&url),
        url,
        env: (*ccx.ecx.env).clone(),
        evm_opts,
    };
    Ok(fork)
}

#[inline]
fn check_broadcast(state: &Cheatcodes) -> Result<()> {
    if state.broadcast.is_none() {
        Ok(())
    } else {
        Err(fmt_err!("cannot select forks during a broadcast"))
    }
}
