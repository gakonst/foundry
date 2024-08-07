//! Implementations of [`Utilities`](spec::Group::Utilities) cheatcodes.

use crate::{Cheatcode, Cheatcodes, Result, Vm::*};
use alloy_primitives::{Address, U256};
use alloy_sol_types::SolValue;
use foundry_common::ens::namehash;
use foundry_evm_core::constants::DEFAULT_CREATE2_DEPLOYER;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

impl Cheatcode for labelCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self { account, newLabel } = self;
        state.labels.insert(*account, newLabel.clone());
        Ok(Default::default())
    }
}

impl Cheatcode for getLabelCall {
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let Self { account } = self;
        Ok(match state.labels.get(account) {
            Some(label) => label.abi_encode(),
            None => format!("unlabeled:{account}").abi_encode(),
        })
    }
}

impl Cheatcode for computeCreateAddressCall {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { nonce, deployer } = self;
        ensure!(*nonce <= U256::from(u64::MAX), "nonce must be less than 2^64 - 1");
        Ok(deployer.create(nonce.to()).abi_encode())
    }
}

impl Cheatcode for computeCreate2Address_0Call {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { salt, initCodeHash, deployer } = self;
        Ok(deployer.create2(salt, initCodeHash).abi_encode())
    }
}

impl Cheatcode for computeCreate2Address_1Call {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { salt, initCodeHash } = self;
        Ok(DEFAULT_CREATE2_DEPLOYER.create2(salt, initCodeHash).abi_encode())
    }
}

impl Cheatcode for ensNamehashCall {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { name } = self;
        Ok(namehash(name).abi_encode())
    }
}

impl Cheatcode for randomUint_0Call {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        // Use thread_rng to get a random number
        let mut rng = rand::thread_rng();
        let random_number: U256 = rng.gen();
        Ok(random_number.abi_encode())
    }
}

impl Cheatcode for randomUint_1Call {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { seed } = self;
        let seed_bytes: [u8; 32] = seed.to_be_bytes();
        let mut rng = ChaCha20Rng::from_seed(seed_bytes);
        let random_number: U256 = rng.gen();
        Ok(random_number.abi_encode())
    }
}

impl Cheatcode for randomUint_2Call {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { min, max } = *self;
        ensure!(min <= max, "min must be less than or equal to max");
        // Generate random between range min..=max
        let mut rng = rand::thread_rng();
        let exclusive_modulo = max - min;
        let mut random_number = rng.gen::<U256>();
        if exclusive_modulo != U256::MAX {
            let inclusive_modulo = exclusive_modulo + U256::from(1);
            random_number %= inclusive_modulo;
        }
        random_number += min;
        Ok(random_number.abi_encode())
    }
}

impl Cheatcode for randomUint_3Call {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { seed, min, max } = *self;
        ensure!(min <= max, "min must be less than or equal to max");
        // Generate random between range min..=max
        let seed_bytes: [u8; 32] = seed.to_be_bytes();
        let mut rng = ChaCha20Rng::from_seed(seed_bytes);
        let exclusive_modulo = max - min;
        let mut random_number = rng.gen::<U256>();
        if exclusive_modulo != U256::MAX {
            let inclusive_modulo = exclusive_modulo + U256::from(1);
            random_number %= inclusive_modulo;
        }
        random_number += min;
        Ok(random_number.abi_encode())
    }
}

impl Cheatcode for randomAddress_0Call {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self {} = self;
        let addr = Address::random();
        Ok(addr.abi_encode())
    }
}

impl Cheatcode for randomAddress_1Call {
    fn apply(&self, _state: &mut Cheatcodes) -> Result {
        let Self { seed } = self;
        let seed_bytes: [u8; 32] = seed.to_be_bytes();
        let mut rng = ChaCha20Rng::from_seed(seed_bytes);
        let addr = Address::random_with(&mut rng);
        Ok(addr.abi_encode())
    }
}
