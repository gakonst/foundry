//! # foundry-evm-core
//!
//! Core EVM abstractions.

#![warn(unused_crate_dependencies)]

use auto_impl::auto_impl;
use revm::{inspectors::NoOpInspector, interpreter::CreateInputs, Database, EvmContext, Inspector};
use revm_inspectors::access_list::AccessListInspector;

#[macro_use]
extern crate tracing;

mod ic;

pub mod abi;
pub mod arbitrum;
pub mod backend;
pub mod constants;
pub mod debug;
pub mod decode;
pub mod fork;
pub mod opcodes;
pub mod opts;
pub mod snapshot;
pub mod utils;

/// An extension trait that allows us to add additional hooks to Inspector for later use in
/// handlers.
#[auto_impl(&mut, Box)]
pub trait InspectorExt<DB: Database>: Inspector<DB> {
    /// Determines whether the `DEFAULT_CREATE2_DEPLOYER` should be used for a CREATE2 frame.
    ///
    /// If this function returns true, we'll replace CREATE2 frame with a CALL frame to CREATE2
    /// factory.
    fn should_use_create2_factory(
        &mut self,
        _context: &mut EvmContext<DB>,
        _inputs: &mut CreateInputs,
    ) -> bool {
        false
    }
}

impl<DB: Database> InspectorExt<DB> for NoOpInspector {}
impl<DB: Database> InspectorExt<DB> for AccessListInspector {}
