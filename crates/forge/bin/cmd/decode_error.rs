use clap::Parser;
use eyre::{eyre, Result};
use foundry_cli::opts::{CompilerArgs, CoreBuildArgs};
use foundry_common::compile::ProjectCompiler;
use foundry_compilers::artifacts::output_selection::ContractOutputSelection;
use std::fmt;

use alloy_dyn_abi::ErrorExt;
use alloy_json_abi::Error;
use alloy_sol_types::{Panic, Revert, SolError};

macro_rules! spaced_print {
    ($($arg:tt)*) => {
        println!($($arg)*);
        println!();
    };
}

#[derive(Debug, Clone)]
enum RevertType {
    Revert,
    Panic,
    /// The 4 byte signature of the error
    Custom([u8; 4]),
}

impl fmt::Display for RevertType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RevertType::Revert => write!(f, "Revert"),
            RevertType::Panic => write!(f, "Panic"),
            RevertType::Custom(selector) => write!(f, "Custom(0x{})", hex::encode(selector)),
        }
    }
}

impl From<[u8; 4]> for RevertType {
    fn from(selector: [u8; 4]) -> Self {
        match selector {
            Revert::SELECTOR => RevertType::Revert,
            Panic::SELECTOR => RevertType::Panic,
            _ => RevertType::Custom(selector),
        }
    }
}

/// CLI arguments for `forge inspect`.
#[derive(Clone, Debug, Parser)]
pub struct DecodeError {
    /// The hex encoded revert data
    revert_data: String,

    /// All build arguments are supported
    #[command(flatten)]
    build: CoreBuildArgs,
}

impl DecodeError {
    pub fn run(self) -> Result<()> {
        let DecodeError { revert_data, build } = self;

        if revert_data.len() < 8 {
            return Err(eyre!("Revert data is too short"));
        }

        // convert to bytes and get the selector
        let data_bytes = hex::decode(revert_data.trim_start_matches("0x"))?;
        let selector: [u8; 4] = data_bytes[..4].try_into()?;

        trace!(target: "forge", "running forge decode-error on error type {}", RevertType::from(selector));

        // Make sure were gonna get the abi out
        let mut cos = build.compiler.extra_output;
        if !cos.iter().any(|selected| *selected == ContractOutputSelection::Abi) {
            cos.push(ContractOutputSelection::Abi);
        }

        // Build modified Args
        let modified_build_args = CoreBuildArgs {
            compiler: CompilerArgs { extra_output: cos, ..build.compiler },
            ..build
        };

        // Build the project
        if let Ok(project) = modified_build_args.project() {
            let compiler = ProjectCompiler::new().quiet(true);
            let output = compiler.compile(&project)?;

            // search the project for the error
            //
            // we want to search even it matches the builtin errors because there could be a
            // collision
            let found_errs = output
                .artifacts()
                .filter_map(|(name, artifact)| {
                    Some((
                        name,
                        artifact.abi.as_ref()?.errors.iter().find_map(|(_, err)| {
                            // check if we have an error with a matching selector
                            // there can only be one per artifact
                            err.iter().find(|err| err.selector() == selector)
                        })?,
                    ))
                })
                .collect::<Vec<_>>();

            if !found_errs.is_empty() {
                pretty_print_custom_errros(found_errs, &data_bytes);
            }
        } else {
            tracing::trace!("No project found")
        }

        // try to decode the builtin errors if it matches
        pretty_print_builtin_errors(selector.into(), &data_bytes);

        Ok(())
    }
}

fn pretty_print_custom_errros(found_errs: Vec<(String, &Error)>, data: &[u8]) {
    let mut failures = Vec::with_capacity(found_errs.len());
    let mut did_succeed = false;
    for (artifact, dyn_err) in found_errs {
        match dyn_err.decode_error(data) {
            Ok(decoded) => {
                did_succeed = true;

                print_line();
                println!("Artifact: {}", artifact);
                println!("Error Name: {}", dyn_err.name);
                for (param, value) in dyn_err.inputs.iter().zip(decoded.body.iter()) {
                    println!("      {}: {:?}", param.name, value);
                }
                println!(" ");
            }
            Err(e) => {
                tracing::error!("Error decoding dyn err: {}", e);
                failures.push(format!("decoding data for {} failed", dyn_err.signature()));
            }
        };
    }

    if !did_succeed {
        for failure in failures {
            tracing::error!("{}", failure);
        }
    }
}

fn pretty_print_builtin_errors(revert_type: RevertType, data: &[u8]) {
    match revert_type {
        RevertType::Revert => {
            if let Ok(revert) = Revert::abi_decode(data, true) {
                print_line();
                spaced_print!("{:#?}\n", revert);
            }
        }
        RevertType::Panic => {
            if let Ok(panic) = Panic::abi_decode(data, true) {
                print_line();
                spaced_print!("{:#?}", panic);
            }
        }
        _ => {}
    }
}

fn print_line() {
    spaced_print!("--------------------------------------------------------");
}
