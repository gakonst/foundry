//! Various utilities to decode test results.

use crate::abi::{Console, Vm};
use alloy_dyn_abi::JsonAbiExt;
use alloy_json_abi::{Error, JsonAbi};
use alloy_primitives::{hex, Log, Selector};
use alloy_sol_types::{SolCall, SolError, SolEventInterface, SolInterface, SolValue};
use foundry_common::{abi::get_error, selectors::OpenChainClient, SELECTOR_LEN};
use itertools::Itertools;
use revm::interpreter::InstructionResult;
use rustc_hash::FxHashMap;
use std::{
    sync::{mpsc, OnceLock},
    thread,
};
use tokio::runtime::Handle;

/// Decode a set of logs, only returning logs from DSTest logging events and Hardhat's `console.log`
pub fn decode_console_logs(logs: &[Log]) -> Vec<String> {
    logs.iter().filter_map(decode_console_log).collect()
}

/// Decode a single log.
///
/// This function returns [None] if it is not a DSTest log or the result of a Hardhat
/// `console.log`.
pub fn decode_console_log(log: &Log) -> Option<String> {
    Console::ConsoleEvents::decode_log(log, false).ok().map(|decoded| decoded.to_string())
}

/// Decodes revert data.
#[derive(Clone, Debug, Default)]
pub struct RevertDecoder {
    /// The custom errors to use for decoding.
    pub errors: FxHashMap<Selector, Vec<Error>>,
    pub open_chain_client: Option<OpenChainClient>,
}

impl Default for &RevertDecoder {
    fn default() -> Self {
        static EMPTY: OnceLock<RevertDecoder> = OnceLock::new();
        EMPTY.get_or_init(RevertDecoder::new)
    }
}

