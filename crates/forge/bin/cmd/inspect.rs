use alloy_json_abi::{EventParam, InternalType, JsonAbi, Param};
use alloy_primitives::{hex, keccak256, Address};
use clap::Parser;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, Cell, Table};
use eyre::{Context, Result};
use forge::revm::primitives::Eof;
use foundry_cli::opts::{BuildOpts, CompilerOpts};
use foundry_common::{
    compile::{PathOrContractInfo, ProjectCompiler},
    find_matching_contract_artifact, find_target_path,
    fmt::pretty_eof,
    shell,
};
use foundry_compilers::artifacts::{
    output_selection::{
        BytecodeOutputSelection, ContractOutputSelection, DeployedBytecodeOutputSelection,
        EvmOutputSelection, EwasmOutputSelection,
    },
    CompactBytecode, StorageLayout,
};
use regex::Regex;
use serde_json::{Map, Value};
use std::{collections::BTreeMap, fmt, str::FromStr, sync::LazyLock};

/// CLI arguments for `forge inspect`.
#[derive(Clone, Debug, Parser)]
pub struct InspectArgs {
    /// The identifier of the contract to inspect in the form `(<path>:)?<contractname>`.
    #[arg(value_parser = PathOrContractInfo::from_str)]
    pub contract: PathOrContractInfo,

    /// The contract artifact field to inspect.
    #[arg(value_enum)]
    pub field: ContractArtifactField,

    /// All build arguments are supported
    #[command(flatten)]
    build: BuildOpts,
}

impl InspectArgs {
    pub fn run(self) -> Result<()> {
        let Self { contract, field, build } = self;

        trace!(target: "forge", ?field, ?contract, "running forge inspect");

        // Map field to ContractOutputSelection
        let mut cos = build.compiler.extra_output;
        if !field.is_default() && !cos.iter().any(|selected| field == *selected) {
            cos.push(field.into());
        }

        // Run Optimized?
        let optimized = if field == ContractArtifactField::AssemblyOptimized {
            Some(true)
        } else {
            build.compiler.optimize
        };

        // Build modified Args
        let modified_build_args = BuildOpts {
            compiler: CompilerOpts { extra_output: cos, optimize: optimized, ..build.compiler },
            ..build
        };

        // Build the project
        let project = modified_build_args.project()?;
        let compiler = ProjectCompiler::new().quiet(true);
        let target_path = find_target_path(&project, &contract)?;
        let mut output = compiler.files([target_path.clone()]).compile(&project)?;

        // Find the artifact
        let artifact = find_matching_contract_artifact(&mut output, &target_path, contract.name())?;

        // Match on ContractArtifactFields and pretty-print
        match field {
            ContractArtifactField::Abi => {
                let abi = artifact
                    .abi
                    .as_ref()
                    .ok_or_else(|| eyre::eyre!("Failed to fetch lossless ABI"))?;
                print_abi(abi)?;
            }
            ContractArtifactField::Bytecode => {
                print_json_str(&artifact.bytecode, Some("object"))?;
            }
            ContractArtifactField::DeployedBytecode => {
                print_json_str(&artifact.deployed_bytecode, Some("object"))?;
            }
            ContractArtifactField::Assembly | ContractArtifactField::AssemblyOptimized => {
                print_json_str(&artifact.assembly, None)?;
            }
            ContractArtifactField::LegacyAssembly => {
                print_json_str(&artifact.legacy_assembly, None)?;
            }
            ContractArtifactField::MethodIdentifiers => {
                print_method_identifiers(&artifact.method_identifiers)?;
            }
            ContractArtifactField::GasEstimates => {
                print_json(&artifact.gas_estimates)?;
            }
            ContractArtifactField::StorageLayout => {
                print_storage_layout(artifact.storage_layout.as_ref())?;
            }
            ContractArtifactField::DevDoc => {
                print_json(&artifact.devdoc)?;
            }
            ContractArtifactField::Ir => {
                print_yul(artifact.ir.as_deref())?;
            }
            ContractArtifactField::IrOptimized => {
                print_yul(artifact.ir_optimized.as_deref())?;
            }
            ContractArtifactField::Metadata => {
                print_json(&artifact.metadata)?;
            }
            ContractArtifactField::UserDoc => {
                print_json(&artifact.userdoc)?;
            }
            ContractArtifactField::Ewasm => {
                print_json_str(&artifact.ewasm, None)?;
            }
            ContractArtifactField::Errors => {
                let out = artifact.abi.as_ref().map_or(Map::new(), parse_errors);
                print_errors_events(&out, true)?;
            }
            ContractArtifactField::Events => {
                let out = artifact.abi.as_ref().map_or(Map::new(), parse_events);
                print_errors_events(&out, false)?;
            }
            ContractArtifactField::Eof => {
                print_eof(artifact.deployed_bytecode.and_then(|b| b.bytecode))?;
            }
            ContractArtifactField::EofInit => {
                print_eof(artifact.bytecode)?;
            }
        };

        Ok(())
    }
}

