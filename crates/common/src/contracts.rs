//! Commonly used contract types and functions.

use alloy_json_abi::{Event, Function, JsonAbi};
use alloy_primitives::{hex, Address, Bytes, Selector, B256};
use eyre::Result;
use foundry_compilers::{
    artifacts::{
        BytecodeObject, CompactBytecode, CompactContractBytecode, CompactDeployedBytecode,
        ContractBytecodeSome, Offsets,
    },
    ArtifactId, ProjectCompileOutput,
};
use std::{collections::BTreeMap, ops::Deref, path::PathBuf, str::FromStr, sync::Arc};

/// Libraries' runtime code always starts with the following instruction:
/// `PUSH20 0x0000000000000000000000000000000000000000`
///
/// See: <https://docs.soliditylang.org/en/latest/contracts.html#call-protection-for-libraries>
const CALL_PROTECTION_BYTECODE_PREFIX: [u8; 21] =
    hex!("730000000000000000000000000000000000000000");

/// Subset of [CompactBytecode] excluding sourcemaps.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct BytecodeData {
    pub object: Option<BytecodeObject>,
    pub link_references: BTreeMap<String, BTreeMap<String, Vec<Offsets>>>,
    pub immutable_references: BTreeMap<String, Vec<Offsets>>,
}

impl BytecodeData {
    fn bytes(&self) -> Option<&Bytes> {
        self.object.as_ref().and_then(|b| b.as_bytes())
    }
}

impl From<CompactBytecode> for BytecodeData {
    fn from(bytecode: CompactBytecode) -> Self {
        Self {
            object: Some(bytecode.object),
            link_references: bytecode.link_references,
            immutable_references: BTreeMap::new(),
        }
    }
}

impl From<CompactDeployedBytecode> for BytecodeData {
    fn from(bytecode: CompactDeployedBytecode) -> Self {
        let (object, link_references) = if let Some(compact) = bytecode.bytecode {
            (Some(compact.object), compact.link_references)
        } else {
            (None, BTreeMap::new())
        };
        Self { object, link_references, immutable_references: bytecode.immutable_references }
    }
}

/// Container for commonly used contract data.
#[derive(Debug)]
pub struct ContractData {
    /// Contract name.
    pub name: String,
    /// Contract ABI.
    pub abi: JsonAbi,
    /// Contract creation code.
    pub bytecode: Option<BytecodeData>,
    /// Contract runtime code.
    pub deployed_bytecode: Option<BytecodeData>,
}

impl ContractData {
    /// Returns reference to bytes of contract creation code, if present.
    pub fn bytecode(&self) -> Option<&Bytes> {
        self.bytecode.as_ref()?.bytes().filter(|b| !b.is_empty())
    }

    /// Returns reference to bytes of contract deployed code, if present.
    pub fn deployed_bytecode(&self) -> Option<&Bytes> {
        self.deployed_bytecode.as_ref()?.bytes().filter(|b| !b.is_empty())
    }
}

type ArtifactWithContractRef<'a> = (&'a ArtifactId, &'a ContractData);

/// Wrapper type that maps an artifact to a contract ABI and bytecode.
#[derive(Clone, Default, Debug)]
pub struct ContractsByArtifact(Arc<BTreeMap<ArtifactId, ContractData>>);

impl ContractsByArtifact {
    /// Creates a new instance by collecting all artifacts with present bytecode from an iterator.
    pub fn new(artifacts: impl IntoIterator<Item = (ArtifactId, CompactContractBytecode)>) -> Self {
        let map = artifacts
            .into_iter()
            .filter_map(|(id, artifact)| {
                let name = id.name.clone();
                let CompactContractBytecode { abi, bytecode, deployed_bytecode } = artifact;
                Some((
                    id,
                    ContractData {
                        name,
                        abi: abi?,
                        bytecode: bytecode.map(Into::into),
                        deployed_bytecode: deployed_bytecode.map(Into::into),
                    },
                ))
            })
            .collect();
        Self(Arc::new(map))
    }

