//! Extension traits and modules to the [`solang_parser`] crate.

/// Same as [`solang_parser::pt`], but with the patched `CodeLocation`.
pub mod pt {
    #[doc(no_inline)]
    pub use super::loc::CodeLocationExt as CodeLocation;

    #[doc(no_inline)]
    pub use solang_parser::pt::{
        Annotation, Base, CatchClause, Comment, ContractDefinition, ContractPart, ContractTy,
        EnumDefinition, ErrorDefinition, ErrorParameter, EventDefinition, EventParameter,
        Expression, FunctionAttribute, FunctionDefinition, FunctionTy, HexLiteral, Identifier,
        IdentifierPath, Import, ImportPath, Loc, Mutability, NamedArgument, OptionalCodeLocation,
        Parameter, ParameterList, SourceUnit, SourceUnitPart, Statement, StorageLocation,
        StringLiteral, StructDefinition, Type, TypeDefinition, UserDefinedOperator, Using,
        UsingFunction, UsingList, VariableAttribute, VariableDeclaration, VariableDefinition,
        Visibility, YulBlock, YulExpression, YulFor, YulFunctionCall, YulFunctionDefinition,
        YulStatement, YulSwitch, YulSwitchOptions, YulTypedIdentifier,
    };
}

mod ast_eq;
mod ext;
mod loc;
mod safe_unwrap;

pub use ast_eq::AstEq;
pub use ext::{CommentExt, StatementExt};
pub use loc::CodeLocationExt;
pub use safe_unwrap::SafeUnwrap;
