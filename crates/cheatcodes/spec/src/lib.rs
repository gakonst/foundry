//! Cheatcode specification for Foundry.

#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

mod cheatcode;
pub use cheatcode::{Cheatcode, CheatcodeDef, Group, Requires, Safety, Status};

mod function;
pub use function::{Function, Mutability, Visibility};

mod items;
pub use items::{Enum, EnumVariant, Error, Event, Struct, StructField};

mod vm;
pub use vm::Vm;

// The `cheatcodes.json` schema.
/// Foundry cheatcodes. Learn more: <https://book.getfoundry.sh/cheatcodes/>
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Cheatcodes<'a> {
    /// Cheatcode errors.
    #[serde(borrow)]
    pub errors: Cow<'a, [Error<'a>]>,
    /// Cheatcode events.
    #[serde(borrow)]
    pub events: Cow<'a, [Event<'a>]>,
    /// Cheatcode enums.
    #[serde(borrow)]
    pub enums: Cow<'a, [Enum<'a>]>,
    /// Cheatcode structs.
    #[serde(borrow)]
    pub structs: Cow<'a, [Struct<'a>]>,
    /// All the cheatcodes.
    #[serde(borrow)]
    pub cheatcodes: Cow<'a, [Cheatcode<'a>]>,
}

impl fmt::Display for Cheatcodes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in self.errors.iter() {
            writeln!(f, "{error}")?;
        }
        for event in self.events.iter() {
            writeln!(f, "{event}")?;
        }
        for enumm in self.enums.iter() {
            writeln!(f, "{enumm}")?;
        }
        for strukt in self.structs.iter() {
            writeln!(f, "{strukt}")?;
        }
        for cheatcode in self.cheatcodes.iter() {
            writeln!(f, "{}", cheatcode.func)?;
        }
        Ok(())
    }
}

impl Default for Cheatcodes<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl Cheatcodes<'static> {
    /// Returns the default cheatcodes.
    pub fn new() -> Self {
        Self {
            // unfortunately technology has not yet advanced to the point where we can get all
            // items of a certain type in a module, so we have to hardcode them here
            structs: Cow::Owned(vec![
                Vm::Log::STRUCT.clone(),
                Vm::Rpc::STRUCT.clone(),
                Vm::EthGetLogs::STRUCT.clone(),
                Vm::DirEntry::STRUCT.clone(),
                Vm::FsMetadata::STRUCT.clone(),
                Vm::Wallet::STRUCT.clone(),
                Vm::FfiResult::STRUCT.clone(),
                Vm::ChainInfo::STRUCT.clone(),
                Vm::AccountAccess::STRUCT.clone(),
                Vm::StorageAccess::STRUCT.clone(),
                Vm::Gas::STRUCT.clone(),
                Vm::DebugStep::STRUCT.clone(),
                Vm::BroadcastTxSummary::STRUCT.clone(),
                Vm::SignedDelegation::STRUCT.clone(),
                Vm::PotentialRevert::STRUCT.clone(),
            ]),
            enums: Cow::Owned(vec![
                Vm::CallerMode::ENUM.clone(),
                Vm::AccountAccessKind::ENUM.clone(),
                Vm::ForgeContext::ENUM.clone(),
                Vm::BroadcastTxType::ENUM.clone(),
            ]),
            errors: Vm::VM_ERRORS.iter().copied().cloned().collect(),
            events: Cow::Borrowed(&[]),
            // events: Vm::VM_EVENTS.iter().copied().cloned().collect(),
            cheatcodes: Vm::CHEATCODES.iter().copied().cloned().collect(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    const JSON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/cheatcodes.json");
    #[cfg(feature = "schema")]
    const SCHEMA_PATH: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/cheatcodes.schema.json");
    const IFACE_PATH: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../../testdata/cheats/Vm.sol");

    /// Generates the `cheatcodes.json` file contents.
    fn json_cheatcodes() -> String {
        serde_json::to_string_pretty(&Cheatcodes::new()).unwrap()
    }

    /// Generates the [cheatcodes](json_cheatcodes) JSON schema.
    #[cfg(feature = "schema")]
    fn json_schema() -> String {
        serde_json::to_string_pretty(&schemars::schema_for!(Cheatcodes<'_>)).unwrap()
    }

    fn sol_iface() -> String {
        let mut cheats = Cheatcodes::new();
        cheats.errors = Default::default(); // Skip errors to allow <0.8.4.
        let cheats = cheats.to_string().trim().replace('\n', "\n    ");
        format!(
            "\
// Automatically generated from `foundry-cheatcodes` Vm definitions. Do not modify manually.
// This interface is just for internal testing purposes. Use `forge-std` instead.

// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity >=0.6.2 <0.9.0;
pragma experimental ABIEncoderV2;

interface Vm {{
    {cheats}
}}
"
        )
    }

    #[test]
    fn spec_up_to_date() {
        ensure_file_contents(Path::new(JSON_PATH), &json_cheatcodes());
    }

    #[test]
    #[cfg(feature = "schema")]
    fn schema_up_to_date() {
        ensure_file_contents(Path::new(SCHEMA_PATH), &json_schema());
    }

    #[test]
    fn iface_up_to_date() {
        ensure_file_contents(Path::new(IFACE_PATH), &sol_iface());
    }

    /// Checks that the `file` has the specified `contents`. If that is not the
    /// case, updates the file and then fails the test.
    fn ensure_file_contents(file: &Path, contents: &str) {
        if let Ok(old_contents) = fs::read_to_string(file) {
            if normalize_newlines(&old_contents) == normalize_newlines(contents) {
                // File is already up to date.
                return
            }
        }

        eprintln!("\n\x1b[31;1merror\x1b[0m: {} was not up-to-date, updating\n", file.display());
        if std::env::var("CI").is_ok() {
            eprintln!("    NOTE: run `cargo cheats` locally and commit the updated files\n");
        }
        if let Some(parent) = file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(file, contents).unwrap();
        panic!("some file was not up to date and has been updated, simply re-run the tests");
    }

    fn normalize_newlines(s: &str) -> String {
        s.replace("\r\n", "\n")
    }
}
