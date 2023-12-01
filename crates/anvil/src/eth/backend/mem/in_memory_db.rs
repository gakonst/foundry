//! The in memory DB

use crate::{
    eth::backend::db::{
        AsHashDB, Db, MaybeForkedDatabase, MaybeHashDatabase, SerializableAccountRecord,
        SerializableState, StateDb,
    },
    mem::state::{state_merkle_trie_root, storage_trie_db, trie_hash_db},
    revm::primitives::AccountInfo,
    Address, U256,
};
use ethers::{prelude::H256, types::BlockId};
use foundry_common::types::{ToAlloy, ToEthers};
use foundry_evm::{
    backend::{DatabaseResult, StateSnapshot},
    fork::BlockchainDb,
};

// reexport for convenience
pub use foundry_evm::{backend::MemDb, revm::db::DatabaseRef};

impl Db for MemDb {
    fn insert_account(&mut self, address: Address, account: AccountInfo) {
        self.inner.insert_account_info(address.to_alloy(), account)
    }

    fn set_storage_at(&mut self, address: Address, slot: U256, val: U256) -> DatabaseResult<()> {
        self.inner.insert_account_storage(address.to_alloy(), slot.to_alloy(), val.to_alloy())
    }

    fn insert_block_hash(&mut self, number: U256, hash: H256) {
        self.inner.block_hashes.insert(number.to_alloy(), hash.to_alloy());
    }

    fn dump_state(&self) -> DatabaseResult<Option<SerializableState>> {
        let accounts = self
            .inner
            .accounts
            .clone()
            .into_iter()
            .map(|(k, v)| -> DatabaseResult<_> {
                let code = if let Some(code) = v.info.code {
                    code
                } else {
                    self.inner.code_by_hash_ref(v.info.code_hash)?
                }
                .to_checked();
                Ok((
                    k.to_ethers(),
                    SerializableAccountRecord {
                        nonce: v.info.nonce,
                        balance: v.info.balance.to_ethers(),
                        code: code.bytes()[..code.len()].to_vec().into(),
                        storage: v
                            .storage
                            .into_iter()
                            .map(|k| (k.0.to_ethers(), k.1.to_ethers()))
                            .collect(),
                    },
                ))
            })
            .collect::<Result<_, _>>()?;

        Ok(Some(SerializableState { accounts }))
    }

    /// Creates a new snapshot
    fn snapshot(&mut self) -> U256 {
        let id = self.snapshots.insert(self.inner.clone());
        trace!(target: "backend::memdb", "Created new snapshot {}", id);
        id.to_ethers()
    }

    fn revert(&mut self, id: U256) -> bool {
        if let Some(snapshot) = self.snapshots.revert_to(id.to_alloy()) {
            self.inner = snapshot;
            trace!(target: "backend::memdb", "Reverted snapshot {}", id);
            true
        } else {
            warn!(target: "backend::memdb", "No snapshot to revert for {}", id);
            false
        }
    }

    fn maybe_state_root(&self) -> Option<H256> {
        Some(state_merkle_trie_root(&self.inner.accounts))
    }

    fn current_state(&self) -> StateDb {
        StateDb::new(MemDb { inner: self.inner.clone(), ..Default::default() })
    }
}

impl MaybeHashDatabase for MemDb {
    fn maybe_as_hash_db(&self) -> Option<(AsHashDB, H256)> {
        Some(trie_hash_db(&self.inner.accounts))
    }

    fn maybe_account_db(&self, addr: Address) -> Option<(AsHashDB, H256)> {
        if let Some(acc) = self.inner.accounts.get(&addr.to_alloy()) {
            Some(storage_trie_db(&acc.storage))
        } else {
            Some(storage_trie_db(&Default::default()))
        }
    }

    fn clear_into_snapshot(&mut self) -> StateSnapshot {
        self.inner.clear_into_snapshot()
    }

    fn clear(&mut self) {
        self.inner.clear();
    }

    fn init_from_snapshot(&mut self, snapshot: StateSnapshot) {
        self.inner.init_from_snapshot(snapshot)
    }
}

impl MaybeForkedDatabase for MemDb {
    fn maybe_reset(&mut self, _url: Option<String>, _block_number: BlockId) -> Result<(), String> {
        Err("not supported".to_string())
    }

    fn maybe_flush_cache(&self) -> Result<(), String> {
        Err("not supported".to_string())
    }