    /// Clears all contracts.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Finds a contract which has a similar bytecode as `code`.
    pub fn find_by_creation_code(&self, code: &[u8]) -> Option<ArtifactWithContractRef<'_>> {
        self.find_by_code(code, 0.1, ContractData::bytecode)
    }

    /// Finds a contract which has a similar deployed bytecode as `code`.
    pub fn find_by_deployed_code(&self, code: &[u8]) -> Option<ArtifactWithContractRef<'_>> {
        self.find_by_code(code, 0.15, ContractData::deployed_bytecode)
    }

    /// Finds a contract based on provided bytecode and accepted match score.
    fn find_by_code(
        &self,
        code: &[u8],
        accepted_score: f64,
        get: impl Fn(&ContractData) -> Option<&Bytes>,
    ) -> Option<ArtifactWithContractRef<'_>> {
        self.iter()
            .filter_map(|(id, contract)| {
                if let Some(deployed_bytecode) = get(contract) {
                    let score = bytecode_diff_score(deployed_bytecode.as_ref(), code);
                    (score <= accepted_score).then_some((score, (id, contract)))
                } else {
                    None
                }
            })
            .min_by(|(score1, _), (score2, _)| score1.partial_cmp(score2).unwrap())
            .map(|(_, data)| data)
    }

    /// Finds a contract which deployed bytecode exactly matches the given code. Accounts for link
    /// references and immutables.
    pub fn find_by_deployed_code_exact(&self, code: &[u8]) -> Option<ArtifactWithContractRef<'_>> {
        // Immediately return None if the code is empty.
        if code.is_empty() {
            return None;
        }

        self.iter().find(|(_, contract)| {
            let Some(deployed_bytecode) = &contract.deployed_bytecode else {
                return false;
            };
            let Some(deployed_code) = &deployed_bytecode.object else {
                return false;
            };

            let len = match deployed_code {
                BytecodeObject::Bytecode(ref bytes) => bytes.len(),
                BytecodeObject::Unlinked(ref bytes) => bytes.len() / 2,
            };

            if len != code.len() {
                return false;
            }

            // Collect ignored offsets by chaining link and immutable references.
            let mut ignored = deployed_bytecode
                .immutable_references
                .values()
                .chain(deployed_bytecode.link_references.values().flat_map(|v| v.values()))
                .flatten()
                .cloned()
                .collect::<Vec<_>>();

            // For libraries solidity adds a call protection prefix to the bytecode. We need to
            // ignore it as it includes library address determined at runtime.
            // See https://docs.soliditylang.org/en/latest/contracts.html#call-protection-for-libraries and
            // https://github.com/NomicFoundation/hardhat/blob/af7807cf38842a4f56e7f4b966b806e39631568a/packages/hardhat-verify/src/internal/solc/bytecode.ts#L172
            let has_call_protection = match deployed_code {
                BytecodeObject::Bytecode(ref bytes) => {
                    bytes.starts_with(&CALL_PROTECTION_BYTECODE_PREFIX)
                }
                BytecodeObject::Unlinked(ref bytes) => {
                    if let Ok(bytes) =
                        Bytes::from_str(&bytes[..CALL_PROTECTION_BYTECODE_PREFIX.len() * 2])
                    {
                        bytes.starts_with(&CALL_PROTECTION_BYTECODE_PREFIX)
                    } else {
                        false
                    }
                }
            };

            if has_call_protection {
                ignored.push(Offsets { start: 1, length: 20 });
            }

            ignored.sort_by_key(|o| o.start);

            let mut left = 0;
            for offset in ignored {
                let right = offset.start as usize;

                let matched = match deployed_code {
                    BytecodeObject::Bytecode(ref bytes) => bytes[left..right] == code[left..right],
                    BytecodeObject::Unlinked(ref bytes) => {
                        if let Ok(bytes) = Bytes::from_str(&bytes[left * 2..right * 2]) {
                            bytes == code[left..right]
                        } else {
                            false
                        }
                    }
                };

                if !matched {
                    return false;
                }

                left = right + offset.length as usize;
            }

            if left < code.len() {
                match deployed_code {
                    BytecodeObject::Bytecode(ref bytes) => bytes[left..] == code[left..],
                    BytecodeObject::Unlinked(ref bytes) => {
                        if let Ok(bytes) = Bytes::from_str(&bytes[left * 2..]) {
                            bytes == code[left..]
                        } else {
                            false
                        }
                    }
                }
            } else {
                true
            }
        })
    }

    /// Finds a contract which has the same contract name or identifier as `id`. If more than one is
    /// found, return error.
    pub fn find_by_name_or_identifier(
        &self,
        id: &str,
    ) -> Result<Option<ArtifactWithContractRef<'_>>> {
        let contracts = self
            .iter()
            .filter(|(artifact, _)| artifact.name == id || artifact.identifier() == id)
            .collect::<Vec<_>>();

        if contracts.len() > 1 {
            eyre::bail!("{id} has more than one implementation.");
        }

        Ok(contracts.first().cloned())
    }

    /// Finds abi for contract which has the same contract name or identifier as `id`.
    pub fn find_abi_by_name_or_identifier(&self, id: &str) -> Option<JsonAbi> {
        self.iter()
            .find(|(artifact, _)| {
                artifact.name.split(".").next().unwrap() == id || artifact.identifier() == id
            })
            .map(|(_, contract)| contract.abi.clone())
    }

    /// Finds abi by name or source path
    ///
    /// Returns the abi and the contract name.
    pub fn find_abi_by_name_or_src_path(&self, name_or_path: &str) -> Option<(JsonAbi, String)> {
        self.iter()
            .find(|(artifact, _)| {
                artifact.name == name_or_path || artifact.source == PathBuf::from(name_or_path)
            })
            .map(|(_, contract)| (contract.abi.clone(), contract.name.clone()))
    }

    /// Flattens the contracts into functions, events and errors.
    pub fn flatten(&self) -> (BTreeMap<Selector, Function>, BTreeMap<B256, Event>, JsonAbi) {
        let mut funcs = BTreeMap::new();
        let mut events = BTreeMap::new();
        let mut errors_abi = JsonAbi::new();
        for (_name, contract) in self.iter() {
            for func in contract.abi.functions() {
                funcs.insert(func.selector(), func.clone());
            }
            for event in contract.abi.events() {
                events.insert(event.selector(), event.clone());
            }
            for error in contract.abi.errors() {
                errors_abi.errors.entry(error.name.clone()).or_default().push(error.clone());
            }
        }
        (funcs, events, errors_abi)
    }
}

