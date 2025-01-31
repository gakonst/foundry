use super::Function;
use alloy_sol_types::SolCall;
use serde::{Deserialize, Serialize};

/// Cheatcode definition trait. Implemented by all [`Vm`](crate::Vm) functions.
pub trait CheatcodeDef: std::fmt::Debug + Clone + SolCall {
    /// The static cheatcode definition.
    const CHEATCODE: &'static Cheatcode<'static>;
}

/// Specification of a single cheatcode. Extends [`Function`] with additional metadata.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[non_exhaustive]
pub struct Cheatcode<'a> {
    // Automatically-generated fields.
    /// The Solidity function declaration.
    #[serde(borrow)]
    pub func: Function<'a>,

    // Manually-specified fields.
    /// The group that the cheatcode belongs to.
    pub group: Group,
    /// The current status of the cheatcode. E.g. whether it is stable or experimental, etc.
    pub status: Status<'a>,
    /// Whether the cheatcode is safe to use inside of scripts. E.g. it does not change state in an
    /// unexpected way.
    pub safety: Safety,
    /// The external requirements of the cheatcode.
    pub requires: Requires,
}

/// The status of a cheatcode.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Status<'a> {
    /// The cheatcode and its API is currently stable.
    Stable,
    /// The cheatcode is unstable, meaning it may contain bugs and may break its API on any
    /// release.
    ///
    /// Use of experimental cheatcodes will result in a warning.
    Experimental,
    /// The cheatcode has been deprecated, meaning it will be removed in a future release.
    ///
    /// Contains the optional reason for deprecation.
    ///
    /// Use of deprecated cheatcodes is discouraged and will result in a warning.
    Deprecated(Option<&'a str>),
    /// The cheatcode has been removed and is no longer available for use.
    ///
    /// Use of removed cheatcodes will result in a hard error.
    Removed,
    /// The cheatcode is only used internally for foundry testing and may be changed or removed at
    /// any time.
    ///
    /// Use of internal cheatcodes is discouraged and will result in a warning.
    Internal,
}

/// Cheatcode groups.
/// Initially derived and modified from inline comments in [`forge-std`'s `Vm.sol`][vmsol].
///
/// [vmsol]: https://github.com/foundry-rs/forge-std/blob/dcb0d52bc4399d37a6545848e3b8f9d03c77b98d/src/Vm.sol
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Group {
    /// Cheatcodes that read from, or write to the current EVM execution state.
    ///
    /// Examples: any of the `record` cheatcodes, `chainId`, `coinbase`.
    ///
    /// Safety: ambiguous, depends on whether the cheatcode is read-only or not.
    Evm,
    /// Cheatcodes that interact with how a test is run.
    ///
    /// Examples: `assume`, `skip`, `expectRevert`.
    ///
    /// Safety: ambiguous, depends on whether the cheatcode is read-only or not.
    Testing,
    /// Cheatcodes that interact with how a script is run.
    ///
    /// Examples: `broadcast`, `startBroadcast`, `stopBroadcast`.
    ///
    /// Safety: safe.
    Scripting,
    /// Cheatcodes that interact with the OS or filesystem.
    ///
    /// Examples: `ffi`, `projectRoot`, `writeFile`.
    ///
    /// Safety: safe.
    Filesystem,
    /// Cheatcodes that interact with the program's environment variables.
    ///
    /// Examples: `setEnv`, `envBool`, `envOr`.
    ///
    /// Safety: safe.
    Environment,
    /// Utility cheatcodes that deal with string parsing and manipulation.
    ///
    /// Examples: `toString`. `parseBytes`.
    ///
    /// Safety: safe.
    String,
    /// Utility cheatcodes that deal with parsing values from and converting values to JSON.
    ///
    /// Examples: `serializeJson`, `parseJsonUint`, `writeJson`.
    ///
    /// Safety: safe.
    Json,
    /// Utility cheatcodes that deal with parsing values from and converting values to TOML.
    ///
    /// Examples: `parseToml`, `writeToml`.
    ///
    /// Safety: safe.
    Toml,
    /// Cryptography-related cheatcodes.
    ///
    /// Examples: `sign*`.
    ///
    /// Safety: safe.
    Crypto,
    /// Generic, uncategorized utilities.
    ///
    /// Examples: `toString`, `parse*`, `serialize*`.
    ///
    /// Safety: safe.
    Utilities,
}

impl Group {
    /// Returns the safety of this cheatcode group.
    ///
    /// Some groups are inherently safe or unsafe, while others are ambiguous and will return
    /// `None`.
    #[inline]
    pub const fn safety(self) -> Option<Safety> {
        match self {
            Self::Evm | Self::Testing => None,
            Self::Scripting |
            Self::Filesystem |
            Self::Environment |
            Self::String |
            Self::Json |
            Self::Toml |
            Self::Crypto |
            Self::Utilities => Some(Safety::Safe),
        }
    }

    /// Returns this value as a string.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Evm => "evm",
            Self::Testing => "testing",
            Self::Scripting => "scripting",
            Self::Filesystem => "filesystem",
            Self::Environment => "environment",
            Self::String => "string",
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Crypto => "crypto",
            Self::Utilities => "utilities",
        }
    }
}

// TODO: Find a better name for this
/// Cheatcode safety.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Safety {
    /// The cheatcode is not safe to use in scripts.
    Unsafe,
    /// The cheatcode is safe to use in scripts.
    #[default]
    Safe,
}

impl Safety {
    /// Returns this value as a string.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Unsafe => "unsafe",
        }
    }

    /// Returns whether this value is safe.
    #[inline]
    pub const fn is_safe(self) -> bool {
        matches!(self, Self::Safe)
    }
}

/// The requirements of a cheatcode it requires to run.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Requires {
    /// The cheatcode has no requirements.
    None,
    /// The cheatcode requires isolation mode to be enabled.
    Isolation,
}

impl Requires {
    /// Returns this value as a string.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Isolation => "isolation",
        }
    }
}
