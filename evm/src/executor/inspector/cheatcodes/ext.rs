use crate::{
    abi::HEVMCalls,
    error,
    executor::inspector::{cheatcodes::util, Cheatcodes},
};
use bytes::Bytes;
use ethers::{
    abi::{self, AbiEncode, JsonAbi, ParamType, Token},
    prelude::artifacts::CompactContractBytecode,
    types::*,
};
use foundry_common::{fmt::*, fs, get_artifact_path};
use foundry_config::fs_permissions::FsAccessKind;
use hex::FromHex;
use jsonpath_lib;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap},
    env,
    io::{BufRead, BufReader, Write},
    path::Path,
    process::Command,
    str::FromStr,
    time::UNIX_EPOCH,
};
use tracing::{error, trace};
/// Invokes a `Command` with the given args and returns the abi encoded response
///
/// If the output of the command is valid hex, it returns the hex decoded value
fn ffi(state: &Cheatcodes, args: &[String]) -> Result<Bytes, Bytes> {
    if args.is_empty() || args[0].is_empty() {
        return Err(error::encode_error("Can't execute empty command"))
    }
    let mut cmd = Command::new(&args[0]);
    if args.len() > 1 {
        cmd.args(&args[1..]);
    }

    trace!(?args, "invoking ffi");

    let output = cmd
        .current_dir(&state.config.root)
        .output()
        .map_err(|err| error::encode_error(format!("Failed to execute command: {err}")))?;

    if !output.stderr.is_empty() {
        let err = String::from_utf8_lossy(&output.stderr);
        error!(?err, "stderr");
    }

    let output = String::from_utf8(output.stdout)
        .map_err(|err| error::encode_error(format!("Failed to decode non utf-8 output: {err}")))?;

    let trim_out = output.trim();
    if let Ok(hex_decoded) = hex::decode(trim_out.strip_prefix("0x").unwrap_or(trim_out)) {
        return Ok(abi::encode(&[Token::Bytes(hex_decoded.to_vec())]).into())
    }

    Ok(trim_out.to_string().encode().into())
}

/// An enum which unifies the deserialization of Hardhat-style artifacts with Forge-style artifacts
/// to get their bytecode.
#[derive(Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
enum ArtifactBytecode {
    Hardhat(HardhatArtifact),
    Solc(JsonAbi),
    Forge(CompactContractBytecode),
}

impl ArtifactBytecode {
    fn into_bytecode(self) -> Option<ethers::types::Bytes> {
        match self {
            ArtifactBytecode::Hardhat(inner) => Some(inner.bytecode),
            ArtifactBytecode::Forge(inner) => {
                inner.bytecode.and_then(|bytecode| bytecode.object.into_bytes())
            }
            ArtifactBytecode::Solc(inner) => inner.bytecode(),
        }
    }

    fn into_deployed_bytecode(self) -> Option<ethers::types::Bytes> {
        match self {
            ArtifactBytecode::Hardhat(inner) => Some(inner.deployed_bytecode),
            ArtifactBytecode::Forge(inner) => inner.deployed_bytecode.and_then(|bytecode| {
                bytecode.bytecode.and_then(|bytecode| bytecode.object.into_bytes())
            }),
            ArtifactBytecode::Solc(inner) => inner.deployed_bytecode(),
        }
    }
}

/// A thin wrapper around a Hardhat-style artifact that only extracts the bytecode.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HardhatArtifact {
    #[serde(deserialize_with = "ethers::solc::artifacts::deserialize_bytes")]
    bytecode: ethers::types::Bytes,
    #[serde(deserialize_with = "ethers::solc::artifacts::deserialize_bytes")]
    deployed_bytecode: ethers::types::Bytes,
}

/// Returns the _deployed_ bytecode (`bytecode`) of the matching artifact
fn get_code(state: &Cheatcodes, path: &str) -> Result<Bytes, Bytes> {
    let bytecode = read_bytecode(state, path)?;
    if let Some(bin) = bytecode.into_bytecode() {
        Ok(abi::encode(&[Token::Bytes(bin.to_vec())]).into())
    } else {
        Err("No bytecode for contract. Is it abstract or unlinked?".to_string().encode().into())
    }
}

/// Returns the _deployed_ bytecode (`bytecode`) of the matching artifact
fn get_deployed_code(state: &Cheatcodes, path: &str) -> Result<Bytes, Bytes> {
    let bytecode = read_bytecode(state, path)?;
    if let Some(bin) = bytecode.into_deployed_bytecode() {
        Ok(abi::encode(&[Token::Bytes(bin.to_vec())]).into())
    } else {
        Err("No bytecode for contract. Is it abstract or unlinked?".to_string().encode().into())
    }
}

/// Reads the bytecode object(s) from the matching artifact
fn read_bytecode(state: &Cheatcodes, path: &str) -> Result<ArtifactBytecode, Bytes> {
    let path = get_artifact_path(&state.config.paths, path);
    let path =
        state.config.ensure_path_allowed(path, FsAccessKind::Read).map_err(error::encode_error)?;

    let data = fs::read_to_string(path).map_err(error::encode_error)?;
    serde_json::from_str::<ArtifactBytecode>(&data).map_err(error::encode_error)
}

fn set_env(key: &str, val: &str) -> Result<Bytes, Bytes> {
    // `std::env::set_var` may panic in the following situations
    // ref: https://doc.rust-lang.org/std/env/fn.set_var.html
    if key.is_empty() {
        Err("Environment variable key can't be empty".to_string().encode().into())
    } else if key.contains('=') {
        Err("Environment variable key can't contain equal sign `=`".to_string().encode().into())
    } else if key.contains('\0') {
        Err("Environment variable key can't contain NUL character `\\0`"
            .to_string()
            .encode()
            .into())
    } else if val.contains('\0') {
        Err("Environment variable value can't contain NUL character `\\0`"
            .to_string()
            .encode()
            .into())
    } else {
        env::set_var(key, val);
        Ok(Bytes::new())
    }
}

fn get_env(
    key: &str,
    r#type: ParamType,
    delim: Option<&str>,
    default: Option<String>,
) -> Result<Bytes, Bytes> {
    let msg = format!("Failed to get environment variable `{key}` as type `{}`", &r#type);
    let val = if let Some(value) = default {
        env::var(key).unwrap_or(value)
    } else {
        env::var(key).map_err::<Bytes, _>(|e| format!("{msg}: {e}").encode().into())?
    };
    let val = if let Some(d) = delim {
        val.split(d).map(|v| v.trim().to_string()).collect()
    } else {
        vec![val]
    };
    let is_array: bool = delim.is_some();
    util::value_to_abi(val, r#type, is_array).map_err(|e| format!("{msg}: {e}").encode().into())
}

fn project_root(state: &Cheatcodes) -> Result<Bytes, Bytes> {
    let root = state.config.root.display().to_string();

    Ok(abi::encode(&[Token::String(root)]).into())
}

fn read_file(state: &Cheatcodes, path: impl AsRef<Path>) -> Result<Bytes, Bytes> {
    let path =
        state.config.ensure_path_allowed(&path, FsAccessKind::Read).map_err(error::encode_error)?;

    let data = fs::read_to_string(path).map_err(error::encode_error)?;

    Ok(abi::encode(&[Token::String(data)]).into())
}

fn read_file_binary(state: &Cheatcodes, path: impl AsRef<Path>) -> Result<Bytes, Bytes> {
    let path =
        state.config.ensure_path_allowed(&path, FsAccessKind::Read).map_err(error::encode_error)?;

    let data = fs::read(path).map_err(error::encode_error)?;

    Ok(abi::encode(&[Token::Bytes(data)]).into())
}

fn read_line(state: &mut Cheatcodes, path: impl AsRef<Path>) -> Result<Bytes, Bytes> {
    let path =
        state.config.ensure_path_allowed(&path, FsAccessKind::Read).map_err(error::encode_error)?;

    // Get reader for previously opened file to continue reading OR initialize new reader
    let reader = state
        .context
        .opened_read_files
        .entry(path.clone())
        .or_insert(BufReader::new(fs::open(path).map_err(error::encode_error)?));

    let mut line: String = String::new();
    reader.read_line(&mut line).map_err(error::encode_error)?;

    // Remove trailing newline character, preserving others for cases where it may be important
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }

    Ok(abi::encode(&[Token::String(line)]).into())
}

/// Writes the content to the file
///
/// This function will create a file if it does not exist, and will entirely replace its contents if
/// it does.
///
/// Caution: writing files is only allowed if the targeted path is allowed, (inside `<root>/` by
/// default)
fn write_file(
    state: &Cheatcodes,
    path: impl AsRef<Path>,
    content: impl AsRef<[u8]>,
) -> Result<Bytes, Bytes> {
    let path = state
        .config
        .ensure_path_allowed(&path, FsAccessKind::Write)
        .map_err(error::encode_error)?;
    // write access to foundry.toml is not allowed
    state.config.ensure_not_foundry_toml(&path).map_err(error::encode_error)?;

    if state.fs_commit {
        fs::write(path, content.as_ref()).map_err(error::encode_error)?;
    }

    Ok(Bytes::new())
}

/// Writes a single line to the file
///
/// This will create a file if it does not exist but append the `line` if it does
fn write_line(state: &Cheatcodes, path: impl AsRef<Path>, line: &str) -> Result<Bytes, Bytes> {
    let path = state
        .config
        .ensure_path_allowed(&path, FsAccessKind::Write)
        .map_err(error::encode_error)?;
    state.config.ensure_not_foundry_toml(&path).map_err(error::encode_error)?;

    if state.fs_commit {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .map_err(error::encode_error)?;

        writeln!(file, "{line}").map_err(error::encode_error)?;
    }

    Ok(Bytes::new())
}

fn close_file(state: &mut Cheatcodes, path: impl AsRef<Path>) -> Result<Bytes, Bytes> {
    let path =
        state.config.ensure_path_allowed(&path, FsAccessKind::Read).map_err(error::encode_error)?;

    state.context.opened_read_files.remove(&path);

    Ok(Bytes::new())
}

/// Removes a file from the filesystem.
///
/// Only files inside `<root>/` can be removed, `foundry.toml` excluded.
///
/// This will return an error if the path points to a directory, or the file does not exist
fn remove_file(state: &mut Cheatcodes, path: impl AsRef<Path>) -> Result<Bytes, Bytes> {
    let path = state
        .config
        .ensure_path_allowed(&path, FsAccessKind::Write)
        .map_err(error::encode_error)?;
    state.config.ensure_not_foundry_toml(&path).map_err(error::encode_error)?;

    // also remove from the set if opened previously
    state.context.opened_read_files.remove(&path);

    if state.fs_commit {
        fs::remove_file(&path).map_err(error::encode_error)?;
    }

    Ok(Bytes::new())
}

/// Gets the metadata of a file/directory
///
/// This will return an error if no file/directory is found, or if the target path isn't allowed
fn fs_metadata(state: &mut Cheatcodes, path: impl AsRef<Path>) -> Result<Bytes, Bytes> {
    let path =
        state.config.ensure_path_allowed(&path, FsAccessKind::Read).map_err(error::encode_error)?;

    let metadata = path.metadata().map_err(error::encode_error)?;

    // These fields not available on all platforms; default to 0
    let [modified, accessed, created] =
        [metadata.modified(), metadata.accessed(), metadata.created()].map(|time| {
            time.unwrap_or(UNIX_EPOCH).duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
        });

    let metadata = (
        metadata.is_dir(),
        metadata.is_symlink(),
        metadata.len(),
        metadata.permissions().readonly(),
        modified,
        accessed,
        created,
    );
    Ok(metadata.encode().into())
}

/// Converts a serde_json::Value to an abi::Token
/// The function is designed to run recursively, so that in case of an object
/// it will call itself to convert each of it's value and encode the whole as a
/// Tuple
fn value_to_token(value: &Value) -> eyre::Result<Token> {
    if let Some(boolean) = value.as_bool() {
        Ok(Token::Bool(boolean))
    } else if let Some(string) = value.as_str() {
        if let Some(val) = string.strip_prefix("0x") {
            // If it can decoded as an address, it's an address
            if let Ok(addr) = H160::from_str(string) {
                Ok(Token::Address(addr))
            } else if hex::decode(val).is_ok() {
                // if length == 32 bytes, then encode as Bytes32, else Bytes
                Ok(if val.len() == 64 {
                    Token::FixedBytes(Vec::from_hex(val).unwrap())
                } else {
                    Token::Bytes(Vec::from_hex(val).unwrap())
                })
            } else {
                // If incorrect length, pad 0 at the beginning
                let arr = format!("0{val}");
                Ok(Token::Bytes(Vec::from_hex(arr).unwrap()))
            }
        } else {
            Ok(Token::String(string.to_owned()))
        }
    } else if let Some(number) = value.as_u64() {
        Ok(Token::Uint(number.into()))
    } else if let Some(number) = value.as_i64() {
        Ok(Token::Int(number.into()))
    } else if let Some(array) = value.as_array() {
        Ok(Token::Array(array.iter().map(value_to_token).collect::<eyre::Result<Vec<_>>>()?))
    } else if value.as_object().is_some() {
        let ordered_object: BTreeMap<String, Value> =
            serde_json::from_value(value.clone()).unwrap();
        let values =
            ordered_object.values().map(value_to_token).collect::<eyre::Result<Vec<_>>>()?;
        Ok(Token::Tuple(values))
    } else if value.is_null() {
        Ok(Token::FixedBytes(vec![0; 32]))
    } else {
        eyre::bail!("Unexpected json value: {}", value)
    }
}
/// Parses a JSON and returns a single value, an array or an entire JSON object encoded as tuple.
/// As the JSON object is parsed serially, with the keys ordered alphabetically, they must be
/// deserialized in the same order. That means that the solidity `struct` should order it's fields
/// alphabetically and not by efficient packing or some other taxonomy.
fn parse_json(_state: &mut Cheatcodes, json_str: &str, key: &str) -> Result<Bytes, Bytes> {
    let json = serde_json::from_str(json_str).map_err(error::encode_error)?;
    let values: Vec<&Value> = jsonpath_lib::select(&json, key).map_err(error::encode_error)?;
    // values is an array of items. Depending on the JsonPath key, they
    // can be many or a single item. An item can be a single value or
    // an entire JSON object.
    let res = values
        .iter()
        .map(|inner| {
            value_to_token(inner).map_err(|err| {
                error::encode_error(err.wrap_err(format!("Failed to parse key {key}")))
            })
        })
        .collect::<Result<Vec<Token>, Bytes>>();
    // encode the bytes as the 'bytes' solidity type
    let abi_encoded = abi::encode(&[Token::Bytes(abi::encode(&res?))]);
    Ok(abi_encoded.into())
}
/// Serializes a key:value pair to a specific object. By calling this function multiple times,
/// the user can serialize multiple KV pairs to the same object. The value can be of any type, even
/// a new object in itself. The function will return
/// a stringified version of the object, so that the user can use that as a value to a new
/// invocation of the same function with a new object key. This enables the user to reuse the same
/// function to crate arbitrarily complex object structures (JSON).
fn serialize_json(
    state: &mut Cheatcodes,
    object_key: &str,
    value_key: &str,
    value: &str,
) -> Result<Bytes, Bytes> {
    let parsed_value =
        serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_string()));
    let json = if let Some(serialization) = state.serialized_jsons.get_mut(object_key) {
        serialization.insert(value_key.to_string(), parsed_value);
        serialization.clone()
    } else {
        let mut serialization = HashMap::new();
        serialization.insert(value_key.to_string(), parsed_value);
        state.serialized_jsons.insert(object_key.to_string(), serialization.clone());
        serialization.clone()
    };
    let stringified = serde_json::to_string(&json)
        .map_err(|err| error::encode_error(format!("Failed to stringify hashmap: {err}")))?;
    Ok(abi::encode(&[Token::String(stringified)]).into())
}
/// Converts an array to it's stringified version, adding the appropriate quotes around it's
/// ellements. This is to signify that the elements of the array are string themselves.
fn array_str_to_str<T: UIfmt>(array: &Vec<T>) -> String {
    format!(
        "[{}",
        array
            .iter()
            .enumerate()
            .map(|(index, value)| {
                if index == array.len() - 1 {
                    format!("\"{}\"]", value.pretty())
                } else {
                    format!("\"{}\",", value.pretty())
                }
            })
            .collect::<String>()
    )
}

/// Converts an array to it's stringified version. It will not add quotes around the values of the
/// array, enabling serde_json to parse the values of the array as types (e.g numbers, booleans,
/// etc.)
fn array_eval_to_str<T: UIfmt>(array: &Vec<T>) -> String {
    format!(
        "[{}",
        array
            .iter()
            .enumerate()
            .map(|(index, value)| {
                if index == array.len() - 1 {
                    format!("{}]", value.pretty())
                } else {
                    format!("{},", value.pretty())
                }
            })
            .collect::<String>()
    )
}

/// Write an object to a new file OR replaces the value of an existing JSON file with the supplied
/// object.
fn write_json(
    _state: &mut Cheatcodes,
    object: &str,
    path: impl AsRef<Path>,
    json_path_or_none: Option<&str>,
) -> Result<Bytes, Bytes> {
    let json: Value =
        serde_json::from_str(object).unwrap_or_else(|_| Value::String(object.to_owned()));
    let json_string = serde_json::to_string_pretty(&if let Some(json_path) = json_path_or_none {
        let path = _state
            .config
            .ensure_path_allowed(&path, FsAccessKind::Read)
            .map_err(error::encode_error)?;
        let data = serde_json::from_str(&fs::read_to_string(path).map_err(error::encode_error)?)
            .map_err(error::encode_error)?;
        jsonpath_lib::replace_with(data, &format!("${json_path}"), &mut |_| Some(json.clone()))
            .map_err(error::encode_error)?
    } else {
        json
    })
    .map_err(error::encode_error)?;
    write_file(_state, path, json_string)?;
    Ok(Bytes::new())
}