impl From<ProjectCompileOutput> for ContractsByArtifact {
    fn from(value: ProjectCompileOutput) -> Self {
        Self::new(value.into_artifacts().map(|(id, ar)| {
            (
                id,
                CompactContractBytecode {
                    abi: ar.abi,
                    bytecode: ar.bytecode,
                    deployed_bytecode: ar.deployed_bytecode,
                },
            )
        }))
    }
}

impl Deref for ContractsByArtifact {
    type Target = BTreeMap<ArtifactId, ContractData>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper type that maps an address to a contract identifier and contract ABI.
pub type ContractsByAddress = BTreeMap<Address, (String, JsonAbi)>;

/// Very simple fuzzy matching of contract bytecode.
///
/// Returns a value between `0.0` (identical) and `1.0` (completely different).
pub fn bytecode_diff_score<'a>(mut a: &'a [u8], mut b: &'a [u8]) -> f64 {
    // Make sure `a` is the longer one.
    if a.len() < b.len() {
        std::mem::swap(&mut a, &mut b);
    }

    // Account for different lengths.
    let mut n_different_bytes = a.len() - b.len();

    // If the difference is more than 32 bytes and more than 10% of the total length,
    // we assume the bytecodes are completely different.
    // This is a simple heuristic to avoid checking every byte when the lengths are very different.
    // 32 is chosen to be a reasonable minimum as it's the size of metadata hashes and one EVM word.
    if n_different_bytes > 32 && n_different_bytes * 10 > a.len() {
        return 1.0;
    }

