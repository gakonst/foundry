//! `forge` subcommands.
//!
//! All subcommands should respect the `foundry_config::Config`.
//! If a subcommand accepts values that are supported by the `Config`, then the subcommand should
//! implement `figment::Provider` which allows the subcommand to override the config's defaults, see
//! [`foundry_config::Config`].

pub mod bind;
pub mod bind_json;
pub mod build;
pub mod cache;
pub mod clone;
pub mod compiler;
pub mod config;
pub mod coverage;
pub mod create;
pub mod doc;
pub mod eip712;
pub mod flatten;
pub mod fmt;
pub mod geiger;
pub mod generate;
pub mod init;
pub mod inspect;
pub mod install;
pub mod lint;
pub mod remappings;
pub mod remove;
pub mod selectors;
pub mod snapshot;
pub mod soldeer;
pub mod test;
pub mod tree;
pub mod update;
pub mod watch;