    fn maybe_inner(&self) -> Result<&BlockchainDb, String> {
        Err("not supported".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        eth::backend::db::{Db, SerializableAccountRecord, SerializableState},
        revm::primitives::AccountInfo,
        Address,
    };
    use alloy_primitives::{Bytes, U256 as rU256};
    use ethers::types::U256;
    use foundry_common::types::ToAlloy;
    use foundry_evm::{
        backend::MemDb,
        revm::primitives::{Bytecode, KECCAK_EMPTY},
    };
    use std::{collections::BTreeMap, str::FromStr};

    // verifies that all substantial aspects of a loaded account remain the state after an account
    // is dumped and reloaded
    #[test]
    fn test_dump_reload_cycle() {
        let test_addr: Address =
            Address::from_str("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266").unwrap();

        let mut dump_db = MemDb::default();

        let contract_code = Bytecode::new_raw(Bytes::from("fake contract code")).to_checked();

        dump_db.insert_account(
            test_addr,
            AccountInfo {
                balance: rU256::from(123456),
                code_hash: KECCAK_EMPTY,
                code: Some(contract_code.clone()),
                nonce: 1234,
            },
        );

        dump_db.set_storage_at(test_addr, U256::from(1234567), U256::from(1)).unwrap();

        let state = dump_db.dump_state().unwrap().unwrap();

        let mut load_db = MemDb::default();

        load_db.load_state(state).unwrap();

        let loaded_account = load_db.basic_ref(test_addr.to_alloy()).unwrap().unwrap();

        assert_eq!(loaded_account.balance, rU256::from(123456));
        assert_eq!(load_db.code_by_hash_ref(loaded_account.code_hash).unwrap(), contract_code);
        assert_eq!(loaded_account.nonce, 1234);
        assert_eq!(
            load_db.storage_ref(test_addr.to_alloy(), rU256::from(1234567)).unwrap(),
            rU256::from(1)
        );
    }

    // verifies that multiple accounts can be loaded at a time, and storage is merged within those
    // accounts as well.
    #[test]
    fn test_load_state_merge() {
        let test_addr: Address =
            Address::from_str("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266").unwrap();
        let test_addr2: Address =
            Address::from_str("0x70997970c51812dc3a010c7d01b50e0d17dc79c8").unwrap();

        let contract_code = Bytecode::new_raw(Bytes::from("fake contract code")).to_checked();

        let mut db = MemDb::default();

        db.insert_account(
            test_addr,
            AccountInfo {
                balance: rU256::from(123456),
                code_hash: KECCAK_EMPTY,
                code: Some(contract_code.clone()),
                nonce: 1234,
            },
        );

        db.set_storage_at(test_addr, U256::from(1234567), U256::from(1)).unwrap();
        db.set_storage_at(test_addr, U256::from(1234568), U256::from(2)).unwrap();

        let mut new_state = SerializableState::default();

        new_state.accounts.insert(
            test_addr2,
            SerializableAccountRecord {
                balance: Default::default(),
                code: Default::default(),
                nonce: 1,
                storage: Default::default(),
            },
        );

        let mut new_storage = BTreeMap::default();
        new_storage.insert(U256::from(1234568), U256::from(5));

        new_state.accounts.insert(
            test_addr,
            SerializableAccountRecord {
                balance: 100100.into(),
                code: contract_code.bytes()[..contract_code.len()].to_vec().into(),
                nonce: 100,
                storage: new_storage,
            },
        );

        db.load_state(new_state).unwrap();

        let loaded_account = db.basic_ref(test_addr.to_alloy()).unwrap().unwrap();
        let loaded_account2 = db.basic_ref(test_addr2.to_alloy()).unwrap().unwrap();

        assert_eq!(loaded_account2.nonce, 1);

        assert_eq!(loaded_account.balance, rU256::from(100100));
        assert_eq!(db.code_by_hash_ref(loaded_account.code_hash).unwrap(), contract_code);
        assert_eq!(loaded_account.nonce, 1234);
        assert_eq!(
            db.storage_ref(test_addr.to_alloy(), rU256::from(1234567)).unwrap(),
            rU256::from(1)
        );
        assert_eq!(
            db.storage_ref(test_addr.to_alloy(), rU256::from(1234568)).unwrap(),
            rU256::from(5)
        );
    }
}