    // Count different bytes.
    // SAFETY: `a` is longer than `b`.
    n_different_bytes += unsafe { count_different_bytes(a, b) };

    n_different_bytes as f64 / a.len() as f64
}

/// Returns the amount of different bytes between two slices.
///
/// # Safety
///
/// `a` must be at least as long as `b`.
unsafe fn count_different_bytes(a: &[u8], b: &[u8]) -> usize {
    // This could've been written as `std::iter::zip(a, b).filter(|(x, y)| x != y).count()`,
    // however this function is very hot, and has been written to be as primitive as
    // possible for lower optimization levels.

    let a_ptr = a.as_ptr();
    let b_ptr = b.as_ptr();
    let len = b.len();

    let mut sum = 0;
    let mut i = 0;
    while i < len {
        // SAFETY: `a` is at least as long as `b`, and `i` is in bound of `b`.
        sum += unsafe { *a_ptr.add(i) != *b_ptr.add(i) } as usize;
        i += 1;
    }
    sum
}

/// Returns contract name for a given contract identifier.
///
/// Artifact/Contract identifier can take the following form:
/// `<artifact file name>:<contract name>`, the `artifact file name` is the name of the json file of
/// the contract's artifact and the contract name is the name of the solidity contract, like
/// `SafeTransferLibTest.json:SafeTransferLibTest`
///
/// This returns the `contract name` part
///
/// # Example
///
/// ```
/// use foundry_common::*;
/// assert_eq!(
///     "SafeTransferLibTest",
///     get_contract_name("SafeTransferLibTest.json:SafeTransferLibTest")
/// );
/// ```
pub fn get_contract_name(id: &str) -> &str {
    id.rsplit(':').next().unwrap_or(id)
}

/// This returns the `file name` part, See [`get_contract_name`]
///
/// # Example
///
/// ```
/// use foundry_common::*;
/// assert_eq!(
///     "SafeTransferLibTest.json",
///     get_file_name("SafeTransferLibTest.json:SafeTransferLibTest")
/// );
/// ```
pub fn get_file_name(id: &str) -> &str {
    id.split(':').next().unwrap_or(id)
}

/// Helper function to convert CompactContractBytecode ~> ContractBytecodeSome
pub fn compact_to_contract(contract: CompactContractBytecode) -> Result<ContractBytecodeSome> {
    Ok(ContractBytecodeSome {
        abi: contract.abi.ok_or_else(|| eyre::eyre!("No contract abi"))?,
        bytecode: contract.bytecode.ok_or_else(|| eyre::eyre!("No contract bytecode"))?.into(),
        deployed_bytecode: contract
            .deployed_bytecode
            .ok_or_else(|| eyre::eyre!("No contract deployed bytecode"))?
            .into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytecode_diffing() {
        assert_eq!(bytecode_diff_score(b"a", b"a"), 0.0);
        assert_eq!(bytecode_diff_score(b"a", b"b"), 1.0);

        let a_100 = &b"a".repeat(100)[..];
        assert_eq!(bytecode_diff_score(a_100, &b"b".repeat(100)), 1.0);
        assert_eq!(bytecode_diff_score(a_100, &b"b".repeat(99)), 1.0);
        assert_eq!(bytecode_diff_score(a_100, &b"b".repeat(101)), 1.0);
        assert_eq!(bytecode_diff_score(a_100, &b"b".repeat(120)), 1.0);
        assert_eq!(bytecode_diff_score(a_100, &b"b".repeat(1000)), 1.0);

        let a_99 = &b"a".repeat(99)[..];
        assert!(bytecode_diff_score(a_100, a_99) <= 0.01);
    }

    #[test]
    fn find_by_deployed_code_exact_with_empty_deployed() {
        let contracts = ContractsByArtifact::new(vec![]);

        assert!(contracts.find_by_deployed_code_exact(&[]).is_none());
    }
}
