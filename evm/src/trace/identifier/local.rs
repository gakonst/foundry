use super::{AddressIdentity, TraceIdentifier};
use ethers::{
    abi::{Abi, Address},
    prelude::ArtifactId,
};
use std::{borrow::Cow, collections::BTreeMap};
use foundry_utils::diff_score;

/// A trace identifier that tries to identify addresses using local contracts.
pub struct LocalTraceIdentifier {
    local_contracts: BTreeMap<Vec<u8>, (String, Abi)>,
}

impl LocalTraceIdentifier {
    pub fn new(known_contracts: &BTreeMap<ArtifactId, (Abi, Vec<u8>)>) -> Self {
        Self {
            local_contracts: known_contracts
                .iter()
                .map(|(id, (abi, runtime_code))| {
                    (runtime_code.clone(), (id.name.clone(), abi.clone()))
                })
                .collect(),
        }
    }
}

impl TraceIdentifier for LocalTraceIdentifier {
    fn identify_addresses(
        &self,
        addresses: Vec<(&Address, Option<&Vec<u8>>)>,
    ) -> Vec<AddressIdentity> {
        addresses
            .into_iter()
            .filter_map(|(address, code)| {
                let code = code?;
                let (_, (name, abi)) = self
                    .local_contracts
                    .iter()
                    .find(|(known_code, _)| diff_score(known_code, code) < 0.1)?;

                Some(AddressIdentity {
                    address: *address,
                    contract: Some(name.clone()),
                    label: Some(name.clone()),
                    abi: Some(Cow::Borrowed(abi)),
                })
            })
            .collect()
    }
}