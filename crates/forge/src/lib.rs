//! Forge is a fast and flexible Ethereum testing framework.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[macro_use]
extern crate foundry_common;

#[macro_use]
extern crate tracing;

pub mod coverage;

pub mod gas_report;

pub mod multi_runner;
pub use multi_runner::{MultiContractRunner, MultiContractRunnerBuilder};

mod runner;
pub use runner::ContractRunner;

mod progress;
pub mod result;

// TODO: remove
pub use foundry_common::traits::TestFilter;
pub use foundry_evm::*;

mod lockfile;
pub use lockfile::{DepIdentifier, DepMap, Lockfile, FOUNDRY_LOCK};
