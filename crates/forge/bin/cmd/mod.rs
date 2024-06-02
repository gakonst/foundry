//! Subcommands for forge
//!
//! All subcommands should respect the `foundry_config::Config`.
//! If a subcommand accepts values that are supported by the `Config`, then the subcommand should
//! implement `figment::Provider` which allows the subcommand to override the config's defaults, see
//! [`foundry_config::Config`].
//!
//! See [`BuildArgs`] for a reference implementation.
//! And [`DebugArgs`] for how to merge `Providers`.
//!
//! # Example
//!
//! create a `clap` subcommand into a `figment::Provider` and integrate it in the
//! `foundry_config::Config`:
//!
//! ```
//! use clap::Parser;
//! use forge::executor::opts::EvmOpts;
//! use foundry_cli::cmd::forge::build::BuildArgs;
//! use foundry_common::evm::EvmArgs;
//! use foundry_config::{figment::Figment, *};
//!
//! // A new clap subcommand that accepts both `EvmArgs` and `BuildArgs`
//! #[derive(Clone, Debug, Parser)]
//! pub struct MyArgs {
//!     #[command(flatten)]
//!     evm_opts: EvmArgs,
//!     #[command(flatten)]
//!     opts: BuildArgs,
//! }
//!
//! // add `Figment` and `Config` converters
//! foundry_config::impl_figment_convert!(MyArgs, opts, evm_opts);
//! let args = MyArgs::parse_from(["build"]);
//!
//! let figment: Figment = From::from(&args);
//! let evm_opts = figment.extract::<EvmOpts>().unwrap();
//!
//! let config: Config = From::from(&args);
//! ```

pub mod bind;
pub mod build;
pub mod cache;
pub mod clone;
pub mod config;
pub mod coverage;
pub mod create;
pub mod debug;
pub mod decode_error;
pub mod doc;
pub mod flatten;
pub mod fmt;
pub mod geiger;
pub mod generate;
pub mod init;
pub mod inspect;
pub mod install;
pub mod remappings;
pub mod remove;
pub mod selectors;
pub mod snapshot;
pub mod test;
pub mod tree;
pub mod update;
pub mod watch;
