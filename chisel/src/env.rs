use core::fmt;
use ethers_solc::{
    artifacts::{CompactBytecode, CompactContractBytecode},
    project_util::TempProject,
    Artifact,
};
use rustyline::Editor;
use std::rc::Rc;

/// Represents a parsed snippet of Solidity code.
#[derive(Debug)]
pub struct SolSnippet {
    pub source_unit: (solang_parser::pt::SourceUnit, Vec<solang_parser::pt::Comment>),
    pub raw: Rc<String>,
}

/// Display impl for `SolToken`
impl fmt::Display for SolSnippet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

/// A Chisel REPL environment.
pub struct ChiselEnv {
    /// The `TempProject` created for the REPL contract.
    pub project: TempProject,
    /// The `rustyline` Editor
    pub rl: Editor<()>,
    /// The current session
    /// A session contains an ordered vector of source units, parsed by the solang-parser,
    /// as well as the raw source.
    pub session: Vec<SolSnippet>,
}

/// Chisel REPL environment impl
impl ChiselEnv {
    /// Create a new `ChiselEnv` with a specified `solc` version.
    pub fn new(solc_version: &'static str) -> Self {
        // Create initialized temporary dapptools-style project
        let mut project = Self::create_temp_project();

        // Set project's solc version explicitly
        project.set_solc(solc_version);

        // Create a new rustyline Editor
        let rl = Self::create_rustyline_editor();

        // Return initialized ChiselEnv with set solc version
        Self { project, rl, session: Vec::default() }
    }

    /// Create a default `ChiselEnv`.
    pub fn default() -> Self {
        Self {
            project: Self::create_temp_project(),
            rl: Self::create_rustyline_editor(),
            session: Vec::default(),
        }
    }

    /// Runs the REPL contract within the executor
    /// TODO
    pub fn run_repl(&self) -> Result<(), &str> {
        // Recompile the project and ensure no errors occurred.
        // TODO: This is pretty slow. Need to speed it up.
        if let Ok(artifacts) = self.project.compile() {
            if artifacts.has_compiler_errors() {
                return Err("Failed to compile REPL contract.")
            }

            if let Some(contract) = artifacts.find_first("REPL") {
                let CompactContractBytecode { abi, bytecode, .. } =
                    contract.clone().into_contract_bytecode();

                let abi = abi.expect("No ABI for contract.");
                let bytecode = bytecode.expect("No bytecode for contract.").object.into_bytes().unwrap();

                println!("REPL Bytecode: {:?}", bytecode);
            } else {
                return Err("Could not find artifact for REPL contract.")
            }

            Ok(())
        } else {
            Err("Failed to compile REPL contract.")
        }
    }

    /// Render the full source code for the current session.
    /// TODO - Render source correctly rather than throwing
    /// everything into `run()`.
    pub fn contract_source(&self) -> String {
        format!(
            r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity {};
// TODO: Inherit `forge-std/Script.sol`
contract REPL {{
    function run() public {{
        {}
    }}
}}
        "#,
            "^0.8.17", // TODO: Grab version from TempProject's solc instance.
            self.session.iter().map(|t| t.to_string()).collect::<Vec<String>>().join("\n")
        )
    }

    /// Helper function to create a new temporary project with proper error handling.
    ///
    /// ### Panics
    ///
    /// Panics if the temporary project cannot be created.
    pub(crate) fn create_temp_project() -> TempProject {
        TempProject::dapptools_init().unwrap_or_else(|e| {
            tracing::error!(target: "chisel-env", "Failed to initialize temporary project! {:?}", e);
            panic!("failed to create a temporary project for the chisel environment! {e}");
        })
    }

    /// Helper function to create a new rustyline Editor with proper error handling.
    ///
    /// ### Panics
    ///
    /// Panics if the rustyline Editor cannot be created.
    pub(crate) fn create_rustyline_editor() -> Editor<()> {
        Editor::<()>::new().unwrap_or_else(|e| {
            tracing::error!(target: "chisel-env", "Failed to initialize rustyline Editor! {}", e);
            panic!("failed to create a rustyline Editor for the chisel environment! {e}");
        })
    }
}