pub fn apply(
    state: &mut Cheatcodes,
    ffi_enabled: bool,
    call: &HEVMCalls,
) -> Option<Result<Bytes, Bytes>> {
    Some(match call {
        HEVMCalls::Ffi(inner) => {
            if !ffi_enabled {
                Err("FFI disabled: run again with `--ffi` if you want to allow tests to call external scripts.".to_string().encode().into())
            } else {
                ffi(state, &inner.0)
            }
        }
        HEVMCalls::GetCode(inner) => get_code(state, &inner.0),
        HEVMCalls::GetDeployedCode(inner) => get_deployed_code(state, &inner.0),
        HEVMCalls::SetEnv(inner) => set_env(&inner.0, &inner.1),
        HEVMCalls::EnvBool0(inner) => get_env(&inner.0, ParamType::Bool, None, None),
        HEVMCalls::EnvUint0(inner) => get_env(&inner.0, ParamType::Uint(256), None, None),
        HEVMCalls::EnvInt0(inner) => get_env(&inner.0, ParamType::Int(256), None, None),
        HEVMCalls::EnvAddress0(inner) => get_env(&inner.0, ParamType::Address, None, None),
        HEVMCalls::EnvBytes320(inner) => get_env(&inner.0, ParamType::FixedBytes(32), None, None),
        HEVMCalls::EnvString0(inner) => get_env(&inner.0, ParamType::String, None, None),
        HEVMCalls::EnvBytes0(inner) => get_env(&inner.0, ParamType::Bytes, None, None),
        HEVMCalls::EnvBool1(inner) => get_env(&inner.0, ParamType::Bool, Some(&inner.1), None),
        HEVMCalls::EnvUint1(inner) => get_env(&inner.0, ParamType::Uint(256), Some(&inner.1), None),
        HEVMCalls::EnvInt1(inner) => get_env(&inner.0, ParamType::Int(256), Some(&inner.1), None),
        HEVMCalls::EnvAddress1(inner) => {
            get_env(&inner.0, ParamType::Address, Some(&inner.1), None)
        }
        HEVMCalls::EnvBytes321(inner) => {
            get_env(&inner.0, ParamType::FixedBytes(32), Some(&inner.1), None)
        }
        HEVMCalls::EnvString1(inner) => get_env(&inner.0, ParamType::String, Some(&inner.1), None),
        HEVMCalls::EnvBytes1(inner) => get_env(&inner.0, ParamType::Bytes, Some(&inner.1), None),
        HEVMCalls::EnvOr0(inner) => {
            get_env(&inner.0, ParamType::Bool, None, Some(inner.1.to_string()))
        }
        HEVMCalls::EnvOr1(inner) => {
            get_env(&inner.0, ParamType::Uint(256), None, Some(inner.1.to_string()))
        }
        HEVMCalls::EnvOr2(inner) => {
            get_env(&inner.0, ParamType::Int(256), None, Some(inner.1.to_string()))
        }
        HEVMCalls::EnvOr3(inner) => {
            get_env(&inner.0, ParamType::Address, None, Some(hex::encode(inner.1)))
        }
        HEVMCalls::EnvOr4(inner) => {
            get_env(&inner.0, ParamType::FixedBytes(32), None, Some(hex::encode(inner.1)))
        }
        HEVMCalls::EnvOr5(inner) => {
            get_env(&inner.0, ParamType::String, None, Some(inner.1.to_string()))
        }
        HEVMCalls::EnvOr6(inner) => {
            get_env(&inner.0, ParamType::Bytes, None, Some(hex::encode(&inner.1)))
        }
        HEVMCalls::EnvOr7(inner) => get_env(
            &inner.0,
            ParamType::Bool,
            Some(&inner.1),
            Some(inner.2.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(&inner.1)),
        ),
        HEVMCalls::EnvOr8(inner) => get_env(
            &inner.0,
            ParamType::Uint(256),
            Some(&inner.1),
            Some(inner.2.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(&inner.1)),
        ),
        HEVMCalls::EnvOr9(inner) => get_env(
            &inner.0,
            ParamType::Int(256),
            Some(&inner.1),
            Some(inner.2.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(&inner.1)),
        ),
        HEVMCalls::EnvOr10(inner) => get_env(
            &inner.0,
            ParamType::Address,
            Some(&inner.1),
            Some(inner.2.iter().map(hex::encode).collect::<Vec<_>>().join(&inner.1)),
        ),
        HEVMCalls::EnvOr11(inner) => get_env(
            &inner.0,
            ParamType::FixedBytes(32),
            Some(&inner.1),
            Some(inner.2.iter().map(hex::encode).collect::<Vec<_>>().join(&inner.1)),
        ),
        HEVMCalls::EnvOr12(inner) => {
            get_env(&inner.0, ParamType::String, Some(&inner.1), Some(inner.2.join(&inner.1)))
        }
        HEVMCalls::EnvOr13(inner) => get_env(
            &inner.0,
            ParamType::Bytes,
            Some(&inner.1),
            Some(inner.2.iter().map(hex::encode).collect::<Vec<_>>().join(&inner.1)),
        ),

        HEVMCalls::ProjectRoot(_) => project_root(state),
        HEVMCalls::ReadFile(inner) => read_file(state, &inner.0),
        HEVMCalls::ReadFileBinary(inner) => read_file_binary(state, &inner.0),
        HEVMCalls::ReadLine(inner) => read_line(state, &inner.0),
        HEVMCalls::WriteFile(inner) => write_file(state, &inner.0, &inner.1),
        HEVMCalls::WriteFileBinary(inner) => write_file(state, &inner.0, &inner.1),
        HEVMCalls::WriteLine(inner) => write_line(state, &inner.0, &inner.1),
        HEVMCalls::CloseFile(inner) => close_file(state, &inner.0),
        HEVMCalls::RemoveFile(inner) => remove_file(state, &inner.0),
        HEVMCalls::FsMetadata(inner) => fs_metadata(state, &inner.0),
        // If no key argument is passed, return the whole JSON object.
        // "$" is the JSONPath key for the root of the object
        HEVMCalls::ParseJson0(inner) => parse_json(state, &inner.0, "$"),
        HEVMCalls::ParseJson1(inner) => parse_json(state, &inner.0, &format!("$.{}", &inner.1)),
        HEVMCalls::SerializeBool0(inner) => {
            serialize_json(state, &inner.0, &inner.1, &inner.2.pretty())
        }
        HEVMCalls::SerializeBool1(inner) => {
            serialize_json(state, &inner.0, &inner.1, &array_eval_to_str(&inner.2))
        }
        HEVMCalls::SerializeUint0(inner) => {
            serialize_json(state, &inner.0, &inner.1, &inner.2.pretty())
        }
        HEVMCalls::SerializeUint1(inner) => {
            serialize_json(state, &inner.0, &inner.1, &array_eval_to_str(&inner.2))
        }
        HEVMCalls::SerializeInt0(inner) => {
            serialize_json(state, &inner.0, &inner.1, &inner.2.pretty())
        }
        HEVMCalls::SerializeInt1(inner) => {
            serialize_json(state, &inner.0, &inner.1, &array_eval_to_str(&inner.2))
        }
        HEVMCalls::SerializeAddress0(inner) => {
            serialize_json(state, &inner.0, &inner.1, &inner.2.pretty())
        }
        HEVMCalls::SerializeAddress1(inner) => {
            serialize_json(state, &inner.0, &inner.1, &array_str_to_str(&inner.2))
        }
        HEVMCalls::SerializeBytes320(inner) => {
            serialize_json(state, &inner.0, &inner.1, &inner.2.pretty())
        }
        HEVMCalls::SerializeBytes321(inner) => {
            serialize_json(state, &inner.0, &inner.1, &array_str_to_str(&inner.2))
        }
        HEVMCalls::SerializeString0(inner) => {
            serialize_json(state, &inner.0, &inner.1, &inner.2.pretty())
        }
        HEVMCalls::SerializeString1(inner) => {
            serialize_json(state, &inner.0, &inner.1, &array_str_to_str(&inner.2))
        }
        HEVMCalls::SerializeBytes0(inner) => {
            serialize_json(state, &inner.0, &inner.1, &inner.2.pretty())
        }
        HEVMCalls::SerializeBytes1(inner) => {
            serialize_json(state, &inner.0, &inner.1, &array_str_to_str(&inner.2))
        }
        HEVMCalls::WriteJson0(inner) => write_json(state, &inner.0, &inner.1, None),
        HEVMCalls::WriteJson1(inner) => write_json(state, &inner.0, &inner.1, Some(&inner.2)),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::inspector::CheatsConfig;
    use ethers::core::abi::AbiDecode;
    use std::{path::PathBuf, sync::Arc};

    fn cheats() -> Cheatcodes {
        let config =
            CheatsConfig { root: PathBuf::from(&env!("CARGO_MANIFEST_DIR")), ..Default::default() };
        Cheatcodes { config: Arc::new(config), ..Default::default() }
    }

    #[test]
    fn test_ffi_hex() {
        let msg = "gm";
        let cheats = cheats();
        let args = ["echo".to_string(), hex::encode(msg)];
        let output = ffi(&cheats, &args).unwrap();

        let output = String::decode(&output).unwrap();
        assert_eq!(output, msg);
    }

    #[test]
    fn test_ffi_string() {
        let msg = "gm";
        let cheats = cheats();

        let args = ["echo".to_string(), msg.to_string()];
        let output = ffi(&cheats, &args).unwrap();

        let output = String::decode(&output).unwrap();
        assert_eq!(output, msg);
    }

    #[test]
    fn test_artifact_parsing() {
        let s = r#"{
  "abi": [
    {
      "inputs": [
        {
          "internalType": "address",
          "name": "_feeToSetter",
          "type": "address"
        }
      ],
      "payable": false,
      "stateMutability": "nonpayable",
      "type": "constructor"
    },
    {
      "anonymous": false,
      "inputs": [
        {
          "indexed": true,
          "internalType": "address",
          "name": "token0",
          "type": "address"
        },
        {
          "indexed": true,
          "internalType": "address",
          "name": "token1",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "address",
          "name": "pair",
          "type": "address"
        },
        {
          "indexed": false,
          "internalType": "uint256",
          "name": "",
          "type": "uint256"
        }
      ],
      "name": "PairCreated",
      "type": "event"
    },
    {
      "constant": true,
      "inputs": [
        {
          "internalType": "uint256",
          "name": "",
          "type": "uint256"
        }
      ],
      "name": "allPairs",
      "outputs": [
        {
          "internalType": "address",
          "name": "",
          "type": "address"
        }
      ],
      "payable": false,
      "stateMutability": "view",
      "type": "function"
    },
    {
      "constant": true,
      "inputs": [],
      "name": "allPairsLength",
      "outputs": [
        {
          "internalType": "uint256",
          "name": "",
          "type": "uint256"
        }
      ],
      "payable": false,
      "stateMutability": "view",
      "type": "function"
    },
    {
      "constant": false,
      "inputs": [
        {
          "internalType": "address",
          "name": "tokenA",
          "type": "address"
        },
        {
          "internalType": "address",
          "name": "tokenB",
          "type": "address"
        }
      ],
      "name": "createPair",
      "outputs": [
        {
          "internalType": "address",
          "name": "pair",
          "type": "address"
        }
      ],
      "payable": false,
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "constant": true,
      "inputs": [],
      "name": "feeTo",
      "outputs": [
        {
          "internalType": "address",
          "name": "",
          "type": "address"
        }
      ],
      "payable": false,
      "stateMutability": "view",
      "type": "function"
    },
    {
      "constant": true,
      "inputs": [],
      "name": "feeToSetter",
      "outputs": [
        {
          "internalType": "address",
          "name": "",
          "type": "address"
        }
      ],
      "payable": false,
      "stateMutability": "view",
      "type": "function"
    },
    {
      "constant": true,
      "inputs": [
        {
          "internalType": "address",
          "name": "",
          "type": "address"
        },
        {
          "internalType": "address",
          "name": "",
          "type": "address"
        }
      ],
      "name": "getPair",
      "outputs": [
        {
          "internalType": "address",
          "name": "",
          "type": "address"
        }
      ],
      "payable": false,
      "stateMutability": "view",
      "type": "function"
    },
    {
      "constant": false,
      "inputs": [
        {
          "internalType": "address",
          "name": "_feeTo",
          "type": "address"
        }
      ],
      "name": "setFeeTo",
      "outputs": [],
      "payable": false,
      "stateMutability": "nonpayable",
      "type": "function"
    },
    {
      "constant": false,
      "inputs": [
        {
          "internalType": "address",
          "name": "_feeToSetter",
          "type": "address"
        }
      ],
      "name": "setFeeToSetter",
      "outputs": [],
      "payable": false,
      "stateMutability": "nonpayable",
      "type": "function"
    }
  ],
  "evm": {
    "bytecode": {
      "linkReferences": {},
      "object": "608060405234801561001057600080fd5b506040516136863803806136868339818101604052602081101561003357600080fd5b5051600180546001600160a01b0319166001600160a01b03909216919091179055613623806100636000396000f3fe608060405234801561001057600080fd5b50600436106100885760003560e01c8063a2e74af61161005b578063a2e74af6146100fd578063c9c6539614610132578063e6a439051461016d578063f46901ed146101a857610088565b8063017e7e581461008d578063094b7415146100be5780631e3dd18b146100c6578063574f2ba3146100e3575b600080fd5b6100956101db565b6040805173ffffffffffffffffffffffffffffffffffffffff9092168252519081900360200190f35b6100956101f7565b610095600480360360208110156100dc57600080fd5b5035610213565b6100eb610247565b60408051918252519081900360200190f35b6101306004803603602081101561011357600080fd5b503573ffffffffffffffffffffffffffffffffffffffff1661024d565b005b6100956004803603604081101561014857600080fd5b5073ffffffffffffffffffffffffffffffffffffffff8135811691602001351661031a565b6100956004803603604081101561018357600080fd5b5073ffffffffffffffffffffffffffffffffffffffff8135811691602001351661076d565b610130600480360360208110156101be57600080fd5b503573ffffffffffffffffffffffffffffffffffffffff166107a0565b60005473ffffffffffffffffffffffffffffffffffffffff1681565b60015473ffffffffffffffffffffffffffffffffffffffff1681565b6003818154811061022057fe5b60009182526020909120015473ffffffffffffffffffffffffffffffffffffffff16905081565b60035490565b60015473ffffffffffffffffffffffffffffffffffffffff1633146102d357604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601460248201527f556e697377617056323a20464f5242494444454e000000000000000000000000604482015290519081900360640190fd5b600180547fffffffffffffffffffffffff00000000000000000000000000000000000000001673ffffffffffffffffffffffffffffffffffffffff92909216919091179055565b60008173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1614156103b757604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601e60248201527f556e697377617056323a204944454e544943414c5f4144445245535345530000604482015290519081900360640190fd5b6000808373ffffffffffffffffffffffffffffffffffffffff168573ffffffffffffffffffffffffffffffffffffffff16106103f45783856103f7565b84845b909250905073ffffffffffffffffffffffffffffffffffffffff821661047e57604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f556e697377617056323a205a45524f5f41444452455353000000000000000000604482015290519081900360640190fd5b73ffffffffffffffffffffffffffffffffffffffff82811660009081526002602090815260408083208585168452909152902054161561051f57604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601660248201527f556e697377617056323a20504149525f45584953545300000000000000000000604482015290519081900360640190fd5b6060604051806020016105319061086d565b6020820181038252601f19601f82011660405250905060008383604051602001808373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1660601b81526014018273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1660601b815260140192505050604051602081830303815290604052805190602001209050808251602084016000f5604080517f485cc95500000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff8781166004830152868116602483015291519297509087169163485cc9559160448082019260009290919082900301818387803b15801561065e57600080fd5b505af1158015610672573d6000803e3d6000fd5b5050505073ffffffffffffffffffffffffffffffffffffffff84811660008181526002602081815260408084208987168086529083528185208054978d167fffffffffffffffffffffffff000000000000000000000000000000000000000098891681179091559383528185208686528352818520805488168517905560038054600181018255958190527fc2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b90950180549097168417909655925483519283529082015281517f0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9929181900390910190a35050505092915050565b600260209081526000928352604080842090915290825290205473ffffffffffffffffffffffffffffffffffffffff1681565b60015473ffffffffffffffffffffffffffffffffffffffff16331461082657604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601460248201527f556e697377617056323a20464f5242494444454e000000000000000000000000604482015290519081900360640190fd5b600080547fffffffffffffffffffffffff00000000000000000000000000000000000000001673ffffffffffffffffffffffffffffffffffffffff92909216919091179055565b612d748061087b8339019056fe60806040526001600c5534801561001557600080fd5b506040514690806052612d228239604080519182900360520182208282018252600a8352692ab734b9bbb0b8102b1960b11b6020938401528151808301835260018152603160f81b908401528151808401919091527fbfcc8ef98ffbf7b6c3fec7bf5185b566b9863e35a9d83acd49ad6824b5969738818301527fc89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6606082015260808101949094523060a0808601919091528151808603909101815260c09094019052825192019190912060035550600580546001600160a01b03191633179055612c1d806101056000396000f3fe608060405234801561001057600080fd5b50600436106101b95760003560e01c80636a627842116100f9578063ba9a7a5611610097578063d21220a711610071578063d21220a7146105da578063d505accf146105e2578063dd62ed3e14610640578063fff6cae91461067b576101b9565b8063ba9a7a5614610597578063bc25cf771461059f578063c45a0155146105d2576101b9565b80637ecebe00116100d35780637ecebe00146104d757806389afcb441461050a57806395d89b4114610556578063a9059cbb1461055e576101b9565b80636a6278421461046957806370a082311461049c5780637464fc3d146104cf576101b9565b806323b872dd116101665780633644e515116101405780633644e51514610416578063485cc9551461041e5780635909c0d5146104595780635a3d549314610461576101b9565b806323b872dd146103ad57806330adf81f146103f0578063313ce567146103f8576101b9565b8063095ea7b311610197578063095ea7b3146103155780630dfe16811461036257806318160ddd14610393576101b9565b8063022c0d9f146101be57806306fdde03146102595780630902f1ac146102d6575b600080fd5b610257600480360360808110156101d457600080fd5b81359160208101359173ffffffffffffffffffffffffffffffffffffffff604083013516919081019060808101606082013564010000000081111561021857600080fd5b82018360208201111561022a57600080fd5b8035906020019184600183028401116401000000008311171561024c57600080fd5b509092509050610683565b005b610261610d57565b6040805160208082528351818301528351919283929083019185019080838360005b8381101561029b578181015183820152602001610283565b50505050905090810190601f1680156102c85780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b6102de610d90565b604080516dffffffffffffffffffffffffffff948516815292909316602083015263ffffffff168183015290519081900360600190f35b61034e6004803603604081101561032b57600080fd5b5073ffffffffffffffffffffffffffffffffffffffff8135169060200135610de5565b604080519115158252519081900360200190f35b61036a610dfc565b6040805173ffffffffffffffffffffffffffffffffffffffff9092168252519081900360200190f35b61039b610e18565b60408051918252519081900360200190f35b61034e600480360360608110156103c357600080fd5b5073ffffffffffffffffffffffffffffffffffffffff813581169160208101359091169060400135610e1e565b61039b610efd565b610400610f21565b6040805160ff9092168252519081900360200190f35b61039b610f26565b6102576004803603604081101561043457600080fd5b5073ffffffffffffffffffffffffffffffffffffffff81358116916020013516610f2c565b61039b611005565b61039b61100b565b61039b6004803603602081101561047f57600080fd5b503573ffffffffffffffffffffffffffffffffffffffff16611011565b61039b600480360360208110156104b257600080fd5b503573ffffffffffffffffffffffffffffffffffffffff166113cb565b61039b6113dd565b61039b600480360360208110156104ed57600080fd5b503573ffffffffffffffffffffffffffffffffffffffff166113e3565b61053d6004803603602081101561052057600080fd5b503573ffffffffffffffffffffffffffffffffffffffff166113f5565b6040805192835260208301919091528051918290030190f35b610261611892565b61034e6004803603604081101561057457600080fd5b5073ffffffffffffffffffffffffffffffffffffffff81351690602001356118cb565b61039b6118d8565b610257600480360360208110156105b557600080fd5b503573ffffffffffffffffffffffffffffffffffffffff166118de565b61036a611ad4565b61036a611af0565b610257600480360360e08110156105f857600080fd5b5073ffffffffffffffffffffffffffffffffffffffff813581169160208101359091169060408101359060608101359060ff6080820135169060a08101359060c00135611b0c565b61039b6004803603604081101561065657600080fd5b5073ffffffffffffffffffffffffffffffffffffffff81358116916020013516611dd8565b610257611df5565b600c546001146106f457604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601160248201527f556e697377617056323a204c4f434b4544000000000000000000000000000000604482015290519081900360640190fd5b6000600c55841515806107075750600084115b61075c576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526025815260200180612b2f6025913960400191505060405180910390fd5b600080610767610d90565b5091509150816dffffffffffffffffffffffffffff168710801561079a5750806dffffffffffffffffffffffffffff1686105b6107ef576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526021815260200180612b786021913960400191505060405180910390fd5b600654600754600091829173ffffffffffffffffffffffffffffffffffffffff91821691908116908916821480159061085457508073ffffffffffffffffffffffffffffffffffffffff168973ffffffffffffffffffffffffffffffffffffffff1614155b6108bf57604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601560248201527f556e697377617056323a20494e56414c49445f544f0000000000000000000000604482015290519081900360640190fd5b8a156108d0576108d0828a8d611fdb565b89156108e1576108e1818a8c611fdb565b86156109c3578873ffffffffffffffffffffffffffffffffffffffff166310d1e85c338d8d8c8c6040518663ffffffff1660e01b8152600401808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001858152602001848152602001806020018281038252848482818152602001925080828437600081840152601f19601f8201169050808301925050509650505050505050600060405180830381600087803b1580156109aa57600080fd5b505af11580156109be573d6000803e3d6000fd5b505050505b604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905173ffffffffffffffffffffffffffffffffffffffff8416916370a08231916024808301926020929190829003018186803b158015610a2f57600080fd5b505afa158015610a43573d6000803e3d6000fd5b505050506040513d6020811015610a5957600080fd5b5051604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905191955073ffffffffffffffffffffffffffffffffffffffff8316916370a0823191602480820192602092909190829003018186803b158015610acb57600080fd5b505afa158015610adf573d6000803e3d6000fd5b505050506040513d6020811015610af557600080fd5b5051925060009150506dffffffffffffffffffffffffffff85168a90038311610b1f576000610b35565b89856dffffffffffffffffffffffffffff160383035b9050600089856dffffffffffffffffffffffffffff16038311610b59576000610b6f565b89856dffffffffffffffffffffffffffff160383035b90506000821180610b805750600081115b610bd5576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526024815260200180612b546024913960400191505060405180910390fd5b6000610c09610beb84600363ffffffff6121e816565b610bfd876103e863ffffffff6121e816565b9063ffffffff61226e16565b90506000610c21610beb84600363ffffffff6121e816565b9050610c59620f4240610c4d6dffffffffffffffffffffffffffff8b8116908b1663ffffffff6121e816565b9063ffffffff6121e816565b610c69838363ffffffff6121e816565b1015610cd657604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152600c60248201527f556e697377617056323a204b0000000000000000000000000000000000000000604482015290519081900360640190fd5b5050610ce4848488886122e0565b60408051838152602081018390528082018d9052606081018c9052905173ffffffffffffffffffffffffffffffffffffffff8b169133917fd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d8229181900360800190a350506001600c55505050505050505050565b6040518060400160405280600a81526020017f556e69737761702056320000000000000000000000000000000000000000000081525081565b6008546dffffffffffffffffffffffffffff808216926e0100000000000000000000000000008304909116917c0100000000000000000000000000000000000000000000000000000000900463ffffffff1690565b6000610df233848461259c565b5060015b92915050565b60065473ffffffffffffffffffffffffffffffffffffffff1681565b60005481565b73ffffffffffffffffffffffffffffffffffffffff831660009081526002602090815260408083203384529091528120547fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff14610ee85773ffffffffffffffffffffffffffffffffffffffff84166000908152600260209081526040808320338452909152902054610eb6908363ffffffff61226e16565b73ffffffffffffffffffffffffffffffffffffffff851660009081526002602090815260408083203384529091529020555b610ef384848461260b565b5060019392505050565b7f6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c981565b601281565b60035481565b60055473ffffffffffffffffffffffffffffffffffffffff163314610fb257604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601460248201527f556e697377617056323a20464f5242494444454e000000000000000000000000604482015290519081900360640190fd5b6006805473ffffffffffffffffffffffffffffffffffffffff9384167fffffffffffffffffffffffff00000000000000000000000000000000000000009182161790915560078054929093169116179055565b60095481565b600a5481565b6000600c5460011461108457604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601160248201527f556e697377617056323a204c4f434b4544000000000000000000000000000000604482015290519081900360640190fd5b6000600c81905580611094610d90565b50600654604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905193955091935060009273ffffffffffffffffffffffffffffffffffffffff909116916370a08231916024808301926020929190829003018186803b15801561110e57600080fd5b505afa158015611122573d6000803e3d6000fd5b505050506040513d602081101561113857600080fd5b5051600754604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905192935060009273ffffffffffffffffffffffffffffffffffffffff909216916370a0823191602480820192602092909190829003018186803b1580156111b157600080fd5b505afa1580156111c5573d6000803e3d6000fd5b505050506040513d60208110156111db57600080fd5b505190506000611201836dffffffffffffffffffffffffffff871663ffffffff61226e16565b90506000611225836dffffffffffffffffffffffffffff871663ffffffff61226e16565b9050600061123387876126ec565b600054909150806112705761125c6103e8610bfd611257878763ffffffff6121e816565b612878565b985061126b60006103e86128ca565b6112cd565b6112ca6dffffffffffffffffffffffffffff8916611294868463ffffffff6121e816565b8161129b57fe5b046dffffffffffffffffffffffffffff89166112bd868563ffffffff6121e816565b816112c457fe5b0461297a565b98505b60008911611326576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526028815260200180612bc16028913960400191505060405180910390fd5b6113308a8a6128ca565b61133c86868a8a6122e0565b811561137e5760085461137a906dffffffffffffffffffffffffffff808216916e01000000000000000000000000000090041663ffffffff6121e816565b600b555b6040805185815260208101859052815133927f4c209b5fc8ad50758f13e2e1088ba56a560dff690a1c6fef26394f4c03821c4f928290030190a250506001600c5550949695505050505050565b60016020526000908152604090205481565b600b5481565b60046020526000908152604090205481565b600080600c5460011461146957604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601160248201527f556e697377617056323a204c4f434b4544000000000000000000000000000000604482015290519081900360640190fd5b6000600c81905580611479610d90565b50600654600754604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905194965092945073ffffffffffffffffffffffffffffffffffffffff9182169391169160009184916370a08231916024808301926020929190829003018186803b1580156114fb57600080fd5b505afa15801561150f573d6000803e3d6000fd5b505050506040513d602081101561152557600080fd5b5051604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905191925060009173ffffffffffffffffffffffffffffffffffffffff8516916370a08231916024808301926020929190829003018186803b15801561159957600080fd5b505afa1580156115ad573d6000803e3d6000fd5b505050506040513d60208110156115c357600080fd5b5051306000908152600160205260408120549192506115e288886126ec565b600054909150806115f9848763ffffffff6121e816565b8161160057fe5b049a5080611614848663ffffffff6121e816565b8161161b57fe5b04995060008b11801561162e575060008a115b611683576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526028815260200180612b996028913960400191505060405180910390fd5b61168d3084612992565b611698878d8d611fdb565b6116a3868d8c611fdb565b604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905173ffffffffffffffffffffffffffffffffffffffff8916916370a08231916024808301926020929190829003018186803b15801561170f57600080fd5b505afa158015611723573d6000803e3d6000fd5b505050506040513d602081101561173957600080fd5b5051604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905191965073ffffffffffffffffffffffffffffffffffffffff8816916370a0823191602480820192602092909190829003018186803b1580156117ab57600080fd5b505afa1580156117bf573d6000803e3d6000fd5b505050506040513d60208110156117d557600080fd5b505193506117e585858b8b6122e0565b811561182757600854611823906dffffffffffffffffffffffffffff808216916e01000000000000000000000000000090041663ffffffff6121e816565b600b555b604080518c8152602081018c9052815173ffffffffffffffffffffffffffffffffffffffff8f169233927fdccd412f0b1252819cb1fd330b93224ca42612892bb3f4f789976e6d81936496929081900390910190a35050505050505050506001600c81905550915091565b6040518060400160405280600681526020017f554e492d5632000000000000000000000000000000000000000000000000000081525081565b6000610df233848461260b565b6103e881565b600c5460011461194f57604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601160248201527f556e697377617056323a204c4f434b4544000000000000000000000000000000604482015290519081900360640190fd5b6000600c55600654600754600854604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905173ffffffffffffffffffffffffffffffffffffffff9485169490931692611a2b9285928792611a26926dffffffffffffffffffffffffffff169185916370a0823191602480820192602092909190829003018186803b1580156119ee57600080fd5b505afa158015611a02573d6000803e3d6000fd5b505050506040513d6020811015611a1857600080fd5b50519063ffffffff61226e16565b611fdb565b600854604080517f70a082310000000000000000000000000000000000000000000000000000000081523060048201529051611aca9284928792611a26926e01000000000000000000000000000090046dffffffffffffffffffffffffffff169173ffffffffffffffffffffffffffffffffffffffff8616916370a0823191602480820192602092909190829003018186803b1580156119ee57600080fd5b50506001600c5550565b60055473ffffffffffffffffffffffffffffffffffffffff1681565b60075473ffffffffffffffffffffffffffffffffffffffff1681565b42841015611b7b57604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601260248201527f556e697377617056323a20455850495245440000000000000000000000000000604482015290519081900360640190fd5b60035473ffffffffffffffffffffffffffffffffffffffff80891660008181526004602090815260408083208054600180820190925582517f6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c98186015280840196909652958d166060860152608085018c905260a085019590955260c08085018b90528151808603909101815260e0850182528051908301207f19010000000000000000000000000000000000000000000000000000000000006101008601526101028501969096526101228085019690965280518085039096018652610142840180825286519683019690962095839052610162840180825286905260ff89166101828501526101a284018890526101c28401879052519193926101e2808201937fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe081019281900390910190855afa158015611cdc573d6000803e3d6000fd5b50506040517fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0015191505073ffffffffffffffffffffffffffffffffffffffff811615801590611d5757508873ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16145b611dc257604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601c60248201527f556e697377617056323a20494e56414c49445f5349474e415455524500000000604482015290519081900360640190fd5b611dcd89898961259c565b505050505050505050565b600260209081526000928352604080842090915290825290205481565b600c54600114611e6657604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601160248201527f556e697377617056323a204c4f434b4544000000000000000000000000000000604482015290519081900360640190fd5b6000600c55600654604080517f70a082310000000000000000000000000000000000000000000000000000000081523060048201529051611fd49273ffffffffffffffffffffffffffffffffffffffff16916370a08231916024808301926020929190829003018186803b158015611edd57600080fd5b505afa158015611ef1573d6000803e3d6000fd5b505050506040513d6020811015611f0757600080fd5b5051600754604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905173ffffffffffffffffffffffffffffffffffffffff909216916370a0823191602480820192602092909190829003018186803b158015611f7a57600080fd5b505afa158015611f8e573d6000803e3d6000fd5b505050506040513d6020811015611fa457600080fd5b50516008546dffffffffffffffffffffffffffff808216916e0100000000000000000000000000009004166122e0565b6001600c55565b604080518082018252601981527f7472616e7366657228616464726573732c75696e743235362900000000000000602091820152815173ffffffffffffffffffffffffffffffffffffffff85811660248301526044808301869052845180840390910181526064909201845291810180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff167fa9059cbb000000000000000000000000000000000000000000000000000000001781529251815160009460609489169392918291908083835b602083106120e157805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe090920191602091820191016120a4565b6001836020036101000a0380198251168184511680821785525050505050509050019150506000604051808303816000865af19150503d8060008114612143576040519150601f19603f3d011682016040523d82523d6000602084013e612148565b606091505b5091509150818015612176575080511580612176575080806020019051602081101561217357600080fd5b50515b6121e157604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f556e697377617056323a205452414e534645525f4641494c4544000000000000604482015290519081900360640190fd5b5050505050565b60008115806122035750508082028282828161220057fe5b04145b610df657604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601460248201527f64732d6d6174682d6d756c2d6f766572666c6f77000000000000000000000000604482015290519081900360640190fd5b80820382811115610df657604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601560248201527f64732d6d6174682d7375622d756e646572666c6f770000000000000000000000604482015290519081900360640190fd5b6dffffffffffffffffffffffffffff841180159061230c57506dffffffffffffffffffffffffffff8311155b61237757604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601360248201527f556e697377617056323a204f564552464c4f5700000000000000000000000000604482015290519081900360640190fd5b60085463ffffffff428116917c0100000000000000000000000000000000000000000000000000000000900481168203908116158015906123c757506dffffffffffffffffffffffffffff841615155b80156123e257506dffffffffffffffffffffffffffff831615155b15612492578063ffffffff16612425856123fb86612a57565b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff169063ffffffff612a7b16565b600980547bffffffffffffffffffffffffffffffffffffffffffffffffffffffff929092169290920201905563ffffffff8116612465846123fb87612a57565b600a80547bffffffffffffffffffffffffffffffffffffffffffffffffffffffff92909216929092020190555b600880547fffffffffffffffffffffffffffffffffffff0000000000000000000000000000166dffffffffffffffffffffffffffff888116919091177fffffffff0000000000000000000000000000ffffffffffffffffffffffffffff166e0100000000000000000000000000008883168102919091177bffffffffffffffffffffffffffffffffffffffffffffffffffffffff167c010000000000000000000000000000000000000000000000000000000063ffffffff871602179283905560408051848416815291909304909116602082015281517f1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1929181900390910190a1505050505050565b73ffffffffffffffffffffffffffffffffffffffff808416600081815260026020908152604080832094871680845294825291829020859055815185815291517f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9259281900390910190a3505050565b73ffffffffffffffffffffffffffffffffffffffff8316600090815260016020526040902054612641908263ffffffff61226e16565b73ffffffffffffffffffffffffffffffffffffffff8085166000908152600160205260408082209390935590841681522054612683908263ffffffff612abc16565b73ffffffffffffffffffffffffffffffffffffffff80841660008181526001602090815260409182902094909455805185815290519193928716927fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef92918290030190a3505050565b600080600560009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1663017e7e586040518163ffffffff1660e01b815260040160206040518083038186803b15801561275757600080fd5b505afa15801561276b573d6000803e3d6000fd5b505050506040513d602081101561278157600080fd5b5051600b5473ffffffffffffffffffffffffffffffffffffffff821615801594509192509061286457801561285f5760006127d86112576dffffffffffffffffffffffffffff88811690881663ffffffff6121e816565b905060006127e583612878565b90508082111561285c576000612813612804848463ffffffff61226e16565b6000549063ffffffff6121e816565b905060006128388361282c86600563ffffffff6121e816565b9063ffffffff612abc16565b9050600081838161284557fe5b04905080156128585761285887826128ca565b5050505b50505b612870565b8015612870576000600b555b505092915050565b600060038211156128bb575080600160028204015b818110156128b5578091506002818285816128a457fe5b0401816128ad57fe5b04905061288d565b506128c5565b81156128c5575060015b919050565b6000546128dd908263ffffffff612abc16565b600090815573ffffffffffffffffffffffffffffffffffffffff8316815260016020526040902054612915908263ffffffff612abc16565b73ffffffffffffffffffffffffffffffffffffffff831660008181526001602090815260408083209490945583518581529351929391927fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9281900390910190a35050565b6000818310612989578161298b565b825b9392505050565b73ffffffffffffffffffffffffffffffffffffffff82166000908152600160205260409020546129c8908263ffffffff61226e16565b73ffffffffffffffffffffffffffffffffffffffff831660009081526001602052604081209190915554612a02908263ffffffff61226e16565b600090815560408051838152905173ffffffffffffffffffffffffffffffffffffffff8516917fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef919081900360200190a35050565b6dffffffffffffffffffffffffffff166e0100000000000000000000000000000290565b60006dffffffffffffffffffffffffffff82167bffffffffffffffffffffffffffffffffffffffffffffffffffffffff841681612ab457fe5b049392505050565b80820182811015610df657604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601460248201527f64732d6d6174682d6164642d6f766572666c6f77000000000000000000000000604482015290519081900360640190fdfe556e697377617056323a20494e53554646494349454e545f4f55545055545f414d4f554e54556e697377617056323a20494e53554646494349454e545f494e5055545f414d4f554e54556e697377617056323a20494e53554646494349454e545f4c4951554944495459556e697377617056323a20494e53554646494349454e545f4c49515549444954595f4255524e4544556e697377617056323a20494e53554646494349454e545f4c49515549444954595f4d494e544544a265627a7a723158207dca18479e58487606bf70c79e44d8dee62353c9ee6d01f9a9d70885b8765f2264736f6c63430005100032454950373132446f6d61696e28737472696e67206e616d652c737472696e672076657273696f6e2c75696e7432353620636861696e49642c6164647265737320766572696679696e67436f6e747261637429a265627a7a723158202760f92d7fa1db6f5aa16307bad65df4ebcc8550c4b1f03755ab8dfd830c178f64736f6c63430005100032",
      "opcodes": "PUSH1 0x80 PUSH1 0x40 MSTORE CALLVALUE DUP1 ISZERO PUSH2 0x10 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP PUSH1 0x40 MLOAD PUSH2 0x3686 CODESIZE SUB DUP1 PUSH2 0x3686 DUP4 CODECOPY DUP2 DUP2 ADD PUSH1 0x40 MSTORE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x33 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD PUSH1 0x1 DUP1 SLOAD PUSH1 0x1 PUSH1 0x1 PUSH1 0xA0 SHL SUB NOT AND PUSH1 0x1 PUSH1 0x1 PUSH1 0xA0 SHL SUB SWAP1 SWAP3 AND SWAP2 SWAP1 SWAP2 OR SWAP1 SSTORE PUSH2 0x3623 DUP1 PUSH2 0x63 PUSH1 0x0 CODECOPY PUSH1 0x0 RETURN INVALID PUSH1 0x80 PUSH1 0x40 MSTORE CALLVALUE DUP1 ISZERO PUSH2 0x10 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP PUSH1 0x4 CALLDATASIZE LT PUSH2 0x88 JUMPI PUSH1 0x0 CALLDATALOAD PUSH1 0xE0 SHR DUP1 PUSH4 0xA2E74AF6 GT PUSH2 0x5B JUMPI DUP1 PUSH4 0xA2E74AF6 EQ PUSH2 0xFD JUMPI DUP1 PUSH4 0xC9C65396 EQ PUSH2 0x132 JUMPI DUP1 PUSH4 0xE6A43905 EQ PUSH2 0x16D JUMPI DUP1 PUSH4 0xF46901ED EQ PUSH2 0x1A8 JUMPI PUSH2 0x88 JUMP JUMPDEST DUP1 PUSH4 0x17E7E58 EQ PUSH2 0x8D JUMPI DUP1 PUSH4 0x94B7415 EQ PUSH2 0xBE JUMPI DUP1 PUSH4 0x1E3DD18B EQ PUSH2 0xC6 JUMPI DUP1 PUSH4 0x574F2BA3 EQ PUSH2 0xE3 JUMPI JUMPDEST PUSH1 0x0 DUP1 REVERT JUMPDEST PUSH2 0x95 PUSH2 0x1DB JUMP JUMPDEST PUSH1 0x40 DUP1 MLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP1 SWAP3 AND DUP3 MSTORE MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x20 ADD SWAP1 RETURN JUMPDEST PUSH2 0x95 PUSH2 0x1F7 JUMP JUMPDEST PUSH2 0x95 PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x20 DUP2 LT ISZERO PUSH2 0xDC JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP CALLDATALOAD PUSH2 0x213 JUMP JUMPDEST PUSH2 0xEB PUSH2 0x247 JUMP JUMPDEST PUSH1 0x40 DUP1 MLOAD SWAP2 DUP3 MSTORE MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x20 ADD SWAP1 RETURN JUMPDEST PUSH2 0x130 PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x113 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP CALLDATALOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH2 0x24D JUMP JUMPDEST STOP JUMPDEST PUSH2 0x95 PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x40 DUP2 LT ISZERO PUSH2 0x148 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP2 CALLDATALOAD DUP2 AND SWAP2 PUSH1 0x20 ADD CALLDATALOAD AND PUSH2 0x31A JUMP JUMPDEST PUSH2 0x95 PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x40 DUP2 LT ISZERO PUSH2 0x183 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP2 CALLDATALOAD DUP2 AND SWAP2 PUSH1 0x20 ADD CALLDATALOAD AND PUSH2 0x76D JUMP JUMPDEST PUSH2 0x130 PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x1BE JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP CALLDATALOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH2 0x7A0 JUMP JUMPDEST PUSH1 0x0 SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP2 JUMP JUMPDEST PUSH1 0x1 SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP2 JUMP JUMPDEST PUSH1 0x3 DUP2 DUP2 SLOAD DUP2 LT PUSH2 0x220 JUMPI INVALID JUMPDEST PUSH1 0x0 SWAP2 DUP3 MSTORE PUSH1 0x20 SWAP1 SWAP2 KECCAK256 ADD SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND SWAP1 POP DUP2 JUMP JUMPDEST PUSH1 0x3 SLOAD SWAP1 JUMP JUMPDEST PUSH1 0x1 SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND CALLER EQ PUSH2 0x2D3 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x14 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A20464F5242494444454E000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH1 0x1 DUP1 SLOAD PUSH32 0xFFFFFFFFFFFFFFFFFFFFFFFF0000000000000000000000000000000000000000 AND PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP3 SWAP1 SWAP3 AND SWAP2 SWAP1 SWAP2 OR SWAP1 SSTORE JUMP JUMPDEST PUSH1 0x0 DUP2 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP4 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND EQ ISZERO PUSH2 0x3B7 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x1E PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A204944454E544943414C5F4144445245535345530000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH1 0x0 DUP1 DUP4 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP6 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND LT PUSH2 0x3F4 JUMPI DUP4 DUP6 PUSH2 0x3F7 JUMP JUMPDEST DUP5 DUP5 JUMPDEST SWAP1 SWAP3 POP SWAP1 POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP3 AND PUSH2 0x47E JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x17 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A205A45524F5F41444452455353000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP3 DUP2 AND PUSH1 0x0 SWAP1 DUP2 MSTORE PUSH1 0x2 PUSH1 0x20 SWAP1 DUP2 MSTORE PUSH1 0x40 DUP1 DUP4 KECCAK256 DUP6 DUP6 AND DUP5 MSTORE SWAP1 SWAP2 MSTORE SWAP1 KECCAK256 SLOAD AND ISZERO PUSH2 0x51F JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x16 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A20504149525F45584953545300000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH1 0x60 PUSH1 0x40 MLOAD DUP1 PUSH1 0x20 ADD PUSH2 0x531 SWAP1 PUSH2 0x86D JUMP JUMPDEST PUSH1 0x20 DUP3 ADD DUP2 SUB DUP3 MSTORE PUSH1 0x1F NOT PUSH1 0x1F DUP3 ADD AND PUSH1 0x40 MSTORE POP SWAP1 POP PUSH1 0x0 DUP4 DUP4 PUSH1 0x40 MLOAD PUSH1 0x20 ADD DUP1 DUP4 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH1 0x60 SHL DUP2 MSTORE PUSH1 0x14 ADD DUP3 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH1 0x60 SHL DUP2 MSTORE PUSH1 0x14 ADD SWAP3 POP POP POP PUSH1 0x40 MLOAD PUSH1 0x20 DUP2 DUP4 SUB SUB DUP2 MSTORE SWAP1 PUSH1 0x40 MSTORE DUP1 MLOAD SWAP1 PUSH1 0x20 ADD KECCAK256 SWAP1 POP DUP1 DUP3 MLOAD PUSH1 0x20 DUP5 ADD PUSH1 0x0 CREATE2 PUSH1 0x40 DUP1 MLOAD PUSH32 0x485CC95500000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP8 DUP2 AND PUSH1 0x4 DUP4 ADD MSTORE DUP7 DUP2 AND PUSH1 0x24 DUP4 ADD MSTORE SWAP2 MLOAD SWAP3 SWAP8 POP SWAP1 DUP8 AND SWAP2 PUSH4 0x485CC955 SWAP2 PUSH1 0x44 DUP1 DUP3 ADD SWAP3 PUSH1 0x0 SWAP3 SWAP1 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP4 DUP8 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x65E JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS CALL ISZERO DUP1 ISZERO PUSH2 0x672 JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP5 DUP2 AND PUSH1 0x0 DUP2 DUP2 MSTORE PUSH1 0x2 PUSH1 0x20 DUP2 DUP2 MSTORE PUSH1 0x40 DUP1 DUP5 KECCAK256 DUP10 DUP8 AND DUP1 DUP7 MSTORE SWAP1 DUP4 MSTORE DUP2 DUP6 KECCAK256 DUP1 SLOAD SWAP8 DUP14 AND PUSH32 0xFFFFFFFFFFFFFFFFFFFFFFFF0000000000000000000000000000000000000000 SWAP9 DUP10 AND DUP2 OR SWAP1 SWAP2 SSTORE SWAP4 DUP4 MSTORE DUP2 DUP6 KECCAK256 DUP7 DUP7 MSTORE DUP4 MSTORE DUP2 DUP6 KECCAK256 DUP1 SLOAD DUP9 AND DUP6 OR SWAP1 SSTORE PUSH1 0x3 DUP1 SLOAD PUSH1 0x1 DUP2 ADD DUP3 SSTORE SWAP6 DUP2 SWAP1 MSTORE PUSH32 0xC2575A0E9E593C00F959F8C92F12DB2869C3395A3B0502D05E2516446F71F85B SWAP1 SWAP6 ADD DUP1 SLOAD SWAP1 SWAP8 AND DUP5 OR SWAP1 SWAP7 SSTORE SWAP3 SLOAD DUP4 MLOAD SWAP3 DUP4 MSTORE SWAP1 DUP3 ADD MSTORE DUP2 MLOAD PUSH32 0xD3648BD0F6BA80134A33BA9275AC585D9D315F0AD8355CDDEFDE31AFA28D0E9 SWAP3 SWAP2 DUP2 SWAP1 SUB SWAP1 SWAP2 ADD SWAP1 LOG3 POP POP POP POP SWAP3 SWAP2 POP POP JUMP JUMPDEST PUSH1 0x2 PUSH1 0x20 SWAP1 DUP2 MSTORE PUSH1 0x0 SWAP3 DUP4 MSTORE PUSH1 0x40 DUP1 DUP5 KECCAK256 SWAP1 SWAP2 MSTORE SWAP1 DUP3 MSTORE SWAP1 KECCAK256 SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP2 JUMP JUMPDEST PUSH1 0x1 SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND CALLER EQ PUSH2 0x826 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x14 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A20464F5242494444454E000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH1 0x0 DUP1 SLOAD PUSH32 0xFFFFFFFFFFFFFFFFFFFFFFFF0000000000000000000000000000000000000000 AND PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP3 SWAP1 SWAP3 AND SWAP2 SWAP1 SWAP2 OR SWAP1 SSTORE JUMP JUMPDEST PUSH2 0x2D74 DUP1 PUSH2 0x87B DUP4 CODECOPY ADD SWAP1 JUMP INVALID PUSH1 0x80 PUSH1 0x40 MSTORE PUSH1 0x1 PUSH1 0xC SSTORE CALLVALUE DUP1 ISZERO PUSH2 0x15 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP PUSH1 0x40 MLOAD CHAINID SWAP1 DUP1 PUSH1 0x52 PUSH2 0x2D22 DUP3 CODECOPY PUSH1 0x40 DUP1 MLOAD SWAP2 DUP3 SWAP1 SUB PUSH1 0x52 ADD DUP3 KECCAK256 DUP3 DUP3 ADD DUP3 MSTORE PUSH1 0xA DUP4 MSTORE PUSH10 0x2AB734B9BBB0B8102B19 PUSH1 0xB1 SHL PUSH1 0x20 SWAP4 DUP5 ADD MSTORE DUP2 MLOAD DUP1 DUP4 ADD DUP4 MSTORE PUSH1 0x1 DUP2 MSTORE PUSH1 0x31 PUSH1 0xF8 SHL SWAP1 DUP5 ADD MSTORE DUP2 MLOAD DUP1 DUP5 ADD SWAP2 SWAP1 SWAP2 MSTORE PUSH32 0xBFCC8EF98FFBF7B6C3FEC7BF5185B566B9863E35A9D83ACD49AD6824B5969738 DUP2 DUP4 ADD MSTORE PUSH32 0xC89EFDAA54C0F20C7ADF612882DF0950F5A951637E0307CDCB4C672F298B8BC6 PUSH1 0x60 DUP3 ADD MSTORE PUSH1 0x80 DUP2 ADD SWAP5 SWAP1 SWAP5 MSTORE ADDRESS PUSH1 0xA0 DUP1 DUP7 ADD SWAP2 SWAP1 SWAP2 MSTORE DUP2 MLOAD DUP1 DUP7 SUB SWAP1 SWAP2 ADD DUP2 MSTORE PUSH1 0xC0 SWAP1 SWAP5 ADD SWAP1 MSTORE DUP3 MLOAD SWAP3 ADD SWAP2 SWAP1 SWAP2 KECCAK256 PUSH1 0x3 SSTORE POP PUSH1 0x5 DUP1 SLOAD PUSH1 0x1 PUSH1 0x1 PUSH1 0xA0 SHL SUB NOT AND CALLER OR SWAP1 SSTORE PUSH2 0x2C1D DUP1 PUSH2 0x105 PUSH1 0x0 CODECOPY PUSH1 0x0 RETURN INVALID PUSH1 0x80 PUSH1 0x40 MSTORE CALLVALUE DUP1 ISZERO PUSH2 0x10 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP PUSH1 0x4 CALLDATASIZE LT PUSH2 0x1B9 JUMPI PUSH1 0x0 CALLDATALOAD PUSH1 0xE0 SHR DUP1 PUSH4 0x6A627842 GT PUSH2 0xF9 JUMPI DUP1 PUSH4 0xBA9A7A56 GT PUSH2 0x97 JUMPI DUP1 PUSH4 0xD21220A7 GT PUSH2 0x71 JUMPI DUP1 PUSH4 0xD21220A7 EQ PUSH2 0x5DA JUMPI DUP1 PUSH4 0xD505ACCF EQ PUSH2 0x5E2 JUMPI DUP1 PUSH4 0xDD62ED3E EQ PUSH2 0x640 JUMPI DUP1 PUSH4 0xFFF6CAE9 EQ PUSH2 0x67B JUMPI PUSH2 0x1B9 JUMP JUMPDEST DUP1 PUSH4 0xBA9A7A56 EQ PUSH2 0x597 JUMPI DUP1 PUSH4 0xBC25CF77 EQ PUSH2 0x59F JUMPI DUP1 PUSH4 0xC45A0155 EQ PUSH2 0x5D2 JUMPI PUSH2 0x1B9 JUMP JUMPDEST DUP1 PUSH4 0x7ECEBE00 GT PUSH2 0xD3 JUMPI DUP1 PUSH4 0x7ECEBE00 EQ PUSH2 0x4D7 JUMPI DUP1 PUSH4 0x89AFCB44 EQ PUSH2 0x50A JUMPI DUP1 PUSH4 0x95D89B41 EQ PUSH2 0x556 JUMPI DUP1 PUSH4 0xA9059CBB EQ PUSH2 0x55E JUMPI PUSH2 0x1B9 JUMP JUMPDEST DUP1 PUSH4 0x6A627842 EQ PUSH2 0x469 JUMPI DUP1 PUSH4 0x70A08231 EQ PUSH2 0x49C JUMPI DUP1 PUSH4 0x7464FC3D EQ PUSH2 0x4CF JUMPI PUSH2 0x1B9 JUMP JUMPDEST DUP1 PUSH4 0x23B872DD GT PUSH2 0x166 JUMPI DUP1 PUSH4 0x3644E515 GT PUSH2 0x140 JUMPI DUP1 PUSH4 0x3644E515 EQ PUSH2 0x416 JUMPI DUP1 PUSH4 0x485CC955 EQ PUSH2 0x41E JUMPI DUP1 PUSH4 0x5909C0D5 EQ PUSH2 0x459 JUMPI DUP1 PUSH4 0x5A3D5493 EQ PUSH2 0x461 JUMPI PUSH2 0x1B9 JUMP JUMPDEST DUP1 PUSH4 0x23B872DD EQ PUSH2 0x3AD JUMPI DUP1 PUSH4 0x30ADF81F EQ PUSH2 0x3F0 JUMPI DUP1 PUSH4 0x313CE567 EQ PUSH2 0x3F8 JUMPI PUSH2 0x1B9 JUMP JUMPDEST DUP1 PUSH4 0x95EA7B3 GT PUSH2 0x197 JUMPI DUP1 PUSH4 0x95EA7B3 EQ PUSH2 0x315 JUMPI DUP1 PUSH4 0xDFE1681 EQ PUSH2 0x362 JUMPI DUP1 PUSH4 0x18160DDD EQ PUSH2 0x393 JUMPI PUSH2 0x1B9 JUMP JUMPDEST DUP1 PUSH4 0x22C0D9F EQ PUSH2 0x1BE JUMPI DUP1 PUSH4 0x6FDDE03 EQ PUSH2 0x259 JUMPI DUP1 PUSH4 0x902F1AC EQ PUSH2 0x2D6 JUMPI JUMPDEST PUSH1 0x0 DUP1 REVERT JUMPDEST PUSH2 0x257 PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x80 DUP2 LT ISZERO PUSH2 0x1D4 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST DUP2 CALLDATALOAD SWAP2 PUSH1 0x20 DUP2 ADD CALLDATALOAD SWAP2 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF PUSH1 0x40 DUP4 ADD CALLDATALOAD AND SWAP2 SWAP1 DUP2 ADD SWAP1 PUSH1 0x80 DUP2 ADD PUSH1 0x60 DUP3 ADD CALLDATALOAD PUSH5 0x100000000 DUP2 GT ISZERO PUSH2 0x218 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST DUP3 ADD DUP4 PUSH1 0x20 DUP3 ADD GT ISZERO PUSH2 0x22A JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST DUP1 CALLDATALOAD SWAP1 PUSH1 0x20 ADD SWAP2 DUP5 PUSH1 0x1 DUP4 MUL DUP5 ADD GT PUSH5 0x100000000 DUP4 GT OR ISZERO PUSH2 0x24C JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP SWAP1 SWAP3 POP SWAP1 POP PUSH2 0x683 JUMP JUMPDEST STOP JUMPDEST PUSH2 0x261 PUSH2 0xD57 JUMP JUMPDEST PUSH1 0x40 DUP1 MLOAD PUSH1 0x20 DUP1 DUP3 MSTORE DUP4 MLOAD DUP2 DUP4 ADD MSTORE DUP4 MLOAD SWAP2 SWAP3 DUP4 SWAP3 SWAP1 DUP4 ADD SWAP2 DUP6 ADD SWAP1 DUP1 DUP4 DUP4 PUSH1 0x0 JUMPDEST DUP4 DUP2 LT ISZERO PUSH2 0x29B JUMPI DUP2 DUP2 ADD MLOAD DUP4 DUP3 ADD MSTORE PUSH1 0x20 ADD PUSH2 0x283 JUMP JUMPDEST POP POP POP POP SWAP1 POP SWAP1 DUP2 ADD SWAP1 PUSH1 0x1F AND DUP1 ISZERO PUSH2 0x2C8 JUMPI DUP1 DUP3 SUB DUP1 MLOAD PUSH1 0x1 DUP4 PUSH1 0x20 SUB PUSH2 0x100 EXP SUB NOT AND DUP2 MSTORE PUSH1 0x20 ADD SWAP2 POP JUMPDEST POP SWAP3 POP POP POP PUSH1 0x40 MLOAD DUP1 SWAP2 SUB SWAP1 RETURN JUMPDEST PUSH2 0x2DE PUSH2 0xD90 JUMP JUMPDEST PUSH1 0x40 DUP1 MLOAD PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP5 DUP6 AND DUP2 MSTORE SWAP3 SWAP1 SWAP4 AND PUSH1 0x20 DUP4 ADD MSTORE PUSH4 0xFFFFFFFF AND DUP2 DUP4 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x60 ADD SWAP1 RETURN JUMPDEST PUSH2 0x34E PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x40 DUP2 LT ISZERO PUSH2 0x32B JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP2 CALLDATALOAD AND SWAP1 PUSH1 0x20 ADD CALLDATALOAD PUSH2 0xDE5 JUMP JUMPDEST PUSH1 0x40 DUP1 MLOAD SWAP2 ISZERO ISZERO DUP3 MSTORE MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x20 ADD SWAP1 RETURN JUMPDEST PUSH2 0x36A PUSH2 0xDFC JUMP JUMPDEST PUSH1 0x40 DUP1 MLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP1 SWAP3 AND DUP3 MSTORE MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x20 ADD SWAP1 RETURN JUMPDEST PUSH2 0x39B PUSH2 0xE18 JUMP JUMPDEST PUSH1 0x40 DUP1 MLOAD SWAP2 DUP3 MSTORE MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x20 ADD SWAP1 RETURN JUMPDEST PUSH2 0x34E PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x60 DUP2 LT ISZERO PUSH2 0x3C3 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP2 CALLDATALOAD DUP2 AND SWAP2 PUSH1 0x20 DUP2 ADD CALLDATALOAD SWAP1 SWAP2 AND SWAP1 PUSH1 0x40 ADD CALLDATALOAD PUSH2 0xE1E JUMP JUMPDEST PUSH2 0x39B PUSH2 0xEFD JUMP JUMPDEST PUSH2 0x400 PUSH2 0xF21 JUMP JUMPDEST PUSH1 0x40 DUP1 MLOAD PUSH1 0xFF SWAP1 SWAP3 AND DUP3 MSTORE MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x20 ADD SWAP1 RETURN JUMPDEST PUSH2 0x39B PUSH2 0xF26 JUMP JUMPDEST PUSH2 0x257 PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x40 DUP2 LT ISZERO PUSH2 0x434 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP2 CALLDATALOAD DUP2 AND SWAP2 PUSH1 0x20 ADD CALLDATALOAD AND PUSH2 0xF2C JUMP JUMPDEST PUSH2 0x39B PUSH2 0x1005 JUMP JUMPDEST PUSH2 0x39B PUSH2 0x100B JUMP JUMPDEST PUSH2 0x39B PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x47F JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP CALLDATALOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH2 0x1011 JUMP JUMPDEST PUSH2 0x39B PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x4B2 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP CALLDATALOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH2 0x13CB JUMP JUMPDEST PUSH2 0x39B PUSH2 0x13DD JUMP JUMPDEST PUSH2 0x39B PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x4ED JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP CALLDATALOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH2 0x13E3 JUMP JUMPDEST PUSH2 0x53D PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x520 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP CALLDATALOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH2 0x13F5 JUMP JUMPDEST PUSH1 0x40 DUP1 MLOAD SWAP3 DUP4 MSTORE PUSH1 0x20 DUP4 ADD SWAP2 SWAP1 SWAP2 MSTORE DUP1 MLOAD SWAP2 DUP3 SWAP1 SUB ADD SWAP1 RETURN JUMPDEST PUSH2 0x261 PUSH2 0x1892 JUMP JUMPDEST PUSH2 0x34E PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x40 DUP2 LT ISZERO PUSH2 0x574 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP2 CALLDATALOAD AND SWAP1 PUSH1 0x20 ADD CALLDATALOAD PUSH2 0x18CB JUMP JUMPDEST PUSH2 0x39B PUSH2 0x18D8 JUMP JUMPDEST PUSH2 0x257 PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x5B5 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP CALLDATALOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH2 0x18DE JUMP JUMPDEST PUSH2 0x36A PUSH2 0x1AD4 JUMP JUMPDEST PUSH2 0x36A PUSH2 0x1AF0 JUMP JUMPDEST PUSH2 0x257 PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0xE0 DUP2 LT ISZERO PUSH2 0x5F8 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP2 CALLDATALOAD DUP2 AND SWAP2 PUSH1 0x20 DUP2 ADD CALLDATALOAD SWAP1 SWAP2 AND SWAP1 PUSH1 0x40 DUP2 ADD CALLDATALOAD SWAP1 PUSH1 0x60 DUP2 ADD CALLDATALOAD SWAP1 PUSH1 0xFF PUSH1 0x80 DUP3 ADD CALLDATALOAD AND SWAP1 PUSH1 0xA0 DUP2 ADD CALLDATALOAD SWAP1 PUSH1 0xC0 ADD CALLDATALOAD PUSH2 0x1B0C JUMP JUMPDEST PUSH2 0x39B PUSH1 0x4 DUP1 CALLDATASIZE SUB PUSH1 0x40 DUP2 LT ISZERO PUSH2 0x656 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP2 CALLDATALOAD DUP2 AND SWAP2 PUSH1 0x20 ADD CALLDATALOAD AND PUSH2 0x1DD8 JUMP JUMPDEST PUSH2 0x257 PUSH2 0x1DF5 JUMP JUMPDEST PUSH1 0xC SLOAD PUSH1 0x1 EQ PUSH2 0x6F4 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x11 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A204C4F434B4544000000000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH1 0x0 PUSH1 0xC SSTORE DUP5 ISZERO ISZERO DUP1 PUSH2 0x707 JUMPI POP PUSH1 0x0 DUP5 GT JUMPDEST PUSH2 0x75C JUMPI PUSH1 0x40 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x4 ADD DUP1 DUP1 PUSH1 0x20 ADD DUP3 DUP2 SUB DUP3 MSTORE PUSH1 0x25 DUP2 MSTORE PUSH1 0x20 ADD DUP1 PUSH2 0x2B2F PUSH1 0x25 SWAP2 CODECOPY PUSH1 0x40 ADD SWAP2 POP POP PUSH1 0x40 MLOAD DUP1 SWAP2 SUB SWAP1 REVERT JUMPDEST PUSH1 0x0 DUP1 PUSH2 0x767 PUSH2 0xD90 JUMP JUMPDEST POP SWAP2 POP SWAP2 POP DUP2 PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP8 LT DUP1 ISZERO PUSH2 0x79A JUMPI POP DUP1 PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP7 LT JUMPDEST PUSH2 0x7EF JUMPI PUSH1 0x40 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x4 ADD DUP1 DUP1 PUSH1 0x20 ADD DUP3 DUP2 SUB DUP3 MSTORE PUSH1 0x21 DUP2 MSTORE PUSH1 0x20 ADD DUP1 PUSH2 0x2B78 PUSH1 0x21 SWAP2 CODECOPY PUSH1 0x40 ADD SWAP2 POP POP PUSH1 0x40 MLOAD DUP1 SWAP2 SUB SWAP1 REVERT JUMPDEST PUSH1 0x6 SLOAD PUSH1 0x7 SLOAD PUSH1 0x0 SWAP2 DUP3 SWAP2 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP2 DUP3 AND SWAP2 SWAP1 DUP2 AND SWAP1 DUP10 AND DUP3 EQ DUP1 ISZERO SWAP1 PUSH2 0x854 JUMPI POP DUP1 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP10 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND EQ ISZERO JUMPDEST PUSH2 0x8BF JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x15 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A20494E56414C49445F544F0000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST DUP11 ISZERO PUSH2 0x8D0 JUMPI PUSH2 0x8D0 DUP3 DUP11 DUP14 PUSH2 0x1FDB JUMP JUMPDEST DUP10 ISZERO PUSH2 0x8E1 JUMPI PUSH2 0x8E1 DUP2 DUP11 DUP13 PUSH2 0x1FDB JUMP JUMPDEST DUP7 ISZERO PUSH2 0x9C3 JUMPI DUP9 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH4 0x10D1E85C CALLER DUP14 DUP14 DUP13 DUP13 PUSH1 0x40 MLOAD DUP7 PUSH4 0xFFFFFFFF AND PUSH1 0xE0 SHL DUP2 MSTORE PUSH1 0x4 ADD DUP1 DUP7 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP2 MSTORE PUSH1 0x20 ADD DUP6 DUP2 MSTORE PUSH1 0x20 ADD DUP5 DUP2 MSTORE PUSH1 0x20 ADD DUP1 PUSH1 0x20 ADD DUP3 DUP2 SUB DUP3 MSTORE DUP5 DUP5 DUP3 DUP2 DUP2 MSTORE PUSH1 0x20 ADD SWAP3 POP DUP1 DUP3 DUP5 CALLDATACOPY PUSH1 0x0 DUP2 DUP5 ADD MSTORE PUSH1 0x1F NOT PUSH1 0x1F DUP3 ADD AND SWAP1 POP DUP1 DUP4 ADD SWAP3 POP POP POP SWAP7 POP POP POP POP POP POP POP PUSH1 0x0 PUSH1 0x40 MLOAD DUP1 DUP4 SUB DUP2 PUSH1 0x0 DUP8 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x9AA JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS CALL ISZERO DUP1 ISZERO PUSH2 0x9BE JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP JUMPDEST PUSH1 0x40 DUP1 MLOAD PUSH32 0x70A0823100000000000000000000000000000000000000000000000000000000 DUP2 MSTORE ADDRESS PUSH1 0x4 DUP3 ADD MSTORE SWAP1 MLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP5 AND SWAP2 PUSH4 0x70A08231 SWAP2 PUSH1 0x24 DUP1 DUP4 ADD SWAP3 PUSH1 0x20 SWAP3 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0xA2F JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0xA43 JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH1 0x40 MLOAD RETURNDATASIZE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0xA59 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD PUSH1 0x40 DUP1 MLOAD PUSH32 0x70A0823100000000000000000000000000000000000000000000000000000000 DUP2 MSTORE ADDRESS PUSH1 0x4 DUP3 ADD MSTORE SWAP1 MLOAD SWAP2 SWAP6 POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP4 AND SWAP2 PUSH4 0x70A08231 SWAP2 PUSH1 0x24 DUP1 DUP3 ADD SWAP3 PUSH1 0x20 SWAP3 SWAP1 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0xACB JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0xADF JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH1 0x40 MLOAD RETURNDATASIZE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0xAF5 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD SWAP3 POP PUSH1 0x0 SWAP2 POP POP PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP6 AND DUP11 SWAP1 SUB DUP4 GT PUSH2 0xB1F JUMPI PUSH1 0x0 PUSH2 0xB35 JUMP JUMPDEST DUP10 DUP6 PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND SUB DUP4 SUB JUMPDEST SWAP1 POP PUSH1 0x0 DUP10 DUP6 PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND SUB DUP4 GT PUSH2 0xB59 JUMPI PUSH1 0x0 PUSH2 0xB6F JUMP JUMPDEST DUP10 DUP6 PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND SUB DUP4 SUB JUMPDEST SWAP1 POP PUSH1 0x0 DUP3 GT DUP1 PUSH2 0xB80 JUMPI POP PUSH1 0x0 DUP2 GT JUMPDEST PUSH2 0xBD5 JUMPI PUSH1 0x40 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x4 ADD DUP1 DUP1 PUSH1 0x20 ADD DUP3 DUP2 SUB DUP3 MSTORE PUSH1 0x24 DUP2 MSTORE PUSH1 0x20 ADD DUP1 PUSH2 0x2B54 PUSH1 0x24 SWAP2 CODECOPY PUSH1 0x40 ADD SWAP2 POP POP PUSH1 0x40 MLOAD DUP1 SWAP2 SUB SWAP1 REVERT JUMPDEST PUSH1 0x0 PUSH2 0xC09 PUSH2 0xBEB DUP5 PUSH1 0x3 PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST PUSH2 0xBFD DUP8 PUSH2 0x3E8 PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST SWAP1 PUSH4 0xFFFFFFFF PUSH2 0x226E AND JUMP JUMPDEST SWAP1 POP PUSH1 0x0 PUSH2 0xC21 PUSH2 0xBEB DUP5 PUSH1 0x3 PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST SWAP1 POP PUSH2 0xC59 PUSH3 0xF4240 PUSH2 0xC4D PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP12 DUP2 AND SWAP1 DUP12 AND PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST SWAP1 PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST PUSH2 0xC69 DUP4 DUP4 PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST LT ISZERO PUSH2 0xCD6 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0xC PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A204B0000000000000000000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST POP POP PUSH2 0xCE4 DUP5 DUP5 DUP9 DUP9 PUSH2 0x22E0 JUMP JUMPDEST PUSH1 0x40 DUP1 MLOAD DUP4 DUP2 MSTORE PUSH1 0x20 DUP2 ADD DUP4 SWAP1 MSTORE DUP1 DUP3 ADD DUP14 SWAP1 MSTORE PUSH1 0x60 DUP2 ADD DUP13 SWAP1 MSTORE SWAP1 MLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP12 AND SWAP2 CALLER SWAP2 PUSH32 0xD78AD95FA46C994B6551D0DA85FC275FE613CE37657FB8D5E3D130840159D822 SWAP2 DUP2 SWAP1 SUB PUSH1 0x80 ADD SWAP1 LOG3 POP POP PUSH1 0x1 PUSH1 0xC SSTORE POP POP POP POP POP POP POP POP POP JUMP JUMPDEST PUSH1 0x40 MLOAD DUP1 PUSH1 0x40 ADD PUSH1 0x40 MSTORE DUP1 PUSH1 0xA DUP2 MSTORE PUSH1 0x20 ADD PUSH32 0x556E697377617020563200000000000000000000000000000000000000000000 DUP2 MSTORE POP DUP2 JUMP JUMPDEST PUSH1 0x8 SLOAD PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP1 DUP3 AND SWAP3 PUSH15 0x10000000000000000000000000000 DUP4 DIV SWAP1 SWAP2 AND SWAP2 PUSH29 0x100000000000000000000000000000000000000000000000000000000 SWAP1 DIV PUSH4 0xFFFFFFFF AND SWAP1 JUMP JUMPDEST PUSH1 0x0 PUSH2 0xDF2 CALLER DUP5 DUP5 PUSH2 0x259C JUMP JUMPDEST POP PUSH1 0x1 JUMPDEST SWAP3 SWAP2 POP POP JUMP JUMPDEST PUSH1 0x6 SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP2 JUMP JUMPDEST PUSH1 0x0 SLOAD DUP2 JUMP JUMPDEST PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP4 AND PUSH1 0x0 SWAP1 DUP2 MSTORE PUSH1 0x2 PUSH1 0x20 SWAP1 DUP2 MSTORE PUSH1 0x40 DUP1 DUP4 KECCAK256 CALLER DUP5 MSTORE SWAP1 SWAP2 MSTORE DUP2 KECCAK256 SLOAD PUSH32 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF EQ PUSH2 0xEE8 JUMPI PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP5 AND PUSH1 0x0 SWAP1 DUP2 MSTORE PUSH1 0x2 PUSH1 0x20 SWAP1 DUP2 MSTORE PUSH1 0x40 DUP1 DUP4 KECCAK256 CALLER DUP5 MSTORE SWAP1 SWAP2 MSTORE SWAP1 KECCAK256 SLOAD PUSH2 0xEB6 SWAP1 DUP4 PUSH4 0xFFFFFFFF PUSH2 0x226E AND JUMP JUMPDEST PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP6 AND PUSH1 0x0 SWAP1 DUP2 MSTORE PUSH1 0x2 PUSH1 0x20 SWAP1 DUP2 MSTORE PUSH1 0x40 DUP1 DUP4 KECCAK256 CALLER DUP5 MSTORE SWAP1 SWAP2 MSTORE SWAP1 KECCAK256 SSTORE JUMPDEST PUSH2 0xEF3 DUP5 DUP5 DUP5 PUSH2 0x260B JUMP JUMPDEST POP PUSH1 0x1 SWAP4 SWAP3 POP POP POP JUMP JUMPDEST PUSH32 0x6E71EDAE12B1B97F4D1F60370FEF10105FA2FAAE0126114A169C64845D6126C9 DUP2 JUMP JUMPDEST PUSH1 0x12 DUP2 JUMP JUMPDEST PUSH1 0x3 SLOAD DUP2 JUMP JUMPDEST PUSH1 0x5 SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND CALLER EQ PUSH2 0xFB2 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x14 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A20464F5242494444454E000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH1 0x6 DUP1 SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP4 DUP5 AND PUSH32 0xFFFFFFFFFFFFFFFFFFFFFFFF0000000000000000000000000000000000000000 SWAP2 DUP3 AND OR SWAP1 SWAP2 SSTORE PUSH1 0x7 DUP1 SLOAD SWAP3 SWAP1 SWAP4 AND SWAP2 AND OR SWAP1 SSTORE JUMP JUMPDEST PUSH1 0x9 SLOAD DUP2 JUMP JUMPDEST PUSH1 0xA SLOAD DUP2 JUMP JUMPDEST PUSH1 0x0 PUSH1 0xC SLOAD PUSH1 0x1 EQ PUSH2 0x1084 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x11 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A204C4F434B4544000000000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH1 0x0 PUSH1 0xC DUP2 SWAP1 SSTORE DUP1 PUSH2 0x1094 PUSH2 0xD90 JUMP JUMPDEST POP PUSH1 0x6 SLOAD PUSH1 0x40 DUP1 MLOAD PUSH32 0x70A0823100000000000000000000000000000000000000000000000000000000 DUP2 MSTORE ADDRESS PUSH1 0x4 DUP3 ADD MSTORE SWAP1 MLOAD SWAP4 SWAP6 POP SWAP2 SWAP4 POP PUSH1 0x0 SWAP3 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP1 SWAP2 AND SWAP2 PUSH4 0x70A08231 SWAP2 PUSH1 0x24 DUP1 DUP4 ADD SWAP3 PUSH1 0x20 SWAP3 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x110E JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0x1122 JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH1 0x40 MLOAD RETURNDATASIZE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x1138 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD PUSH1 0x7 SLOAD PUSH1 0x40 DUP1 MLOAD PUSH32 0x70A0823100000000000000000000000000000000000000000000000000000000 DUP2 MSTORE ADDRESS PUSH1 0x4 DUP3 ADD MSTORE SWAP1 MLOAD SWAP3 SWAP4 POP PUSH1 0x0 SWAP3 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP1 SWAP3 AND SWAP2 PUSH4 0x70A08231 SWAP2 PUSH1 0x24 DUP1 DUP3 ADD SWAP3 PUSH1 0x20 SWAP3 SWAP1 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x11B1 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0x11C5 JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH1 0x40 MLOAD RETURNDATASIZE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x11DB JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD SWAP1 POP PUSH1 0x0 PUSH2 0x1201 DUP4 PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP8 AND PUSH4 0xFFFFFFFF PUSH2 0x226E AND JUMP JUMPDEST SWAP1 POP PUSH1 0x0 PUSH2 0x1225 DUP4 PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP8 AND PUSH4 0xFFFFFFFF PUSH2 0x226E AND JUMP JUMPDEST SWAP1 POP PUSH1 0x0 PUSH2 0x1233 DUP8 DUP8 PUSH2 0x26EC JUMP JUMPDEST PUSH1 0x0 SLOAD SWAP1 SWAP2 POP DUP1 PUSH2 0x1270 JUMPI PUSH2 0x125C PUSH2 0x3E8 PUSH2 0xBFD PUSH2 0x1257 DUP8 DUP8 PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST PUSH2 0x2878 JUMP JUMPDEST SWAP9 POP PUSH2 0x126B PUSH1 0x0 PUSH2 0x3E8 PUSH2 0x28CA JUMP JUMPDEST PUSH2 0x12CD JUMP JUMPDEST PUSH2 0x12CA PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP10 AND PUSH2 0x1294 DUP7 DUP5 PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST DUP2 PUSH2 0x129B JUMPI INVALID JUMPDEST DIV PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP10 AND PUSH2 0x12BD DUP7 DUP6 PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST DUP2 PUSH2 0x12C4 JUMPI INVALID JUMPDEST DIV PUSH2 0x297A JUMP JUMPDEST SWAP9 POP JUMPDEST PUSH1 0x0 DUP10 GT PUSH2 0x1326 JUMPI PUSH1 0x40 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x4 ADD DUP1 DUP1 PUSH1 0x20 ADD DUP3 DUP2 SUB DUP3 MSTORE PUSH1 0x28 DUP2 MSTORE PUSH1 0x20 ADD DUP1 PUSH2 0x2BC1 PUSH1 0x28 SWAP2 CODECOPY PUSH1 0x40 ADD SWAP2 POP POP PUSH1 0x40 MLOAD DUP1 SWAP2 SUB SWAP1 REVERT JUMPDEST PUSH2 0x1330 DUP11 DUP11 PUSH2 0x28CA JUMP JUMPDEST PUSH2 0x133C DUP7 DUP7 DUP11 DUP11 PUSH2 0x22E0 JUMP JUMPDEST DUP2 ISZERO PUSH2 0x137E JUMPI PUSH1 0x8 SLOAD PUSH2 0x137A SWAP1 PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP1 DUP3 AND SWAP2 PUSH15 0x10000000000000000000000000000 SWAP1 DIV AND PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST PUSH1 0xB SSTORE JUMPDEST PUSH1 0x40 DUP1 MLOAD DUP6 DUP2 MSTORE PUSH1 0x20 DUP2 ADD DUP6 SWAP1 MSTORE DUP2 MLOAD CALLER SWAP3 PUSH32 0x4C209B5FC8AD50758F13E2E1088BA56A560DFF690A1C6FEF26394F4C03821C4F SWAP3 DUP3 SWAP1 SUB ADD SWAP1 LOG2 POP POP PUSH1 0x1 PUSH1 0xC SSTORE POP SWAP5 SWAP7 SWAP6 POP POP POP POP POP POP JUMP JUMPDEST PUSH1 0x1 PUSH1 0x20 MSTORE PUSH1 0x0 SWAP1 DUP2 MSTORE PUSH1 0x40 SWAP1 KECCAK256 SLOAD DUP2 JUMP JUMPDEST PUSH1 0xB SLOAD DUP2 JUMP JUMPDEST PUSH1 0x4 PUSH1 0x20 MSTORE PUSH1 0x0 SWAP1 DUP2 MSTORE PUSH1 0x40 SWAP1 KECCAK256 SLOAD DUP2 JUMP JUMPDEST PUSH1 0x0 DUP1 PUSH1 0xC SLOAD PUSH1 0x1 EQ PUSH2 0x1469 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x11 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A204C4F434B4544000000000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH1 0x0 PUSH1 0xC DUP2 SWAP1 SSTORE DUP1 PUSH2 0x1479 PUSH2 0xD90 JUMP JUMPDEST POP PUSH1 0x6 SLOAD PUSH1 0x7 SLOAD PUSH1 0x40 DUP1 MLOAD PUSH32 0x70A0823100000000000000000000000000000000000000000000000000000000 DUP2 MSTORE ADDRESS PUSH1 0x4 DUP3 ADD MSTORE SWAP1 MLOAD SWAP5 SWAP7 POP SWAP3 SWAP5 POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP2 DUP3 AND SWAP4 SWAP2 AND SWAP2 PUSH1 0x0 SWAP2 DUP5 SWAP2 PUSH4 0x70A08231 SWAP2 PUSH1 0x24 DUP1 DUP4 ADD SWAP3 PUSH1 0x20 SWAP3 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x14FB JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0x150F JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH1 0x40 MLOAD RETURNDATASIZE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x1525 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD PUSH1 0x40 DUP1 MLOAD PUSH32 0x70A0823100000000000000000000000000000000000000000000000000000000 DUP2 MSTORE ADDRESS PUSH1 0x4 DUP3 ADD MSTORE SWAP1 MLOAD SWAP2 SWAP3 POP PUSH1 0x0 SWAP2 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP6 AND SWAP2 PUSH4 0x70A08231 SWAP2 PUSH1 0x24 DUP1 DUP4 ADD SWAP3 PUSH1 0x20 SWAP3 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x1599 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0x15AD JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH1 0x40 MLOAD RETURNDATASIZE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x15C3 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD ADDRESS PUSH1 0x0 SWAP1 DUP2 MSTORE PUSH1 0x1 PUSH1 0x20 MSTORE PUSH1 0x40 DUP2 KECCAK256 SLOAD SWAP2 SWAP3 POP PUSH2 0x15E2 DUP9 DUP9 PUSH2 0x26EC JUMP JUMPDEST PUSH1 0x0 SLOAD SWAP1 SWAP2 POP DUP1 PUSH2 0x15F9 DUP5 DUP8 PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST DUP2 PUSH2 0x1600 JUMPI INVALID JUMPDEST DIV SWAP11 POP DUP1 PUSH2 0x1614 DUP5 DUP7 PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST DUP2 PUSH2 0x161B JUMPI INVALID JUMPDEST DIV SWAP10 POP PUSH1 0x0 DUP12 GT DUP1 ISZERO PUSH2 0x162E JUMPI POP PUSH1 0x0 DUP11 GT JUMPDEST PUSH2 0x1683 JUMPI PUSH1 0x40 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x4 ADD DUP1 DUP1 PUSH1 0x20 ADD DUP3 DUP2 SUB DUP3 MSTORE PUSH1 0x28 DUP2 MSTORE PUSH1 0x20 ADD DUP1 PUSH2 0x2B99 PUSH1 0x28 SWAP2 CODECOPY PUSH1 0x40 ADD SWAP2 POP POP PUSH1 0x40 MLOAD DUP1 SWAP2 SUB SWAP1 REVERT JUMPDEST PUSH2 0x168D ADDRESS DUP5 PUSH2 0x2992 JUMP JUMPDEST PUSH2 0x1698 DUP8 DUP14 DUP14 PUSH2 0x1FDB JUMP JUMPDEST PUSH2 0x16A3 DUP7 DUP14 DUP13 PUSH2 0x1FDB JUMP JUMPDEST PUSH1 0x40 DUP1 MLOAD PUSH32 0x70A0823100000000000000000000000000000000000000000000000000000000 DUP2 MSTORE ADDRESS PUSH1 0x4 DUP3 ADD MSTORE SWAP1 MLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP10 AND SWAP2 PUSH4 0x70A08231 SWAP2 PUSH1 0x24 DUP1 DUP4 ADD SWAP3 PUSH1 0x20 SWAP3 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x170F JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0x1723 JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH1 0x40 MLOAD RETURNDATASIZE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x1739 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD PUSH1 0x40 DUP1 MLOAD PUSH32 0x70A0823100000000000000000000000000000000000000000000000000000000 DUP2 MSTORE ADDRESS PUSH1 0x4 DUP3 ADD MSTORE SWAP1 MLOAD SWAP2 SWAP7 POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP9 AND SWAP2 PUSH4 0x70A08231 SWAP2 PUSH1 0x24 DUP1 DUP3 ADD SWAP3 PUSH1 0x20 SWAP3 SWAP1 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x17AB JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0x17BF JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH1 0x40 MLOAD RETURNDATASIZE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x17D5 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD SWAP4 POP PUSH2 0x17E5 DUP6 DUP6 DUP12 DUP12 PUSH2 0x22E0 JUMP JUMPDEST DUP2 ISZERO PUSH2 0x1827 JUMPI PUSH1 0x8 SLOAD PUSH2 0x1823 SWAP1 PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP1 DUP3 AND SWAP2 PUSH15 0x10000000000000000000000000000 SWAP1 DIV AND PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST PUSH1 0xB SSTORE JUMPDEST PUSH1 0x40 DUP1 MLOAD DUP13 DUP2 MSTORE PUSH1 0x20 DUP2 ADD DUP13 SWAP1 MSTORE DUP2 MLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP16 AND SWAP3 CALLER SWAP3 PUSH32 0xDCCD412F0B1252819CB1FD330B93224CA42612892BB3F4F789976E6D81936496 SWAP3 SWAP1 DUP2 SWAP1 SUB SWAP1 SWAP2 ADD SWAP1 LOG3 POP POP POP POP POP POP POP POP POP PUSH1 0x1 PUSH1 0xC DUP2 SWAP1 SSTORE POP SWAP2 POP SWAP2 JUMP JUMPDEST PUSH1 0x40 MLOAD DUP1 PUSH1 0x40 ADD PUSH1 0x40 MSTORE DUP1 PUSH1 0x6 DUP2 MSTORE PUSH1 0x20 ADD PUSH32 0x554E492D56320000000000000000000000000000000000000000000000000000 DUP2 MSTORE POP DUP2 JUMP JUMPDEST PUSH1 0x0 PUSH2 0xDF2 CALLER DUP5 DUP5 PUSH2 0x260B JUMP JUMPDEST PUSH2 0x3E8 DUP2 JUMP JUMPDEST PUSH1 0xC SLOAD PUSH1 0x1 EQ PUSH2 0x194F JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x11 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A204C4F434B4544000000000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH1 0x0 PUSH1 0xC SSTORE PUSH1 0x6 SLOAD PUSH1 0x7 SLOAD PUSH1 0x8 SLOAD PUSH1 0x40 DUP1 MLOAD PUSH32 0x70A0823100000000000000000000000000000000000000000000000000000000 DUP2 MSTORE ADDRESS PUSH1 0x4 DUP3 ADD MSTORE SWAP1 MLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP5 DUP6 AND SWAP5 SWAP1 SWAP4 AND SWAP3 PUSH2 0x1A2B SWAP3 DUP6 SWAP3 DUP8 SWAP3 PUSH2 0x1A26 SWAP3 PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND SWAP2 DUP6 SWAP2 PUSH4 0x70A08231 SWAP2 PUSH1 0x24 DUP1 DUP3 ADD SWAP3 PUSH1 0x20 SWAP3 SWAP1 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x19EE JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0x1A02 JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH1 0x40 MLOAD RETURNDATASIZE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x1A18 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD SWAP1 PUSH4 0xFFFFFFFF PUSH2 0x226E AND JUMP JUMPDEST PUSH2 0x1FDB JUMP JUMPDEST PUSH1 0x8 SLOAD PUSH1 0x40 DUP1 MLOAD PUSH32 0x70A0823100000000000000000000000000000000000000000000000000000000 DUP2 MSTORE ADDRESS PUSH1 0x4 DUP3 ADD MSTORE SWAP1 MLOAD PUSH2 0x1ACA SWAP3 DUP5 SWAP3 DUP8 SWAP3 PUSH2 0x1A26 SWAP3 PUSH15 0x10000000000000000000000000000 SWAP1 DIV PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND SWAP2 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP7 AND SWAP2 PUSH4 0x70A08231 SWAP2 PUSH1 0x24 DUP1 DUP3 ADD SWAP3 PUSH1 0x20 SWAP3 SWAP1 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x19EE JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP POP PUSH1 0x1 PUSH1 0xC SSTORE POP JUMP JUMPDEST PUSH1 0x5 SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP2 JUMP JUMPDEST PUSH1 0x7 SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP2 JUMP JUMPDEST TIMESTAMP DUP5 LT ISZERO PUSH2 0x1B7B JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x12 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A20455850495245440000000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH1 0x3 SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP1 DUP10 AND PUSH1 0x0 DUP2 DUP2 MSTORE PUSH1 0x4 PUSH1 0x20 SWAP1 DUP2 MSTORE PUSH1 0x40 DUP1 DUP4 KECCAK256 DUP1 SLOAD PUSH1 0x1 DUP1 DUP3 ADD SWAP1 SWAP3 SSTORE DUP3 MLOAD PUSH32 0x6E71EDAE12B1B97F4D1F60370FEF10105FA2FAAE0126114A169C64845D6126C9 DUP2 DUP7 ADD MSTORE DUP1 DUP5 ADD SWAP7 SWAP1 SWAP7 MSTORE SWAP6 DUP14 AND PUSH1 0x60 DUP7 ADD MSTORE PUSH1 0x80 DUP6 ADD DUP13 SWAP1 MSTORE PUSH1 0xA0 DUP6 ADD SWAP6 SWAP1 SWAP6 MSTORE PUSH1 0xC0 DUP1 DUP6 ADD DUP12 SWAP1 MSTORE DUP2 MLOAD DUP1 DUP7 SUB SWAP1 SWAP2 ADD DUP2 MSTORE PUSH1 0xE0 DUP6 ADD DUP3 MSTORE DUP1 MLOAD SWAP1 DUP4 ADD KECCAK256 PUSH32 0x1901000000000000000000000000000000000000000000000000000000000000 PUSH2 0x100 DUP7 ADD MSTORE PUSH2 0x102 DUP6 ADD SWAP7 SWAP1 SWAP7 MSTORE PUSH2 0x122 DUP1 DUP6 ADD SWAP7 SWAP1 SWAP7 MSTORE DUP1 MLOAD DUP1 DUP6 SUB SWAP1 SWAP7 ADD DUP7 MSTORE PUSH2 0x142 DUP5 ADD DUP1 DUP3 MSTORE DUP7 MLOAD SWAP7 DUP4 ADD SWAP7 SWAP1 SWAP7 KECCAK256 SWAP6 DUP4 SWAP1 MSTORE PUSH2 0x162 DUP5 ADD DUP1 DUP3 MSTORE DUP7 SWAP1 MSTORE PUSH1 0xFF DUP10 AND PUSH2 0x182 DUP6 ADD MSTORE PUSH2 0x1A2 DUP5 ADD DUP9 SWAP1 MSTORE PUSH2 0x1C2 DUP5 ADD DUP8 SWAP1 MSTORE MLOAD SWAP2 SWAP4 SWAP3 PUSH2 0x1E2 DUP1 DUP3 ADD SWAP4 PUSH32 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE0 DUP2 ADD SWAP3 DUP2 SWAP1 SUB SWAP1 SWAP2 ADD SWAP1 DUP6 GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0x1CDC JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP PUSH1 0x40 MLOAD PUSH32 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE0 ADD MLOAD SWAP2 POP POP PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP2 AND ISZERO DUP1 ISZERO SWAP1 PUSH2 0x1D57 JUMPI POP DUP9 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND DUP2 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND EQ JUMPDEST PUSH2 0x1DC2 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x1C PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A20494E56414C49445F5349474E415455524500000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH2 0x1DCD DUP10 DUP10 DUP10 PUSH2 0x259C JUMP JUMPDEST POP POP POP POP POP POP POP POP POP JUMP JUMPDEST PUSH1 0x2 PUSH1 0x20 SWAP1 DUP2 MSTORE PUSH1 0x0 SWAP3 DUP4 MSTORE PUSH1 0x40 DUP1 DUP5 KECCAK256 SWAP1 SWAP2 MSTORE SWAP1 DUP3 MSTORE SWAP1 KECCAK256 SLOAD DUP2 JUMP JUMPDEST PUSH1 0xC SLOAD PUSH1 0x1 EQ PUSH2 0x1E66 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x11 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A204C4F434B4544000000000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH1 0x0 PUSH1 0xC SSTORE PUSH1 0x6 SLOAD PUSH1 0x40 DUP1 MLOAD PUSH32 0x70A0823100000000000000000000000000000000000000000000000000000000 DUP2 MSTORE ADDRESS PUSH1 0x4 DUP3 ADD MSTORE SWAP1 MLOAD PUSH2 0x1FD4 SWAP3 PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND SWAP2 PUSH4 0x70A08231 SWAP2 PUSH1 0x24 DUP1 DUP4 ADD SWAP3 PUSH1 0x20 SWAP3 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x1EDD JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0x1EF1 JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH1 0x40 MLOAD RETURNDATASIZE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x1F07 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD PUSH1 0x7 SLOAD PUSH1 0x40 DUP1 MLOAD PUSH32 0x70A0823100000000000000000000000000000000000000000000000000000000 DUP2 MSTORE ADDRESS PUSH1 0x4 DUP3 ADD MSTORE SWAP1 MLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP1 SWAP3 AND SWAP2 PUSH4 0x70A08231 SWAP2 PUSH1 0x24 DUP1 DUP3 ADD SWAP3 PUSH1 0x20 SWAP3 SWAP1 SWAP2 SWAP1 DUP3 SWAP1 SUB ADD DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x1F7A JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0x1F8E JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH1 0x40 MLOAD RETURNDATASIZE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x1FA4 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD PUSH1 0x8 SLOAD PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP1 DUP3 AND SWAP2 PUSH15 0x10000000000000000000000000000 SWAP1 DIV AND PUSH2 0x22E0 JUMP JUMPDEST PUSH1 0x1 PUSH1 0xC SSTORE JUMP JUMPDEST PUSH1 0x40 DUP1 MLOAD DUP1 DUP3 ADD DUP3 MSTORE PUSH1 0x19 DUP2 MSTORE PUSH32 0x7472616E7366657228616464726573732C75696E743235362900000000000000 PUSH1 0x20 SWAP2 DUP3 ADD MSTORE DUP2 MLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP6 DUP2 AND PUSH1 0x24 DUP4 ADD MSTORE PUSH1 0x44 DUP1 DUP4 ADD DUP7 SWAP1 MSTORE DUP5 MLOAD DUP1 DUP5 SUB SWAP1 SWAP2 ADD DUP2 MSTORE PUSH1 0x64 SWAP1 SWAP3 ADD DUP5 MSTORE SWAP2 DUP2 ADD DUP1 MLOAD PUSH28 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH32 0xA9059CBB00000000000000000000000000000000000000000000000000000000 OR DUP2 MSTORE SWAP3 MLOAD DUP2 MLOAD PUSH1 0x0 SWAP5 PUSH1 0x60 SWAP5 DUP10 AND SWAP4 SWAP3 SWAP2 DUP3 SWAP2 SWAP1 DUP1 DUP4 DUP4 JUMPDEST PUSH1 0x20 DUP4 LT PUSH2 0x20E1 JUMPI DUP1 MLOAD DUP3 MSTORE PUSH32 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE0 SWAP1 SWAP3 ADD SWAP2 PUSH1 0x20 SWAP2 DUP3 ADD SWAP2 ADD PUSH2 0x20A4 JUMP JUMPDEST PUSH1 0x1 DUP4 PUSH1 0x20 SUB PUSH2 0x100 EXP SUB DUP1 NOT DUP3 MLOAD AND DUP2 DUP5 MLOAD AND DUP1 DUP3 OR DUP6 MSTORE POP POP POP POP POP POP SWAP1 POP ADD SWAP2 POP POP PUSH1 0x0 PUSH1 0x40 MLOAD DUP1 DUP4 SUB DUP2 PUSH1 0x0 DUP7 GAS CALL SWAP2 POP POP RETURNDATASIZE DUP1 PUSH1 0x0 DUP2 EQ PUSH2 0x2143 JUMPI PUSH1 0x40 MLOAD SWAP2 POP PUSH1 0x1F NOT PUSH1 0x3F RETURNDATASIZE ADD AND DUP3 ADD PUSH1 0x40 MSTORE RETURNDATASIZE DUP3 MSTORE RETURNDATASIZE PUSH1 0x0 PUSH1 0x20 DUP5 ADD RETURNDATACOPY PUSH2 0x2148 JUMP JUMPDEST PUSH1 0x60 SWAP2 POP JUMPDEST POP SWAP2 POP SWAP2 POP DUP2 DUP1 ISZERO PUSH2 0x2176 JUMPI POP DUP1 MLOAD ISZERO DUP1 PUSH2 0x2176 JUMPI POP DUP1 DUP1 PUSH1 0x20 ADD SWAP1 MLOAD PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x2173 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD JUMPDEST PUSH2 0x21E1 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x1A PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A205452414E534645525F4641494C4544000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST POP POP POP POP POP JUMP JUMPDEST PUSH1 0x0 DUP2 ISZERO DUP1 PUSH2 0x2203 JUMPI POP POP DUP1 DUP3 MUL DUP3 DUP3 DUP3 DUP2 PUSH2 0x2200 JUMPI INVALID JUMPDEST DIV EQ JUMPDEST PUSH2 0xDF6 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x14 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x64732D6D6174682D6D756C2D6F766572666C6F77000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST DUP1 DUP3 SUB DUP3 DUP2 GT ISZERO PUSH2 0xDF6 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x15 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x64732D6D6174682D7375622D756E646572666C6F770000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP5 GT DUP1 ISZERO SWAP1 PUSH2 0x230C JUMPI POP PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP4 GT ISZERO JUMPDEST PUSH2 0x2377 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x13 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x556E697377617056323A204F564552464C4F5700000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT JUMPDEST PUSH1 0x8 SLOAD PUSH4 0xFFFFFFFF TIMESTAMP DUP2 AND SWAP2 PUSH29 0x100000000000000000000000000000000000000000000000000000000 SWAP1 DIV DUP2 AND DUP3 SUB SWAP1 DUP2 AND ISZERO DUP1 ISZERO SWAP1 PUSH2 0x23C7 JUMPI POP PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP5 AND ISZERO ISZERO JUMPDEST DUP1 ISZERO PUSH2 0x23E2 JUMPI POP PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP4 AND ISZERO ISZERO JUMPDEST ISZERO PUSH2 0x2492 JUMPI DUP1 PUSH4 0xFFFFFFFF AND PUSH2 0x2425 DUP6 PUSH2 0x23FB DUP7 PUSH2 0x2A57 JUMP JUMPDEST PUSH28 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND SWAP1 PUSH4 0xFFFFFFFF PUSH2 0x2A7B AND JUMP JUMPDEST PUSH1 0x9 DUP1 SLOAD PUSH28 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP3 SWAP1 SWAP3 AND SWAP3 SWAP1 SWAP3 MUL ADD SWAP1 SSTORE PUSH4 0xFFFFFFFF DUP2 AND PUSH2 0x2465 DUP5 PUSH2 0x23FB DUP8 PUSH2 0x2A57 JUMP JUMPDEST PUSH1 0xA DUP1 SLOAD PUSH28 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF SWAP3 SWAP1 SWAP3 AND SWAP3 SWAP1 SWAP3 MUL ADD SWAP1 SSTORE JUMPDEST PUSH1 0x8 DUP1 SLOAD PUSH32 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000000000000000000000000000 AND PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP9 DUP2 AND SWAP2 SWAP1 SWAP2 OR PUSH32 0xFFFFFFFF0000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH15 0x10000000000000000000000000000 DUP9 DUP4 AND DUP2 MUL SWAP2 SWAP1 SWAP2 OR PUSH28 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH29 0x100000000000000000000000000000000000000000000000000000000 PUSH4 0xFFFFFFFF DUP8 AND MUL OR SWAP3 DUP4 SWAP1 SSTORE PUSH1 0x40 DUP1 MLOAD DUP5 DUP5 AND DUP2 MSTORE SWAP2 SWAP1 SWAP4 DIV SWAP1 SWAP2 AND PUSH1 0x20 DUP3 ADD MSTORE DUP2 MLOAD PUSH32 0x1C411E9A96E071241C2F21F7726B17AE89E3CAB4C78BE50E062B03A9FFFBBAD1 SWAP3 SWAP2 DUP2 SWAP1 SUB SWAP1 SWAP2 ADD SWAP1 LOG1 POP POP POP POP POP POP JUMP JUMPDEST PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP1 DUP5 AND PUSH1 0x0 DUP2 DUP2 MSTORE PUSH1 0x2 PUSH1 0x20 SWAP1 DUP2 MSTORE PUSH1 0x40 DUP1 DUP4 KECCAK256 SWAP5 DUP8 AND DUP1 DUP5 MSTORE SWAP5 DUP3 MSTORE SWAP2 DUP3 SWAP1 KECCAK256 DUP6 SWAP1 SSTORE DUP2 MLOAD DUP6 DUP2 MSTORE SWAP2 MLOAD PUSH32 0x8C5BE1E5EBEC7D5BD14F71427D1E84F3DD0314C0F7B2291E5B200AC8C7C3B925 SWAP3 DUP2 SWAP1 SUB SWAP1 SWAP2 ADD SWAP1 LOG3 POP POP POP JUMP JUMPDEST PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP4 AND PUSH1 0x0 SWAP1 DUP2 MSTORE PUSH1 0x1 PUSH1 0x20 MSTORE PUSH1 0x40 SWAP1 KECCAK256 SLOAD PUSH2 0x2641 SWAP1 DUP3 PUSH4 0xFFFFFFFF PUSH2 0x226E AND JUMP JUMPDEST PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP1 DUP6 AND PUSH1 0x0 SWAP1 DUP2 MSTORE PUSH1 0x1 PUSH1 0x20 MSTORE PUSH1 0x40 DUP1 DUP3 KECCAK256 SWAP4 SWAP1 SWAP4 SSTORE SWAP1 DUP5 AND DUP2 MSTORE KECCAK256 SLOAD PUSH2 0x2683 SWAP1 DUP3 PUSH4 0xFFFFFFFF PUSH2 0x2ABC AND JUMP JUMPDEST PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP1 DUP5 AND PUSH1 0x0 DUP2 DUP2 MSTORE PUSH1 0x1 PUSH1 0x20 SWAP1 DUP2 MSTORE PUSH1 0x40 SWAP2 DUP3 SWAP1 KECCAK256 SWAP5 SWAP1 SWAP5 SSTORE DUP1 MLOAD DUP6 DUP2 MSTORE SWAP1 MLOAD SWAP2 SWAP4 SWAP3 DUP8 AND SWAP3 PUSH32 0xDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF SWAP3 SWAP2 DUP3 SWAP1 SUB ADD SWAP1 LOG3 POP POP POP JUMP JUMPDEST PUSH1 0x0 DUP1 PUSH1 0x5 PUSH1 0x0 SWAP1 SLOAD SWAP1 PUSH2 0x100 EXP SWAP1 DIV PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH4 0x17E7E58 PUSH1 0x40 MLOAD DUP2 PUSH4 0xFFFFFFFF AND PUSH1 0xE0 SHL DUP2 MSTORE PUSH1 0x4 ADD PUSH1 0x20 PUSH1 0x40 MLOAD DUP1 DUP4 SUB DUP2 DUP7 DUP1 EXTCODESIZE ISZERO DUP1 ISZERO PUSH2 0x2757 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP GAS STATICCALL ISZERO DUP1 ISZERO PUSH2 0x276B JUMPI RETURNDATASIZE PUSH1 0x0 DUP1 RETURNDATACOPY RETURNDATASIZE PUSH1 0x0 REVERT JUMPDEST POP POP POP POP PUSH1 0x40 MLOAD RETURNDATASIZE PUSH1 0x20 DUP2 LT ISZERO PUSH2 0x2781 JUMPI PUSH1 0x0 DUP1 REVERT JUMPDEST POP MLOAD PUSH1 0xB SLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP3 AND ISZERO DUP1 ISZERO SWAP5 POP SWAP2 SWAP3 POP SWAP1 PUSH2 0x2864 JUMPI DUP1 ISZERO PUSH2 0x285F JUMPI PUSH1 0x0 PUSH2 0x27D8 PUSH2 0x1257 PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP9 DUP2 AND SWAP1 DUP9 AND PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST SWAP1 POP PUSH1 0x0 PUSH2 0x27E5 DUP4 PUSH2 0x2878 JUMP JUMPDEST SWAP1 POP DUP1 DUP3 GT ISZERO PUSH2 0x285C JUMPI PUSH1 0x0 PUSH2 0x2813 PUSH2 0x2804 DUP5 DUP5 PUSH4 0xFFFFFFFF PUSH2 0x226E AND JUMP JUMPDEST PUSH1 0x0 SLOAD SWAP1 PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST SWAP1 POP PUSH1 0x0 PUSH2 0x2838 DUP4 PUSH2 0x282C DUP7 PUSH1 0x5 PUSH4 0xFFFFFFFF PUSH2 0x21E8 AND JUMP JUMPDEST SWAP1 PUSH4 0xFFFFFFFF PUSH2 0x2ABC AND JUMP JUMPDEST SWAP1 POP PUSH1 0x0 DUP2 DUP4 DUP2 PUSH2 0x2845 JUMPI INVALID JUMPDEST DIV SWAP1 POP DUP1 ISZERO PUSH2 0x2858 JUMPI PUSH2 0x2858 DUP8 DUP3 PUSH2 0x28CA JUMP JUMPDEST POP POP POP JUMPDEST POP POP JUMPDEST PUSH2 0x2870 JUMP JUMPDEST DUP1 ISZERO PUSH2 0x2870 JUMPI PUSH1 0x0 PUSH1 0xB SSTORE JUMPDEST POP POP SWAP3 SWAP2 POP POP JUMP JUMPDEST PUSH1 0x0 PUSH1 0x3 DUP3 GT ISZERO PUSH2 0x28BB JUMPI POP DUP1 PUSH1 0x1 PUSH1 0x2 DUP3 DIV ADD JUMPDEST DUP2 DUP2 LT ISZERO PUSH2 0x28B5 JUMPI DUP1 SWAP2 POP PUSH1 0x2 DUP2 DUP3 DUP6 DUP2 PUSH2 0x28A4 JUMPI INVALID JUMPDEST DIV ADD DUP2 PUSH2 0x28AD JUMPI INVALID JUMPDEST DIV SWAP1 POP PUSH2 0x288D JUMP JUMPDEST POP PUSH2 0x28C5 JUMP JUMPDEST DUP2 ISZERO PUSH2 0x28C5 JUMPI POP PUSH1 0x1 JUMPDEST SWAP2 SWAP1 POP JUMP JUMPDEST PUSH1 0x0 SLOAD PUSH2 0x28DD SWAP1 DUP3 PUSH4 0xFFFFFFFF PUSH2 0x2ABC AND JUMP JUMPDEST PUSH1 0x0 SWAP1 DUP2 SSTORE PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP4 AND DUP2 MSTORE PUSH1 0x1 PUSH1 0x20 MSTORE PUSH1 0x40 SWAP1 KECCAK256 SLOAD PUSH2 0x2915 SWAP1 DUP3 PUSH4 0xFFFFFFFF PUSH2 0x2ABC AND JUMP JUMPDEST PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP4 AND PUSH1 0x0 DUP2 DUP2 MSTORE PUSH1 0x1 PUSH1 0x20 SWAP1 DUP2 MSTORE PUSH1 0x40 DUP1 DUP4 KECCAK256 SWAP5 SWAP1 SWAP5 SSTORE DUP4 MLOAD DUP6 DUP2 MSTORE SWAP4 MLOAD SWAP3 SWAP4 SWAP2 SWAP3 PUSH32 0xDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF SWAP3 DUP2 SWAP1 SUB SWAP1 SWAP2 ADD SWAP1 LOG3 POP POP JUMP JUMPDEST PUSH1 0x0 DUP2 DUP4 LT PUSH2 0x2989 JUMPI DUP2 PUSH2 0x298B JUMP JUMPDEST DUP3 JUMPDEST SWAP4 SWAP3 POP POP POP JUMP JUMPDEST PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP3 AND PUSH1 0x0 SWAP1 DUP2 MSTORE PUSH1 0x1 PUSH1 0x20 MSTORE PUSH1 0x40 SWAP1 KECCAK256 SLOAD PUSH2 0x29C8 SWAP1 DUP3 PUSH4 0xFFFFFFFF PUSH2 0x226E AND JUMP JUMPDEST PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP4 AND PUSH1 0x0 SWAP1 DUP2 MSTORE PUSH1 0x1 PUSH1 0x20 MSTORE PUSH1 0x40 DUP2 KECCAK256 SWAP2 SWAP1 SWAP2 SSTORE SLOAD PUSH2 0x2A02 SWAP1 DUP3 PUSH4 0xFFFFFFFF PUSH2 0x226E AND JUMP JUMPDEST PUSH1 0x0 SWAP1 DUP2 SSTORE PUSH1 0x40 DUP1 MLOAD DUP4 DUP2 MSTORE SWAP1 MLOAD PUSH20 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP6 AND SWAP2 PUSH32 0xDDF252AD1BE2C89B69C2B068FC378DAA952BA7F163C4A11628F55A4DF523B3EF SWAP2 SWAP1 DUP2 SWAP1 SUB PUSH1 0x20 ADD SWAP1 LOG3 POP POP JUMP JUMPDEST PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF AND PUSH15 0x10000000000000000000000000000 MUL SWAP1 JUMP JUMPDEST PUSH1 0x0 PUSH14 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP3 AND PUSH28 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF DUP5 AND DUP2 PUSH2 0x2AB4 JUMPI INVALID JUMPDEST DIV SWAP4 SWAP3 POP POP POP JUMP JUMPDEST DUP1 DUP3 ADD DUP3 DUP2 LT ISZERO PUSH2 0xDF6 JUMPI PUSH1 0x40 DUP1 MLOAD PUSH32 0x8C379A000000000000000000000000000000000000000000000000000000000 DUP2 MSTORE PUSH1 0x20 PUSH1 0x4 DUP3 ADD MSTORE PUSH1 0x14 PUSH1 0x24 DUP3 ADD MSTORE PUSH32 0x64732D6D6174682D6164642D6F766572666C6F77000000000000000000000000 PUSH1 0x44 DUP3 ADD MSTORE SWAP1 MLOAD SWAP1 DUP2 SWAP1 SUB PUSH1 0x64 ADD SWAP1 REVERT INVALID SSTORE PUSH15 0x697377617056323A20494E53554646 0x49 NUMBER 0x49 GASLIMIT 0x4E SLOAD 0x5F 0x4F SSTORE SLOAD POP SSTORE SLOAD 0x5F COINBASE 0x4D 0x4F SSTORE 0x4E SLOAD SSTORE PUSH15 0x697377617056323A20494E53554646 0x49 NUMBER 0x49 GASLIMIT 0x4E SLOAD 0x5F 0x49 0x4E POP SSTORE SLOAD 0x5F COINBASE 0x4D 0x4F SSTORE 0x4E SLOAD SSTORE PUSH15 0x697377617056323A20494E53554646 0x49 NUMBER 0x49 GASLIMIT 0x4E SLOAD 0x5F 0x4C 0x49 MLOAD SSTORE 0x49 DIFFICULTY 0x49 SLOAD MSIZE SSTORE PUSH15 0x697377617056323A20494E53554646 0x49 NUMBER 0x49 GASLIMIT 0x4E SLOAD 0x5F 0x4C 0x49 MLOAD SSTORE 0x49 DIFFICULTY 0x49 SLOAD MSIZE 0x5F TIMESTAMP SSTORE MSTORE 0x4E GASLIMIT DIFFICULTY SSTORE PUSH15 0x697377617056323A20494E53554646 0x49 NUMBER 0x49 GASLIMIT 0x4E SLOAD 0x5F 0x4C 0x49 MLOAD SSTORE 0x49 DIFFICULTY 0x49 SLOAD MSIZE 0x5F 0x4D 0x49 0x4E SLOAD GASLIMIT DIFFICULTY LOG2 PUSH6 0x627A7A723158 KECCAK256 PUSH30 0xCA18479E58487606BF70C79E44D8DEE62353C9EE6D01F9A9D70885B8765F 0x22 PUSH5 0x736F6C6343 STOP SDIV LT STOP ORIGIN GASLIMIT 0x49 POP CALLDATACOPY BALANCE ORIGIN DIFFICULTY PUSH16 0x6D61696E28737472696E67206E616D65 0x2C PUSH20 0x7472696E672076657273696F6E2C75696E743235 CALLDATASIZE KECCAK256 PUSH4 0x6861696E 0x49 PUSH5 0x2C61646472 PUSH6 0x737320766572 PUSH10 0x6679696E67436F6E7472 PUSH2 0x6374 0x29 LOG2 PUSH6 0x627A7A723158 KECCAK256 0x27 PUSH1 0xF9 0x2D PUSH32 0xA1DB6F5AA16307BAD65DF4EBCC8550C4B1F03755AB8DFD830C178F64736F6C63 NUMBER STOP SDIV LT STOP ORIGIN ",
      "sourceMap": "102:1764:1:-;;;406:84;8:9:-1;5:2;;;30:1;27;20:12;5:2;406:84:1;;;;;;;;;;;;;;;13:2:-1;8:3;5:11;2:2;;;29:1;26;19:12;2:2;-1:-1;406:84:1;457:11;:26;;-1:-1:-1;;;;;;457:26:1;-1:-1:-1;;;;;457:26:1;;;;;;;;;102:1764;;;-1:-1:-1;102:1764:1;;"
    },
    "deployedBytecode": {
      "linkReferences": {},
      "object": "608060405234801561001057600080fd5b50600436106100885760003560e01c8063a2e74af61161005b578063a2e74af6146100fd578063c9c6539614610132578063e6a439051461016d578063f46901ed146101a857610088565b8063017e7e581461008d578063094b7415146100be5780631e3dd18b146100c6578063574f2ba3146100e3575b600080fd5b6100956101db565b6040805173ffffffffffffffffffffffffffffffffffffffff9092168252519081900360200190f35b6100956101f7565b610095600480360360208110156100dc57600080fd5b5035610213565b6100eb610247565b60408051918252519081900360200190f35b6101306004803603602081101561011357600080fd5b503573ffffffffffffffffffffffffffffffffffffffff1661024d565b005b6100956004803603604081101561014857600080fd5b5073ffffffffffffffffffffffffffffffffffffffff8135811691602001351661031a565b6100956004803603604081101561018357600080fd5b5073ffffffffffffffffffffffffffffffffffffffff8135811691602001351661076d565b610130600480360360208110156101be57600080fd5b503573ffffffffffffffffffffffffffffffffffffffff166107a0565b60005473ffffffffffffffffffffffffffffffffffffffff1681565b60015473ffffffffffffffffffffffffffffffffffffffff1681565b6003818154811061022057fe5b60009182526020909120015473ffffffffffffffffffffffffffffffffffffffff16905081565b60035490565b60015473ffffffffffffffffffffffffffffffffffffffff1633146102d357604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601460248201527f556e697377617056323a20464f5242494444454e000000000000000000000000604482015290519081900360640190fd5b600180547fffffffffffffffffffffffff00000000000000000000000000000000000000001673ffffffffffffffffffffffffffffffffffffffff92909216919091179055565b60008173ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff1614156103b757604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601e60248201527f556e697377617056323a204944454e544943414c5f4144445245535345530000604482015290519081900360640190fd5b6000808373ffffffffffffffffffffffffffffffffffffffff168573ffffffffffffffffffffffffffffffffffffffff16106103f45783856103f7565b84845b909250905073ffffffffffffffffffffffffffffffffffffffff821661047e57604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601760248201527f556e697377617056323a205a45524f5f41444452455353000000000000000000604482015290519081900360640190fd5b73ffffffffffffffffffffffffffffffffffffffff82811660009081526002602090815260408083208585168452909152902054161561051f57604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601660248201527f556e697377617056323a20504149525f45584953545300000000000000000000604482015290519081900360640190fd5b6060604051806020016105319061086d565b6020820181038252601f19601f82011660405250905060008383604051602001808373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1660601b81526014018273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1660601b815260140192505050604051602081830303815290604052805190602001209050808251602084016000f5604080517f485cc95500000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff8781166004830152868116602483015291519297509087169163485cc9559160448082019260009290919082900301818387803b15801561065e57600080fd5b505af1158015610672573d6000803e3d6000fd5b5050505073ffffffffffffffffffffffffffffffffffffffff84811660008181526002602081815260408084208987168086529083528185208054978d167fffffffffffffffffffffffff000000000000000000000000000000000000000098891681179091559383528185208686528352818520805488168517905560038054600181018255958190527fc2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b90950180549097168417909655925483519283529082015281517f0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9929181900390910190a35050505092915050565b600260209081526000928352604080842090915290825290205473ffffffffffffffffffffffffffffffffffffffff1681565b60015473ffffffffffffffffffffffffffffffffffffffff16331461082657604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601460248201527f556e697377617056323a20464f5242494444454e000000000000000000000000604482015290519081900360640190fd5b600080547fffffffffffffffffffffffff00000000000000000000000000000000000000001673ffffffffffffffffffffffffffffffffffffffff92909216919091179055565b612d748061087b8339019056fe60806040526001600c5534801561001557600080fd5b506040514690806052612d228239604080519182900360520182208282018252600a8352692ab734b9bbb0b8102b1960b11b6020938401528151808301835260018152603160f81b908401528151808401919091527fbfcc8ef98ffbf7b6c3fec7bf5185b566b9863e35a9d83acd49ad6824b5969738818301527fc89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6606082015260808101949094523060a0808601919091528151808603909101815260c09094019052825192019190912060035550600580546001600160a01b03191633179055612c1d806101056000396000f3fe608060405234801561001057600080fd5b50600436106101b95760003560e01c80636a627842116100f9578063ba9a7a5611610097578063d21220a711610071578063d21220a7146105da578063d505accf146105e2578063dd62ed3e14610640578063fff6cae91461067b576101b9565b8063ba9a7a5614610597578063bc25cf771461059f578063c45a0155146105d2576101b9565b80637ecebe00116100d35780637ecebe00146104d757806389afcb441461050a57806395d89b4114610556578063a9059cbb1461055e576101b9565b80636a6278421461046957806370a082311461049c5780637464fc3d146104cf576101b9565b806323b872dd116101665780633644e515116101405780633644e51514610416578063485cc9551461041e5780635909c0d5146104595780635a3d549314610461576101b9565b806323b872dd146103ad57806330adf81f146103f0578063313ce567146103f8576101b9565b8063095ea7b311610197578063095ea7b3146103155780630dfe16811461036257806318160ddd14610393576101b9565b8063022c0d9f146101be57806306fdde03146102595780630902f1ac146102d6575b600080fd5b610257600480360360808110156101d457600080fd5b81359160208101359173ffffffffffffffffffffffffffffffffffffffff604083013516919081019060808101606082013564010000000081111561021857600080fd5b82018360208201111561022a57600080fd5b8035906020019184600183028401116401000000008311171561024c57600080fd5b509092509050610683565b005b610261610d57565b6040805160208082528351818301528351919283929083019185019080838360005b8381101561029b578181015183820152602001610283565b50505050905090810190601f1680156102c85780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b6102de610d90565b604080516dffffffffffffffffffffffffffff948516815292909316602083015263ffffffff168183015290519081900360600190f35b61034e6004803603604081101561032b57600080fd5b5073ffffffffffffffffffffffffffffffffffffffff8135169060200135610de5565b604080519115158252519081900360200190f35b61036a610dfc565b6040805173ffffffffffffffffffffffffffffffffffffffff9092168252519081900360200190f35b61039b610e18565b60408051918252519081900360200190f35b61034e600480360360608110156103c357600080fd5b5073ffffffffffffffffffffffffffffffffffffffff813581169160208101359091169060400135610e1e565b61039b610efd565b610400610f21565b6040805160ff9092168252519081900360200190f35b61039b610f26565b6102576004803603604081101561043457600080fd5b5073ffffffffffffffffffffffffffffffffffffffff81358116916020013516610f2c565b61039b611005565b61039b61100b565b61039b6004803603602081101561047f57600080fd5b503573ffffffffffffffffffffffffffffffffffffffff16611011565b61039b600480360360208110156104b257600080fd5b503573ffffffffffffffffffffffffffffffffffffffff166113cb565b61039b6113dd565b61039b600480360360208110156104ed57600080fd5b503573ffffffffffffffffffffffffffffffffffffffff166113e3565b61053d6004803603602081101561052057600080fd5b503573ffffffffffffffffffffffffffffffffffffffff166113f5565b6040805192835260208301919091528051918290030190f35b610261611892565b61034e6004803603604081101561057457600080fd5b5073ffffffffffffffffffffffffffffffffffffffff81351690602001356118cb565b61039b6118d8565b610257600480360360208110156105b557600080fd5b503573ffffffffffffffffffffffffffffffffffffffff166118de565b61036a611ad4565b61036a611af0565b610257600480360360e08110156105f857600080fd5b5073ffffffffffffffffffffffffffffffffffffffff813581169160208101359091169060408101359060608101359060ff6080820135169060a08101359060c00135611b0c565b61039b6004803603604081101561065657600080fd5b5073ffffffffffffffffffffffffffffffffffffffff81358116916020013516611dd8565b610257611df5565b600c546001146106f457604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601160248201527f556e697377617056323a204c4f434b4544000000000000000000000000000000604482015290519081900360640190fd5b6000600c55841515806107075750600084115b61075c576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526025815260200180612b2f6025913960400191505060405180910390fd5b600080610767610d90565b5091509150816dffffffffffffffffffffffffffff168710801561079a5750806dffffffffffffffffffffffffffff1686105b6107ef576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526021815260200180612b786021913960400191505060405180910390fd5b600654600754600091829173ffffffffffffffffffffffffffffffffffffffff91821691908116908916821480159061085457508073ffffffffffffffffffffffffffffffffffffffff168973ffffffffffffffffffffffffffffffffffffffff1614155b6108bf57604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601560248201527f556e697377617056323a20494e56414c49445f544f0000000000000000000000604482015290519081900360640190fd5b8a156108d0576108d0828a8d611fdb565b89156108e1576108e1818a8c611fdb565b86156109c3578873ffffffffffffffffffffffffffffffffffffffff166310d1e85c338d8d8c8c6040518663ffffffff1660e01b8152600401808673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001858152602001848152602001806020018281038252848482818152602001925080828437600081840152601f19601f8201169050808301925050509650505050505050600060405180830381600087803b1580156109aa57600080fd5b505af11580156109be573d6000803e3d6000fd5b505050505b604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905173ffffffffffffffffffffffffffffffffffffffff8416916370a08231916024808301926020929190829003018186803b158015610a2f57600080fd5b505afa158015610a43573d6000803e3d6000fd5b505050506040513d6020811015610a5957600080fd5b5051604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905191955073ffffffffffffffffffffffffffffffffffffffff8316916370a0823191602480820192602092909190829003018186803b158015610acb57600080fd5b505afa158015610adf573d6000803e3d6000fd5b505050506040513d6020811015610af557600080fd5b5051925060009150506dffffffffffffffffffffffffffff85168a90038311610b1f576000610b35565b89856dffffffffffffffffffffffffffff160383035b9050600089856dffffffffffffffffffffffffffff16038311610b59576000610b6f565b89856dffffffffffffffffffffffffffff160383035b90506000821180610b805750600081115b610bd5576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526024815260200180612b546024913960400191505060405180910390fd5b6000610c09610beb84600363ffffffff6121e816565b610bfd876103e863ffffffff6121e816565b9063ffffffff61226e16565b90506000610c21610beb84600363ffffffff6121e816565b9050610c59620f4240610c4d6dffffffffffffffffffffffffffff8b8116908b1663ffffffff6121e816565b9063ffffffff6121e816565b610c69838363ffffffff6121e816565b1015610cd657604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152600c60248201527f556e697377617056323a204b0000000000000000000000000000000000000000604482015290519081900360640190fd5b5050610ce4848488886122e0565b60408051838152602081018390528082018d9052606081018c9052905173ffffffffffffffffffffffffffffffffffffffff8b169133917fd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d8229181900360800190a350506001600c55505050505050505050565b6040518060400160405280600a81526020017f556e69737761702056320000000000000000000000000000000000000000000081525081565b6008546dffffffffffffffffffffffffffff808216926e0100000000000000000000000000008304909116917c0100000000000000000000000000000000000000000000000000000000900463ffffffff1690565b6000610df233848461259c565b5060015b92915050565b60065473ffffffffffffffffffffffffffffffffffffffff1681565b60005481565b73ffffffffffffffffffffffffffffffffffffffff831660009081526002602090815260408083203384529091528120547fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff14610ee85773ffffffffffffffffffffffffffffffffffffffff84166000908152600260209081526040808320338452909152902054610eb6908363ffffffff61226e16565b73ffffffffffffffffffffffffffffffffffffffff851660009081526002602090815260408083203384529091529020555b610ef384848461260b565b5060019392505050565b7f6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c981565b601281565b60035481565b60055473ffffffffffffffffffffffffffffffffffffffff163314610fb257604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601460248201527f556e697377617056323a20464f5242494444454e000000000000000000000000604482015290519081900360640190fd5b6006805473ffffffffffffffffffffffffffffffffffffffff9384167fffffffffffffffffffffffff00000000000000000000000000000000000000009182161790915560078054929093169116179055565b60095481565b600a5481565b6000600c5460011461108457604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601160248201527f556e697377617056323a204c4f434b4544000000000000000000000000000000604482015290519081900360640190fd5b6000600c81905580611094610d90565b50600654604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905193955091935060009273ffffffffffffffffffffffffffffffffffffffff909116916370a08231916024808301926020929190829003018186803b15801561110e57600080fd5b505afa158015611122573d6000803e3d6000fd5b505050506040513d602081101561113857600080fd5b5051600754604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905192935060009273ffffffffffffffffffffffffffffffffffffffff909216916370a0823191602480820192602092909190829003018186803b1580156111b157600080fd5b505afa1580156111c5573d6000803e3d6000fd5b505050506040513d60208110156111db57600080fd5b505190506000611201836dffffffffffffffffffffffffffff871663ffffffff61226e16565b90506000611225836dffffffffffffffffffffffffffff871663ffffffff61226e16565b9050600061123387876126ec565b600054909150806112705761125c6103e8610bfd611257878763ffffffff6121e816565b612878565b985061126b60006103e86128ca565b6112cd565b6112ca6dffffffffffffffffffffffffffff8916611294868463ffffffff6121e816565b8161129b57fe5b046dffffffffffffffffffffffffffff89166112bd868563ffffffff6121e816565b816112c457fe5b0461297a565b98505b60008911611326576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526028815260200180612bc16028913960400191505060405180910390fd5b6113308a8a6128ca565b61133c86868a8a6122e0565b811561137e5760085461137a906dffffffffffffffffffffffffffff808216916e01000000000000000000000000000090041663ffffffff6121e816565b600b555b6040805185815260208101859052815133927f4c209b5fc8ad50758f13e2e1088ba56a560dff690a1c6fef26394f4c03821c4f928290030190a250506001600c5550949695505050505050565b60016020526000908152604090205481565b600b5481565b60046020526000908152604090205481565b600080600c5460011461146957604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601160248201527f556e697377617056323a204c4f434b4544000000000000000000000000000000604482015290519081900360640190fd5b6000600c81905580611479610d90565b50600654600754604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905194965092945073ffffffffffffffffffffffffffffffffffffffff9182169391169160009184916370a08231916024808301926020929190829003018186803b1580156114fb57600080fd5b505afa15801561150f573d6000803e3d6000fd5b505050506040513d602081101561152557600080fd5b5051604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905191925060009173ffffffffffffffffffffffffffffffffffffffff8516916370a08231916024808301926020929190829003018186803b15801561159957600080fd5b505afa1580156115ad573d6000803e3d6000fd5b505050506040513d60208110156115c357600080fd5b5051306000908152600160205260408120549192506115e288886126ec565b600054909150806115f9848763ffffffff6121e816565b8161160057fe5b049a5080611614848663ffffffff6121e816565b8161161b57fe5b04995060008b11801561162e575060008a115b611683576040517f08c379a0000000000000000000000000000000000000000000000000000000008152600401808060200182810382526028815260200180612b996028913960400191505060405180910390fd5b61168d3084612992565b611698878d8d611fdb565b6116a3868d8c611fdb565b604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905173ffffffffffffffffffffffffffffffffffffffff8916916370a08231916024808301926020929190829003018186803b15801561170f57600080fd5b505afa158015611723573d6000803e3d6000fd5b505050506040513d602081101561173957600080fd5b5051604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905191965073ffffffffffffffffffffffffffffffffffffffff8816916370a0823191602480820192602092909190829003018186803b1580156117ab57600080fd5b505afa1580156117bf573d6000803e3d6000fd5b505050506040513d60208110156117d557600080fd5b505193506117e585858b8b6122e0565b811561182757600854611823906dffffffffffffffffffffffffffff808216916e01000000000000000000000000000090041663ffffffff6121e816565b600b555b604080518c8152602081018c9052815173ffffffffffffffffffffffffffffffffffffffff8f169233927fdccd412f0b1252819cb1fd330b93224ca42612892bb3f4f789976e6d81936496929081900390910190a35050505050505050506001600c81905550915091565b6040518060400160405280600681526020017f554e492d5632000000000000000000000000000000000000000000000000000081525081565b6000610df233848461260b565b6103e881565b600c5460011461194f57604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601160248201527f556e697377617056323a204c4f434b4544000000000000000000000000000000604482015290519081900360640190fd5b6000600c55600654600754600854604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905173ffffffffffffffffffffffffffffffffffffffff9485169490931692611a2b9285928792611a26926dffffffffffffffffffffffffffff169185916370a0823191602480820192602092909190829003018186803b1580156119ee57600080fd5b505afa158015611a02573d6000803e3d6000fd5b505050506040513d6020811015611a1857600080fd5b50519063ffffffff61226e16565b611fdb565b600854604080517f70a082310000000000000000000000000000000000000000000000000000000081523060048201529051611aca9284928792611a26926e01000000000000000000000000000090046dffffffffffffffffffffffffffff169173ffffffffffffffffffffffffffffffffffffffff8616916370a0823191602480820192602092909190829003018186803b1580156119ee57600080fd5b50506001600c5550565b60055473ffffffffffffffffffffffffffffffffffffffff1681565b60075473ffffffffffffffffffffffffffffffffffffffff1681565b42841015611b7b57604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601260248201527f556e697377617056323a20455850495245440000000000000000000000000000604482015290519081900360640190fd5b60035473ffffffffffffffffffffffffffffffffffffffff80891660008181526004602090815260408083208054600180820190925582517f6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c98186015280840196909652958d166060860152608085018c905260a085019590955260c08085018b90528151808603909101815260e0850182528051908301207f19010000000000000000000000000000000000000000000000000000000000006101008601526101028501969096526101228085019690965280518085039096018652610142840180825286519683019690962095839052610162840180825286905260ff89166101828501526101a284018890526101c28401879052519193926101e2808201937fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe081019281900390910190855afa158015611cdc573d6000803e3d6000fd5b50506040517fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0015191505073ffffffffffffffffffffffffffffffffffffffff811615801590611d5757508873ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16145b611dc257604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601c60248201527f556e697377617056323a20494e56414c49445f5349474e415455524500000000604482015290519081900360640190fd5b611dcd89898961259c565b505050505050505050565b600260209081526000928352604080842090915290825290205481565b600c54600114611e6657604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601160248201527f556e697377617056323a204c4f434b4544000000000000000000000000000000604482015290519081900360640190fd5b6000600c55600654604080517f70a082310000000000000000000000000000000000000000000000000000000081523060048201529051611fd49273ffffffffffffffffffffffffffffffffffffffff16916370a08231916024808301926020929190829003018186803b158015611edd57600080fd5b505afa158015611ef1573d6000803e3d6000fd5b505050506040513d6020811015611f0757600080fd5b5051600754604080517f70a08231000000000000000000000000000000000000000000000000000000008152306004820152905173ffffffffffffffffffffffffffffffffffffffff909216916370a0823191602480820192602092909190829003018186803b158015611f7a57600080fd5b505afa158015611f8e573d6000803e3d6000fd5b505050506040513d6020811015611fa457600080fd5b50516008546dffffffffffffffffffffffffffff808216916e0100000000000000000000000000009004166122e0565b6001600c55565b604080518082018252601981527f7472616e7366657228616464726573732c75696e743235362900000000000000602091820152815173ffffffffffffffffffffffffffffffffffffffff85811660248301526044808301869052845180840390910181526064909201845291810180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff167fa9059cbb000000000000000000000000000000000000000000000000000000001781529251815160009460609489169392918291908083835b602083106120e157805182527fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe090920191602091820191016120a4565b6001836020036101000a0380198251168184511680821785525050505050509050019150506000604051808303816000865af19150503d8060008114612143576040519150601f19603f3d011682016040523d82523d6000602084013e612148565b606091505b5091509150818015612176575080511580612176575080806020019051602081101561217357600080fd5b50515b6121e157604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f556e697377617056323a205452414e534645525f4641494c4544000000000000604482015290519081900360640190fd5b5050505050565b60008115806122035750508082028282828161220057fe5b04145b610df657604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601460248201527f64732d6d6174682d6d756c2d6f766572666c6f77000000000000000000000000604482015290519081900360640190fd5b80820382811115610df657604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601560248201527f64732d6d6174682d7375622d756e646572666c6f770000000000000000000000604482015290519081900360640190fd5b6dffffffffffffffffffffffffffff841180159061230c57506dffffffffffffffffffffffffffff8311155b61237757604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601360248201527f556e697377617056323a204f564552464c4f5700000000000000000000000000604482015290519081900360640190fd5b60085463ffffffff428116917c0100000000000000000000000000000000000000000000000000000000900481168203908116158015906123c757506dffffffffffffffffffffffffffff841615155b80156123e257506dffffffffffffffffffffffffffff831615155b15612492578063ffffffff16612425856123fb86612a57565b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff169063ffffffff612a7b16565b600980547bffffffffffffffffffffffffffffffffffffffffffffffffffffffff929092169290920201905563ffffffff8116612465846123fb87612a57565b600a80547bffffffffffffffffffffffffffffffffffffffffffffffffffffffff92909216929092020190555b600880547fffffffffffffffffffffffffffffffffffff0000000000000000000000000000166dffffffffffffffffffffffffffff888116919091177fffffffff0000000000000000000000000000ffffffffffffffffffffffffffff166e0100000000000000000000000000008883168102919091177bffffffffffffffffffffffffffffffffffffffffffffffffffffffff167c010000000000000000000000000000000000000000000000000000000063ffffffff871602179283905560408051848416815291909304909116602082015281517f1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1929181900390910190a1505050505050565b73ffffffffffffffffffffffffffffffffffffffff808416600081815260026020908152604080832094871680845294825291829020859055815185815291517f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9259281900390910190a3505050565b73ffffffffffffffffffffffffffffffffffffffff8316600090815260016020526040902054612641908263ffffffff61226e16565b73ffffffffffffffffffffffffffffffffffffffff8085166000908152600160205260408082209390935590841681522054612683908263ffffffff612abc16565b73ffffffffffffffffffffffffffffffffffffffff80841660008181526001602090815260409182902094909455805185815290519193928716927fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef92918290030190a3505050565b600080600560009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1663017e7e586040518163ffffffff1660e01b815260040160206040518083038186803b15801561275757600080fd5b505afa15801561276b573d6000803e3d6000fd5b505050506040513d602081101561278157600080fd5b5051600b5473ffffffffffffffffffffffffffffffffffffffff821615801594509192509061286457801561285f5760006127d86112576dffffffffffffffffffffffffffff88811690881663ffffffff6121e816565b905060006127e583612878565b90508082111561285c576000612813612804848463ffffffff61226e16565b6000549063ffffffff6121e816565b905060006128388361282c86600563ffffffff6121e816565b9063ffffffff612abc16565b9050600081838161284557fe5b04905080156128585761285887826128ca565b5050505b50505b612870565b8015612870576000600b555b505092915050565b600060038211156128bb575080600160028204015b818110156128b5578091506002818285816128a457fe5b0401816128ad57fe5b04905061288d565b506128c5565b81156128c5575060015b919050565b6000546128dd908263ffffffff612abc16565b600090815573ffffffffffffffffffffffffffffffffffffffff8316815260016020526040902054612915908263ffffffff612abc16565b73ffffffffffffffffffffffffffffffffffffffff831660008181526001602090815260408083209490945583518581529351929391927fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef9281900390910190a35050565b6000818310612989578161298b565b825b9392505050565b73ffffffffffffffffffffffffffffffffffffffff82166000908152600160205260409020546129c8908263ffffffff61226e16565b73ffffffffffffffffffffffffffffffffffffffff831660009081526001602052604081209190915554612a02908263ffffffff61226e16565b600090815560408051838152905173ffffffffffffffffffffffffffffffffffffffff8516917fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef919081900360200190a35050565b6dffffffffffffffffffffffffffff166e0100000000000000000000000000000290565b60006dffffffffffffffffffffffffffff82167bffffffffffffffffffffffffffffffffffffffffffffffffffffffff841681612ab457fe5b049392505050565b80820182811015610df657604080517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601460248201527f64732d6d6174682d6164642d6f766572666c6f77000000000000000000000000604482015290519081900360640190fdfe556e697377617056323a20494e53554646494349454e545f4f55545055545f414d4f554e54556e697377617056323a20494e53554646494349454e545f494e5055545f414d4f554e54556e697377617056323a20494e53554646494349454e545f4c4951554944495459556e697377617056323a20494e53554646494349454e545f4c49515549444954595f4255524e4544556e697377617056323a20494e53554646494349454e545f4c49515549444954595f4d494e544544a265627a7a723158207dca18479e58487606bf70c79e44d8dee62353c9ee6d01f9a9d70885b8765f2264736f6c63430005100032454950373132446f6d61696e28737472696e67206e616d652c737472696e672076657273696f6e2c75696e7432353620636861696e49642c6164647265737320766572696679696e67436f6e747261637429a265627a7a723158202760f92d7fa1db6f5aa16307bad65df4ebcc8550c4b1f03755ab8dfd830c178f64736f6c63430005100032"
    }
  }
}"#;
        let artifact: ArtifactBytecode = serde_json::from_str(s).unwrap();
        assert!(artifact.into_bytecode().is_some());

        let artifact: ArtifactBytecode = serde_json::from_str(s).unwrap();
        assert!(artifact.into_deployed_bytecode().is_some());
    }
}