fn parse_errors(abi: &JsonAbi) -> Map<String, Value> {
    let mut out = serde_json::Map::new();
    for er in abi.errors.iter().flat_map(|(_, errors)| errors) {
        let types = get_ty_sig(&er.inputs);
        let sig = format!("{:x}", er.selector());
        let sig_trimmed = &sig[0..8];
        out.insert(format!("{}({})", er.name, types), sig_trimmed.to_string().into());
    }
    out
}

fn parse_events(abi: &JsonAbi) -> Map<String, Value> {
    let mut out = serde_json::Map::new();
    for ev in abi.events.iter().flat_map(|(_, events)| events) {
        let types = parse_event_params(&ev.inputs);
        let topic = hex::encode(keccak256(ev.signature()));
        out.insert(format!("{}({})", ev.name, types), format!("0x{topic}").into());
    }
    out
}

fn parse_event_params(ev_params: &[EventParam]) -> String {
    ev_params
        .iter()
        .map(|p| {
            if let Some(ty) = p.internal_type() {
                return internal_ty(ty)
            }
            p.ty.clone()
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn print_abi(abi: &JsonAbi) -> Result<()> {
    if shell::is_json() {
        return print_json(abi)
    }

    let headers = vec![Cell::new("Type"), Cell::new("Signature"), Cell::new("Selector")];
    print_table(headers, |table| {
        // Print events
        for ev in abi.events.iter().flat_map(|(_, events)| events) {
            let types = parse_event_params(&ev.inputs);
            let selector = ev.selector().to_string();
            table.add_row(["event", &format!("{}({})", ev.name, types), &selector]);
        }

        // Print errors
        for er in abi.errors.iter().flat_map(|(_, errors)| errors) {
            let selector = er.selector().to_string();
            table.add_row([
                "error",
                &format!("{}({})", er.name, get_ty_sig(&er.inputs)),
                &selector,
            ]);
        }

        // Print functions
        for func in abi.functions.iter().flat_map(|(_, f)| f) {
            let selector = func.selector().to_string();
            let state_mut = func.state_mutability.as_json_str();
            let func_sig = if !func.outputs.is_empty() {
                format!(
                    "{}({}) {state_mut} returns ({})",
                    func.name,
                    get_ty_sig(&func.inputs),
                    get_ty_sig(&func.outputs)
                )
            } else {
                format!("{}({}) {state_mut}", func.name, get_ty_sig(&func.inputs))
            };
            table.add_row(["function", &func_sig, &selector]);
        }

        if let Some(constructor) = abi.constructor() {
            let state_mut = constructor.state_mutability.as_json_str();
            table.add_row([
                "constructor",
                &format!("constructor({}) {state_mut}", get_ty_sig(&constructor.inputs)),
                "",
            ]);
        }

        if let Some(fallback) = &abi.fallback {
            let state_mut = fallback.state_mutability.as_json_str();
            table.add_row(["fallback", &format!("fallback() {state_mut}"), ""]);
        }

        if let Some(receive) = &abi.receive {
            let state_mut = receive.state_mutability.as_json_str();
            table.add_row(["receive", &format!("receive() {state_mut}"), ""]);
        }
    })
}

fn get_ty_sig(inputs: &[Param]) -> String {
    inputs
        .iter()
        .map(|p| {
            if let Some(ty) = p.internal_type() {
                return internal_ty(ty);
            }
            p.ty.clone()
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn internal_ty(ty: &InternalType) -> String {
    let contract_ty =
        |c: &Option<String>, ty: &String| c.clone().map_or(ty.clone(), |c| format!("{c}.{ty}"));
    match ty {
        InternalType::AddressPayable(addr) => addr.clone(),
        InternalType::Contract(contract) => contract.clone(),
        InternalType::Enum { contract, ty } => contract_ty(contract, ty),
        InternalType::Struct { contract, ty } => contract_ty(contract, ty),
        InternalType::Other { contract, ty } => contract_ty(contract, ty),
    }
}

pub fn print_storage_layout(storage_layout: Option<&StorageLayout>) -> Result<()> {
    let Some(storage_layout) = storage_layout else {
        eyre::bail!("Could not get storage layout");
    };

    if shell::is_json() {
        return print_json(&storage_layout)
    }

    let headers = vec![
        Cell::new("Name"),
        Cell::new("Type"),
        Cell::new("Slot"),
        Cell::new("Offset"),
        Cell::new("Bytes"),
        Cell::new("Contract"),
    ];

    print_table(headers, |table| {
        for slot in &storage_layout.storage {
            let storage_type = storage_layout.types.get(&slot.storage_type);
            table.add_row([
                slot.label.as_str(),
                storage_type.map_or("?", |t| &t.label),
                &slot.slot,
                &slot.offset.to_string(),
                storage_type.map_or("?", |t| &t.number_of_bytes),
                &slot.contract,
            ]);
        }
    })
}

fn print_method_identifiers(method_identifiers: &Option<BTreeMap<String, String>>) -> Result<()> {
    let Some(method_identifiers) = method_identifiers else {
        eyre::bail!("Could not get method identifiers");
    };

    if shell::is_json() {
        return print_json(method_identifiers)
    }

    let headers = vec![Cell::new("Method"), Cell::new("Identifier")];

    print_table(headers, |table| {
        for (method, identifier) in method_identifiers {
            table.add_row([method, identifier]);
        }
    })
}

fn print_errors_events(map: &Map<String, Value>, is_err: bool) -> Result<()> {
    if shell::is_json() {
        return print_json(map);
    }

    let headers = if is_err {
        vec![Cell::new("Error"), Cell::new("Selector")]
    } else {
        vec![Cell::new("Event"), Cell::new("Topic")]
    };
    print_table(headers, |table| {
        for (method, selector) in map {
            table.add_row([method, selector.as_str().unwrap()]);
        }
    })
}

fn print_table(headers: Vec<Cell>, add_rows: impl FnOnce(&mut Table)) -> Result<()> {
    let mut table = Table::new();
    table.apply_modifier(UTF8_ROUND_CORNERS);
    table.set_header(headers);
    add_rows(&mut table);
    sh_println!("\n{table}\n")?;
    Ok(())
}

/// Contract level output selection
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContractArtifactField {
    Abi,
    Bytecode,
    DeployedBytecode,
    Assembly,
    AssemblyOptimized,
    LegacyAssembly,
    MethodIdentifiers,
    GasEstimates,
    StorageLayout,
    DevDoc,
    Ir,
    IrOptimized,
    Metadata,
    UserDoc,
    Ewasm,
    Errors,
    Events,
    Eof,
    EofInit,
}

macro_rules! impl_value_enum {
    (enum $name:ident { $($field:ident => $main:literal $(| $alias:literal)*),+ $(,)? }) => {
        impl $name {
            /// All the variants of this enum.
            pub const ALL: &'static [Self] = &[$(Self::$field),+];

            /// Returns the string representation of `self`.
            #[inline]
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(
                        Self::$field => $main,
                    )+
                }
            }

            /// Returns all the aliases of `self`.
            #[inline]
            pub const fn aliases(&self) -> &'static [&'static str] {
                match self {
                    $(
                        Self::$field => &[$($alias),*],
                    )+
                }
            }
        }

        impl ::clap::ValueEnum for $name {
            #[inline]
            fn value_variants<'a>() -> &'a [Self] {
                Self::ALL
            }

            #[inline]
            fn to_possible_value(&self) -> Option<::clap::builder::PossibleValue> {
                Some(::clap::builder::PossibleValue::new(Self::as_str(self)).aliases(Self::aliases(self)))
            }

            #[inline]
            fn from_str(input: &str, ignore_case: bool) -> Result<Self, String> {
                let _ = ignore_case;
                <Self as ::std::str::FromStr>::from_str(input)
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $main $(| $alias)* => Ok(Self::$field),
                    )+
                    _ => Err(format!(concat!("Invalid ", stringify!($name), " value: {}"), s)),
                }
            }
        }
    };
}

impl_value_enum! {
    enum ContractArtifactField {
        Abi               => "abi",
        Bytecode          => "bytecode" | "bytes" | "b",
        DeployedBytecode  => "deployedBytecode" | "deployed_bytecode" | "deployed-bytecode"
                             | "deployed" | "deployedbytecode",
        Assembly          => "assembly" | "asm",
        LegacyAssembly    => "legacyAssembly" | "legacyassembly" | "legacy_assembly",
        AssemblyOptimized => "assemblyOptimized" | "asmOptimized" | "assemblyoptimized"
                             | "assembly_optimized" | "asmopt" | "assembly-optimized"
                             | "asmo" | "asm-optimized" | "asmoptimized" | "asm_optimized",
        MethodIdentifiers => "methodIdentifiers" | "methodidentifiers" | "methods"
                             | "method_identifiers" | "method-identifiers" | "mi",
        GasEstimates      => "gasEstimates" | "gas" | "gas_estimates" | "gas-estimates"
                             | "gasestimates",
        StorageLayout     => "storageLayout" | "storage_layout" | "storage-layout"
                             | "storagelayout" | "storage",
        DevDoc            => "devdoc" | "dev-doc" | "devDoc",
        Ir                => "ir" | "iR" | "IR",
        IrOptimized       => "irOptimized" | "ir-optimized" | "iroptimized" | "iro" | "iropt",
        Metadata          => "metadata" | "meta",
        UserDoc           => "userdoc" | "userDoc" | "user-doc",
        Ewasm             => "ewasm" | "e-wasm",
        Errors            => "errors" | "er",
        Events            => "events" | "ev",
        Eof               => "eof" | "eof-container" | "eof-deployed",
        EofInit           => "eof-init" | "eof-initcode" | "eof-initcontainer",
    }
}

impl From<ContractArtifactField> for ContractOutputSelection {
    fn from(field: ContractArtifactField) -> Self {
        type Caf = ContractArtifactField;
        match field {
            Caf::Abi => Self::Abi,
            Caf::Bytecode => Self::Evm(EvmOutputSelection::ByteCode(BytecodeOutputSelection::All)),
            Caf::DeployedBytecode => Self::Evm(EvmOutputSelection::DeployedByteCode(
                DeployedBytecodeOutputSelection::All,
            )),
            Caf::Assembly | Caf::AssemblyOptimized => Self::Evm(EvmOutputSelection::Assembly),
            Caf::LegacyAssembly => Self::Evm(EvmOutputSelection::LegacyAssembly),
            Caf::MethodIdentifiers => Self::Evm(EvmOutputSelection::MethodIdentifiers),
            Caf::GasEstimates => Self::Evm(EvmOutputSelection::GasEstimates),
            Caf::StorageLayout => Self::StorageLayout,
            Caf::DevDoc => Self::DevDoc,
            Caf::Ir => Self::Ir,
            Caf::IrOptimized => Self::IrOptimized,
            Caf::Metadata => Self::Metadata,
            Caf::UserDoc => Self::UserDoc,
            Caf::Ewasm => Self::Ewasm(EwasmOutputSelection::All),
            Caf::Errors => Self::Abi,
            Caf::Events => Self::Abi,
            Caf::Eof => Self::Evm(EvmOutputSelection::DeployedByteCode(
                DeployedBytecodeOutputSelection::All,
            )),
            Caf::EofInit => Self::Evm(EvmOutputSelection::ByteCode(BytecodeOutputSelection::All)),
        }
    }
}

impl PartialEq<ContractOutputSelection> for ContractArtifactField {
    fn eq(&self, other: &ContractOutputSelection) -> bool {
        type Cos = ContractOutputSelection;
        type Eos = EvmOutputSelection;
        matches!(
            (self, other),
            (Self::Abi | Self::Events, Cos::Abi) |
                (Self::Errors, Cos::Abi) |
                (Self::Bytecode, Cos::Evm(Eos::ByteCode(_))) |
                (Self::DeployedBytecode, Cos::Evm(Eos::DeployedByteCode(_))) |
                (Self::Assembly | Self::AssemblyOptimized, Cos::Evm(Eos::Assembly)) |
                (Self::LegacyAssembly, Cos::Evm(Eos::LegacyAssembly)) |
                (Self::MethodIdentifiers, Cos::Evm(Eos::MethodIdentifiers)) |
                (Self::GasEstimates, Cos::Evm(Eos::GasEstimates)) |
                (Self::StorageLayout, Cos::StorageLayout) |
                (Self::DevDoc, Cos::DevDoc) |
                (Self::Ir, Cos::Ir) |
                (Self::IrOptimized, Cos::IrOptimized) |
                (Self::Metadata, Cos::Metadata) |
                (Self::UserDoc, Cos::UserDoc) |
                (Self::Ewasm, Cos::Ewasm(_)) |
                (Self::Eof, Cos::Evm(Eos::DeployedByteCode(_))) |
                (Self::EofInit, Cos::Evm(Eos::ByteCode(_)))
        )
    }
}

impl fmt::Display for ContractArtifactField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ContractArtifactField {
    /// Returns true if this field is generated by default.
    pub const fn is_default(&self) -> bool {
        matches!(self, Self::Bytecode | Self::DeployedBytecode)
    }
}

fn print_json(obj: &impl serde::Serialize) -> Result<()> {
    sh_println!("{}", serde_json::to_string_pretty(obj)?)?;
    Ok(())
}

fn print_json_str(obj: &impl serde::Serialize, key: Option<&str>) -> Result<()> {
    sh_println!("{}", get_json_str(obj, key)?)?;
    Ok(())
}

fn print_yul(yul: Option<&str>) -> Result<()> {
    let Some(yul) = yul else {
        eyre::bail!("Could not get IR output");
    };

    static YUL_COMMENTS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(///.*\n\s*)|(\s*/\*\*.*\*/)").unwrap());

    sh_println!("{}", YUL_COMMENTS.replace_all(yul, ""))?;

    Ok(())
}

fn get_json_str(obj: &impl serde::Serialize, key: Option<&str>) -> Result<String> {
    let value = serde_json::to_value(obj)?;
    let mut value_ref = &value;
    if let Some(key) = key {
        if let Some(value2) = value.get(key) {
            value_ref = value2;
        }
    }
    let s = match value_ref.as_str() {
        Some(s) => s.to_string(),
        None => format!("{value_ref:#}"),
    };
    Ok(s)
}

/// Pretty-prints bytecode decoded EOF.
fn print_eof(bytecode: Option<CompactBytecode>) -> Result<()> {
    let Some(mut bytecode) = bytecode else { eyre::bail!("No bytecode") };

    // Replace link references with zero address.
    if bytecode.object.is_unlinked() {
        for (file, references) in bytecode.link_references.clone() {
            for (name, _) in references {
                bytecode.link(&file, &name, Address::ZERO);
            }
        }
    }

    let Some(bytecode) = bytecode.object.into_bytes() else {
        eyre::bail!("Failed to link bytecode");
    };

    let eof = Eof::decode(bytecode).wrap_err("Failed to decode EOF")?;

    sh_println!("{}", pretty_eof(&eof)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_output_selection() {
        for &field in ContractArtifactField::ALL {
            let selection: ContractOutputSelection = field.into();
            assert_eq!(field, selection);

            let s = field.as_str();
            assert_eq!(s, field.to_string());
            assert_eq!(s.parse::<ContractArtifactField>().unwrap(), field);
            for alias in field.aliases() {
                assert_eq!(alias.parse::<ContractArtifactField>().unwrap(), field);
            }
        }
    }
}
