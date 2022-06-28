/// Gas reports
pub mod gas_report;

/// Coverage reports
pub mod coverage;

/// The Forge test runner
mod runner;
pub use runner::{ContractRunner, TestOptions};

/// Forge test runners for multiple contracts
mod multi_runner;
pub use multi_runner::{MultiContractRunner, MultiContractRunnerBuilder};

mod traits;
pub use traits::*;

pub mod result;

/// The Forge EVM backend
pub use foundry_evm::*;

#[cfg(test)]
mod test_helpers;
