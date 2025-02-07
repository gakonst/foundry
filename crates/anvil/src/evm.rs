use alloy_primitives::Address;
use revm::ContextPrecompile;
use std::{fmt::Debug, sync::Arc};

/// Object-safe trait that enables injecting extra precompiles when using
/// `anvil` as a library.
pub trait PrecompileFactory<DB: revm::Database>: Send + Sync + Unpin + Debug {
    /// Returns a set of precompiles to extend the EVM with.
    fn precompiles(&self) -> Vec<(Address, ContextPrecompile<DB>)>;
}

/// Appends a handler register to `evm` that injects the given `precompiles`.
///
/// This will add an additional handler that extends the default precompiles with the given set of
/// precompiles.
pub fn inject_precompiles<DB: revm::Database, I>(
    evm: &mut revm::Evm<'_, I, DB>,
    precompiles: Vec<(Address, ContextPrecompile<DB>)>,
) {
    evm.handler.append_handler_register_box(Box::new(move |handler| {
        let precompiles = precompiles.clone();
        let prev = handler.pre_execution.load_precompiles.clone();
        handler.pre_execution.load_precompiles = Arc::new(move || {
            let mut cx = prev();
            cx.extend(precompiles.iter().cloned().map(|(a, b)| (a, b.into())));
            cx
        });
    }));
}

#[cfg(test)]
mod tests {
    use crate::{evm::inject_precompiles, PrecompileFactory};
    use alloy_primitives::{Address, U256};
    use foundry_evm::revm::primitives::{address, Bytes, Precompile, PrecompileResult, SpecId};
    use revm::{
        primitives::{PrecompileError, PrecompileErrors, PrecompileOutput},
        ContextPrecompile, ContextStatefulPrecompileMut,
    };

    #[test]
    fn build_evm_with_extra_precompiles() {
        const PRECOMPILE_ADDR: Address = address!("0000000000000000000000000000000000000071");

        fn my_precompile(_bytes: &Bytes, _gas_limit: u64) -> PrecompileResult {
            Ok(PrecompileOutput { bytes: Bytes::new(), gas_used: 0 })
        }

        #[derive(Debug)]
        struct CustomPrecompileFactory;

        impl<DB: revm::Database> PrecompileFactory<DB> for CustomPrecompileFactory {
            fn precompiles(&self) -> Vec<(Address, ContextPrecompile<DB>)> {
                vec![(
                    PRECOMPILE_ADDR,
                    ContextPrecompile::Ordinary(Precompile::Standard(my_precompile)),
                )]
            }
        }

        let db = revm::db::EmptyDB::default();
        let env = Box::<revm::primitives::Env>::default();
        let spec = SpecId::LATEST;
        let handler_cfg = revm::primitives::HandlerCfg::new(spec);
        let inspector = revm::inspectors::NoOpInspector;
        let context = revm::Context::new(revm::EvmContext::new_with_env(db, env), inspector);
        let handler = revm::Handler::new(handler_cfg);
        let mut evm = revm::Evm::new(context, handler);
        assert!(!evm
            .handler
            .pre_execution()
            .load_precompiles()
            .addresses()
            .any(|&addr| addr == PRECOMPILE_ADDR));

        inject_precompiles(&mut evm, CustomPrecompileFactory.precompiles());
        assert!(evm
            .handler
            .pre_execution()
            .load_precompiles()
            .addresses()
            .any(|&addr| addr == PRECOMPILE_ADDR));

        let result = evm.transact().unwrap();
        assert!(result.result.is_success());
    }

    #[test]
    fn build_evm_with_extra_stateful_precompile() {
        const PRECOMPILE_ADDR: Address = address!("0000000000000000000000000000000000000071");
        const FROM_ADDR: Address = address!("000000000000000000000000000000000000000A");
        const TO_ADDR: Address = address!("000000000000000000000000000000000000000B");

        #[derive(Debug, Clone)]
        pub struct Transfer;

        impl<DB: revm::Database> ContextStatefulPrecompileMut<DB> for Transfer {
            fn call_mut(
                &mut self,
                bytes: &Bytes,
                _gas_price: u64,
                evmctx: &mut revm::InnerEvmContext<DB>,
            ) -> PrecompileResult {
                let amount = U256::from(123);

                evmctx.journaled_state.transfer(&FROM_ADDR, &TO_ADDR, amount, &mut evmctx.db);
                Ok(PrecompileOutput { gas_used: 100, bytes: bytes.clone() })
            }
        }

        #[derive(Debug)]
        struct CustomPrecompileFactory;

        impl<DB: revm::Database> PrecompileFactory<DB> for CustomPrecompileFactory {
            fn precompiles(&self) -> Vec<(Address, ContextPrecompile<DB>)> {
                vec![(PRECOMPILE_ADDR, ContextPrecompile::ContextStatefulMut(Box::new(Transfer)))]
            }
        }

        let db = revm::db::EmptyDB::default();
        let env = Box::<revm::primitives::Env>::default();
        let spec = SpecId::LATEST;
        let handler_cfg = revm::primitives::HandlerCfg::new(spec);
        let inspector = revm::inspectors::NoOpInspector;
        let context = revm::Context::new(revm::EvmContext::new_with_env(db, env), inspector);
        let handler = revm::Handler::new(handler_cfg);
        let mut evm = revm::Evm::new(context, handler);
        assert!(!evm
            .handler
            .pre_execution()
            .load_precompiles()
            .addresses()
            .any(|&addr| addr == PRECOMPILE_ADDR));

        inject_precompiles(&mut evm, CustomPrecompileFactory.precompiles());
        assert!(evm
            .handler
            .pre_execution()
            .load_precompiles()
            .addresses()
            .any(|&addr| addr == PRECOMPILE_ADDR));

        let result = evm.transact().unwrap();
        assert!(result.result.is_success());
    }
}
