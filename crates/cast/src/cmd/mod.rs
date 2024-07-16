//! Subcommands for cast
//!
//! All subcommands should respect the `foundry_config::Config`.
//! If a subcommand accepts values that are supported by the `Config`, then the subcommand should
//! implement `figment::Provider` which allows the subcommand to override the config's defaults, see
//! [`foundry_config::Config`].

pub mod access_list;
pub mod bind;
pub mod call;
pub mod create2;
pub mod estimate;
pub mod find_block;
pub mod interface;
pub mod logs;
pub mod mktx;
pub mod rpc;
pub mod run;
pub mod send;
pub mod storage;
pub mod wallet;
