use crate::{opts::forge::ContractInfo, suggestions};
use clap::Parser;
use ethers::{
    abi::Abi,
    prelude::artifacts::{CompactBytecode, CompactDeployedBytecode},
    solc::{
        artifacts::CompactContractBytecode, cache::SolFilesCache, Project, ProjectCompileOutput,
    },
};
use futures::future::BoxFuture;
use std::{path::PathBuf, time::Duration};

/// Common trait for all cli commands
pub trait Cmd: clap::Parser + Sized {
    type Output;
    fn run(self) -> eyre::Result<Self::Output>;
}

/// Given a project and its compiled artifacts, proceeds to return the ABI, Bytecode and
/// Runtime Bytecode of the given contract.
#[track_caller]
pub fn read_artifact(
    project: &Project,
    compiled: ProjectCompileOutput,
    contract: ContractInfo,
) -> eyre::Result<(Abi, CompactBytecode, CompactDeployedBytecode)> {
    Ok(match contract.path {
        Some(path) => get_artifact_from_path(project, path, contract.name)?,
        None => get_artifact_from_name(contract, compiled)?,
    })
}

/// Helper function for finding a contract by ContractName
// TODO: Is there a better / more ergonomic way to get the artifacts given a project and a
// contract name?
fn get_artifact_from_name(
    contract: ContractInfo,
    compiled: ProjectCompileOutput,
) -> eyre::Result<(Abi, CompactBytecode, CompactDeployedBytecode)> {
    let mut contract_artifact = None;
    let mut alternatives = Vec::new();

    for (artifact_id, artifact) in compiled.into_artifacts() {
        if artifact_id.name == contract.name {
            if contract_artifact.is_some() {
                eyre::bail!(
                    "contract with duplicate name `{}`. please pass the path instead",
                    contract.name
                )
            }
            contract_artifact = Some(artifact);
        } else {
            alternatives.push(artifact_id.name);
        }
    }

    if let Some(artifact) = contract_artifact {
        let abi = artifact
            .abi
            .map(Into::into)
            .ok_or_else(|| eyre::eyre!("abi not found for {}", contract.name))?;

        let code = artifact
            .bytecode
            .ok_or_else(|| eyre::eyre!("bytecode not found for {}", contract.name))?;

        let deployed_code = artifact
            .deployed_bytecode
            .ok_or_else(|| eyre::eyre!("bytecode not found for {}", contract.name))?;
        return Ok((abi, code, deployed_code));
    }

    let mut err = format!("could not find artifact: `{}`", contract.name);
    if let Some(suggestion) = suggestions::did_you_mean(&contract.name, &alternatives).pop() {
        err = format!(
            r#"{}

        Did you mean `{}`?"#,
            err, suggestion
        );
    }
    eyre::bail!(err)
}

/// Find using src/ContractSource.sol:ContractName
fn get_artifact_from_path(
    project: &Project,
    contract_path: String,
    contract_name: String,
) -> eyre::Result<(Abi, CompactBytecode, CompactDeployedBytecode)> {
    // Get sources from the requested location
    let abs_path = dunce::canonicalize(PathBuf::from(contract_path))?;

    let cache = SolFilesCache::read_joined(&project.paths)?;

    // Read the artifact from disk
    let artifact: CompactContractBytecode = cache.read_artifact(abs_path, &contract_name)?;

    Ok((
        artifact
            .abi
            .ok_or_else(|| eyre::Error::msg(format!("abi not found for {contract_name}")))?,
        artifact
            .bytecode
            .ok_or_else(|| eyre::Error::msg(format!("bytecode not found for {contract_name}")))?,
        artifact
            .deployed_bytecode
            .ok_or_else(|| eyre::Error::msg(format!("bytecode not found for {contract_name}")))?,
    ))
}

/// A type that keeps track of attempts
#[derive(Debug, Clone, Parser)]
pub struct RetryArgs {
    #[clap(
        long,
        help = "Number of attempts for retrying",
        default_value = "1",
        validator = u32_validator(1, 10)
    )]
    retries: u32,

    #[clap(
        long,
        help = "Optional timeout to apply inbetween attempts in seconds.",
        validator = u32_validator(0, 30)
    )]
    delay: Option<u32>,
}

fn u32_validator(min: u32, max: u32) -> impl FnMut(&str) -> eyre::Result<()> {
    return move |v: &str| -> eyre::Result<()> {
        let v = v.parse::<u32>()?;
        if v >= min && v <= max {
            Ok(())
        } else {
            Err(eyre::eyre!("Expected between {} and {} inclusive.", min, max))
        }
    };
}

/// Sample retry logic implementation
impl RetryArgs {
    pub fn new(retries: u32, delay: Option<u32>) -> Self {
        RetryArgs { retries, delay }
    }

    fn handle_err(&mut self, err: eyre::Report) {
        self.retries -= 1;
        tracing::warn!(
            "erroneous attempt ({} tries remaining): {}",
            self.retries,
            err.root_cause()
        );
        if let Some(delay) = self.delay {
            std::thread::sleep(Duration::from_secs(delay.into()));
        }
    }

    pub fn run<T, F>(mut self, mut callback: F) -> eyre::Result<T>
    where
        F: FnMut() -> eyre::Result<T>,
    {
        loop {
            match callback() {
                Err(e) if self.retries > 0 => self.handle_err(e),
                res => return res,
            }
        }
    }

    pub async fn run_async<'a, T, F>(mut self, mut callback: F) -> eyre::Result<T>
    where
        F: FnMut() -> BoxFuture<'a, eyre::Result<T>>,
    {
        loop {
            match callback().await {
                Err(e) if self.retries > 0 => self.handle_err(e),
                res => return res,
            };
        }
    }
}
