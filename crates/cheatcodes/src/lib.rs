//! # foundry-cheatcodes
//!
//! Foundry cheatcodes implementations.

#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![allow(elided_lifetimes_in_paths)] // Cheats context uses 3 lifetimes

#[macro_use]
extern crate foundry_common;

#[macro_use]
pub extern crate foundry_cheatcodes_spec as spec;

#[macro_use]
extern crate tracing;

use alloy_primitives::Address;
use foundry_evm_core::backend::DatabaseExt;
use revm::{ContextPrecompiles, InnerEvmContext};
use spec::{Requires, Status};

pub use config::CheatsConfig;
pub use error::{Error, ErrorKind, Result};
pub use inspector::{
    BroadcastableTransaction, BroadcastableTransactions, Cheatcodes, CheatcodesExecutor, Context,
};
pub use spec::{CheatcodeDef, Vm};
pub use Vm::ForgeContext;

#[macro_use]
mod error;

mod base64;

mod config;

mod crypto;

mod env;
pub use env::set_execution_context;

mod evm;

mod fs;

mod inspector;

mod json;

mod script;
pub use script::{Wallets, WalletsInner};

mod string;

mod test;
pub use test::expect::ExpectedCallTracker;

mod toml;

mod utils;

/// Cheatcode implementation.
pub(crate) trait Cheatcode: CheatcodeDef + DynCheatcode {
    /// Applies this cheatcode to the given state.
    ///
    /// Implement this function if you don't need access to the EVM data.
    fn apply(&self, state: &mut Cheatcodes) -> Result {
        let _ = state;
        unimplemented!("{}", Self::CHEATCODE.func.id)
    }

    /// Applies this cheatcode to the given context.
    ///
    /// Implement this function if you need access to the EVM data.
    #[inline(always)]
    fn apply_stateful(&self, ccx: &mut CheatsCtxt) -> Result {
        self.apply(ccx.state)
    }

    /// Applies this cheatcode to the given context and executor.
    ///
    /// Implement this function if you need access to the executor.
    #[inline(always)]
    fn apply_full(&self, ccx: &mut CheatsCtxt, executor: &mut dyn CheatcodesExecutor) -> Result {
        let _ = executor;
        self.apply_stateful(ccx)
    }
}

pub(crate) trait DynCheatcode: 'static {
    fn cheatcode(&self) -> &'static spec::Cheatcode<'static>;

    fn as_debug(&self) -> &dyn std::fmt::Debug;

    fn dyn_apply(&self, ccx: &mut CheatsCtxt, executor: &mut dyn CheatcodesExecutor) -> Result;
}

impl<T: Cheatcode> DynCheatcode for T {
    #[inline]
    fn cheatcode(&self) -> &'static spec::Cheatcode<'static> {
        Self::CHEATCODE
    }

    #[inline]
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    #[inline]
    fn dyn_apply(&self, ccx: &mut CheatsCtxt, executor: &mut dyn CheatcodesExecutor) -> Result {
        self.apply_full(ccx, executor)
    }
}

impl dyn DynCheatcode {
    pub(crate) fn name(&self) -> &'static str {
        self.cheatcode().func.signature.split('(').next().unwrap()
    }

    pub(crate) fn id(&self) -> &'static str {
        self.cheatcode().func.id
    }

    pub(crate) fn signature(&self) -> &'static str {
        self.cheatcode().func.signature
    }

    pub(crate) fn status(&self) -> &Status<'static> {
        &self.cheatcode().status
    }

    pub(crate) fn requires(&self) -> &Requires {
        &self.cheatcode().requires
    }
}

/// The cheatcode context, used in `Cheatcode`.
pub struct CheatsCtxt<'cheats, 'evm, 'db, 'db2> {
    /// The cheatcodes inspector state.
    pub(crate) state: &'cheats mut Cheatcodes,
    /// The EVM data.
    pub(crate) ecx: &'evm mut InnerEvmContext<&'db mut (dyn DatabaseExt + 'db2)>,
    /// The precompiles context.
    pub(crate) precompiles: &'evm mut ContextPrecompiles<&'db mut (dyn DatabaseExt + 'db2)>,
    /// The original `msg.sender`.
    pub(crate) caller: Address,
    /// Gas limit of the current cheatcode call.
    pub(crate) gas_limit: u64,
}

impl<'db, 'db2> std::ops::Deref for CheatsCtxt<'_, '_, 'db, 'db2> {
    type Target = InnerEvmContext<&'db mut (dyn DatabaseExt + 'db2)>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.ecx
    }
}

impl std::ops::DerefMut for CheatsCtxt<'_, '_, '_, '_> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.ecx
    }
}

impl CheatsCtxt<'_, '_, '_, '_> {
    #[inline]
    pub(crate) fn is_precompile(&self, address: &Address) -> bool {
        self.precompiles.contains(address)
    }
}
