//! Uncategorised utilities.

use crate::compile::PathOrContractInfo;
use alloy_primitives::{keccak256, B256, U256};
use eyre::{OptionExt, Result};
use foundry_compilers::{
    artifacts::ConfigurableContractArtifact, utils::canonicalized, Project, ProjectCompileOutput,
};
use std::path::{Path, PathBuf};

/// Block on a future using the current tokio runtime on the current thread.
pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    block_on_handle(&tokio::runtime::Handle::current(), future)
}

/// Block on a future using the current tokio runtime on the current thread with the given handle.
pub fn block_on_handle<F: std::future::Future>(
    handle: &tokio::runtime::Handle,
    future: F,
) -> F::Output {
    tokio::task::block_in_place(|| handle.block_on(future))
}

/// Computes the storage slot as specified by `ERC-7201`, using the `erc7201` formula ID.
///
/// This is defined as:
///
/// ```text
/// erc7201(id: string) = keccak256(keccak256(id) - 1) & ~0xff
/// ```
///
/// # Examples
///
/// ```
/// use alloy_primitives::b256;
/// use foundry_common::erc7201;
///
/// assert_eq!(
///     erc7201("example.main"),
///     b256!("183a6125c38840424c4a85fa12bab2ab606c4b6d0e7cc73c0c06ba5300eab500"),
/// );
/// ```
pub fn erc7201(id: &str) -> B256 {
    let x = U256::from_be_bytes(keccak256(id).0) - U256::from(1);
    keccak256(x.to_be_bytes::<32>()) & B256::from(!U256::from(0xff))
}

/// Returns the canonicalized target path for the given identifier.
pub fn find_target_path(project: &Project, identifier: &PathOrContractInfo) -> Result<PathBuf> {
    match identifier {
        PathOrContractInfo::Path(path) => Ok(canonicalized(project.root().join(path))),
        PathOrContractInfo::ContractInfo(info) => {
            let path = project.find_contract_path(&info.name)?;
            Ok(path)
        }
    }
}

/// Returns the target artifact given the path and name.
pub fn find_matching_contract_artifact(
    output: &mut ProjectCompileOutput,
    target_path: &Path,
    target_name: Option<&str>,
) -> eyre::Result<ConfigurableContractArtifact> {
    if let Some(name) = target_name {
        output
            .remove(target_path, name)
            .ok_or_eyre(format!("Could not find artifact `{name}` in the compiled artifacts"))
    } else {
        let possible_targets = output
            .artifact_ids()
            .filter(|(id, _artifact)| id.source == target_path)
            .collect::<Vec<_>>();

        if possible_targets.len() == 1 {
            return Ok(possible_targets[0].1.clone());
        }

        if possible_targets.is_empty() {
            eyre::bail!("Could not find artifact linked to source `{target_path:?}` in the compiled artifacts");
        }

        // If all artifact_ids in `possible_targets` have the same name (without ".", indicates
        // additional compiler profiles), it means that there are multiple contracts in the
        // same file.
        let contract_name = possible_targets.first().unwrap().0.clone().name;
        if !contract_name.contains(".") &&
            possible_targets.iter().any(|(id, _)| id.name != contract_name)
        {
            eyre::bail!("Multiple contracts found in the same file, please specify the target <path>:<contract> or <contract>");
        }

        // Otherwise, we're dealing with additional compiler profiles wherein `id.source` is the
        // same but `id.path` is different.
        // First, try to find the one with default profile, it not, return the first one
        let default_target = possible_targets.iter().find_map(|(id, artifact)| {
            if id.profile == "default" {
                return Some(artifact)
            }
            None
        });

        if let Some(artifact) = default_target {
            return Ok((*artifact).clone());
        }

        Ok(possible_targets[0].1.clone())
    }
}