impl RevertDecoder {
    /// Creates a new, empty revert decoder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the ABIs to use for error decoding.
    ///
    /// Note that this is decently expensive as it will hash all errors for faster indexing.
    pub fn with_abis<'a>(mut self, abi: impl IntoIterator<Item = &'a JsonAbi>) -> Self {
        self.extend_from_abis(abi);
        self
    }

    /// Sets the ABI to use for error decoding.
    ///
    /// Note that this is decently expensive as it will hash all errors for faster indexing.
    pub fn with_abi(mut self, abi: &JsonAbi) -> Self {
        self.extend_from_abi(abi);
        self
    }

    /// Sets the ABI to use for error decoding, if it is present.
    ///
    /// Note that this is decently expensive as it will hash all errors for faster indexing.
    pub fn with_abi_opt(mut self, abi: Option<&JsonAbi>) -> Self {
        if let Some(abi) = abi {
            self.extend_from_abi(abi);
        }
        self
    }

    /// Extends the decoder with the given ABI's custom errors.
    pub fn extend_from_abis<'a>(&mut self, abi: impl IntoIterator<Item = &'a JsonAbi>) {
        for abi in abi {
            self.extend_from_abi(abi);
        }
    }

    /// Extends the decoder with the given ABI's custom errors.
    pub fn extend_from_abi(&mut self, abi: &JsonAbi) {
        for error in abi.errors() {
            self.push_error(error.clone());
        }
    }

    /// Adds a custom error to use for decoding.
    pub fn push_error(&mut self, error: Error) {
        self.errors.entry(error.selector()).or_default().push(error);
    }

    /// Tries to decode an error message from the given revert bytes.
    ///
    /// Note that this is just a best-effort guess, and should not be relied upon for anything other
    /// than user output.
    pub fn decode(&self, err: &[u8], status: Option<InstructionResult>) -> String {
        self.maybe_decode(err, status).unwrap_or_else(|| {
            if err.is_empty() {
                "<empty revert data>".to_string()
            } else {
                trimmed_hex(err)
            }
        })
    }

    /// Tries to decode an error message from the given revert bytes.
    ///
    /// See [`decode`](Self::decode) for more information.
    pub fn maybe_decode(&self, err: &[u8], status: Option<InstructionResult>) -> Option<String> {
        if err.len() < SELECTOR_LEN {
            if let Some(status) = status {
                if !status.is_ok() {
                    return Some(format!("EvmError: {status:?}"));
                }
            }
            return if err.is_empty() {
                None
            } else {
                Some(format!("custom error bytes {}", hex::encode_prefixed(err)))
            };
        }

        if err == crate::constants::MAGIC_SKIP {
            // Also used in forge fuzz runner
            return Some("SKIPPED".to_string());
        }

        // Solidity's `Error(string)` or `Panic(uint256)`
        if let Ok(e) = alloy_sol_types::GenericContractError::abi_decode(err, false) {
            return Some(e.to_string());
        }

        let (selector, data) = err.split_at(SELECTOR_LEN);
        let selector: &[u8; 4] = selector.try_into().unwrap();

        match *selector {
            // `CheatcodeError(string)`
            Vm::CheatcodeError::SELECTOR => {
                let e = Vm::CheatcodeError::abi_decode_raw(data, false).ok()?;
                return Some(e.message);
            }
            // `expectRevert(bytes)`
            Vm::expectRevert_2Call::SELECTOR => {
                let e = Vm::expectRevert_2Call::abi_decode_raw(data, false).ok()?;
                return self.maybe_decode(&e.revertData[..], status);
            }
            // `expectRevert(bytes4)`
            Vm::expectRevert_1Call::SELECTOR => {
                let e = Vm::expectRevert_1Call::abi_decode_raw(data, false).ok()?;
                return self.maybe_decode(&e.revertData[..], status);
            }
            _ => {}
        }

        // Custom errors.
        if let Some(errors) = self.errors.get(selector) {
            for error in errors {
                // If we don't decode, don't return an error, try to decode as a string later.
                if let Ok(decoded) = error.abi_decode_input(data, false) {
                    return Some(format!(
                        "{}({})",
                        error.name,
                        decoded.iter().map(foundry_common::fmt::format_token).format(", ")
                    ));
                }
            }
        }

        // ABI-encoded `string`.
        if let Ok(s) = String::abi_decode(err, true) {
            return Some(s);
        }

        // ASCII string.
        if err.is_ascii() {
            return Some(std::str::from_utf8(err).unwrap().to_string());
        }

        // Generic custom error.
        Some(format!(
            "custom error {}:{}",
            hex::encode(selector),
            std::str::from_utf8(data).map_or_else(|_| trimmed_hex(data), String::from)
        ))
    }

    pub fn may_decode_using_open_chain(&self, err: &[u8]) -> Option<String> {
        let (selector, data) = err.split_at(SELECTOR_LEN);
        // try from https://openchain.xyz
        if let Some(client) = self.open_chain_client.clone() {
            if let Ok(handle) = Handle::try_current() {
                let (tx, rx) = mpsc::channel();
                let encoded_selector = hex::encode(selector);
                thread::spawn(move || {
                    let result =
                        handle.block_on(client.decode_function_selector(&encoded_selector));
                    tx.send(result).unwrap();
                });

                let result = match rx.recv() {
                    Ok(Ok(sigs)) => Some(sigs),
                    Ok(Err(_)) | Err(_) => None,
                };
                if let Some(sigs) = result {
                    for sig in sigs {
                        if let Ok(error) = get_error(&sig) {
                            if let Ok(decoded) = error.abi_decode_input(data, true) {
                                return Some(format!(
                                    "{}({})",
                                    error.name,
                                    decoded
                                        .iter()
                                        .map(foundry_common::fmt::format_token)
                                        .format(", ")
                                ));
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

fn trimmed_hex(s: &[u8]) -> String {
    let n = 32;
    if s.len() <= n {
        hex::encode(s)
    } else {
        format!(
            "{}…{} ({} bytes)",
            &hex::encode(&s[..n / 2]),
            &hex::encode(&s[s.len() - n / 2..]),
            s.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_trimmed_hex() {
        assert_eq!(trimmed_hex(&hex::decode("1234567890").unwrap()), "1234567890");
        assert_eq!(
            trimmed_hex(&hex::decode("492077697368207275737420737570706F72746564206869676865722D6B696E646564207479706573").unwrap()),
            "49207769736820727573742073757070…6865722d6b696e646564207479706573 (41 bytes)"
        );
    }
}
