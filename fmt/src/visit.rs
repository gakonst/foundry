//! Visitor helpers to traverse the [solang](https://github.com/hyperledger-labs/solang) Solidity Parse Tree

use crate::solang_ext::*;
use solang_parser::pt::*;

/// A trait that is invoked while traversing the Solidity Parse Tree.
/// Each method of the [Visitor] trait is a hook that can be potentially overridden.
///
/// Currently the main implementor of this trait is the [`Formatter`](crate::Formatter) struct.
pub trait Visitor {
    type Error: std::error::Error;

    fn visit_source(&mut self, _loc: Loc) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_source_unit(&mut self, _source_unit: &mut SourceUnit) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_doc_comment(&mut self, _doc_comment: &mut DocComment) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_contract(&mut self, _contract: &mut ContractDefinition) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_pragma(
        &mut self,
        _ident: &mut Identifier,
        _str: &mut StringLiteral,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_import_plain(
        &mut self,
        _loc: Loc,
        _import: &mut StringLiteral,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_import_global(
        &mut self,
        _loc: Loc,
        _global: &mut StringLiteral,
        _alias: &mut Identifier,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_import_renames(
        &mut self,
        _loc: Loc,
        _imports: &mut [(Identifier, Option<Identifier>)],
        _from: &mut StringLiteral,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_enum(&mut self, _enum: &mut EnumDefinition) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_assembly(
        &mut self,
        loc: Loc,
        _dialect: &mut Option<StringLiteral>,
        _block: &mut YulBlock,
    ) -> Result<(), Self::Error> {
        self.visit_source(loc)
    }

    fn visit_block(
        &mut self,
        loc: Loc,
        _unchecked: bool,
        _statements: &mut Vec<Statement>,
    ) -> Result<(), Self::Error> {
        self.visit_source(loc)
    }

    fn visit_args(&mut self, loc: Loc, _args: &mut Vec<NamedArgument>) -> Result<(), Self::Error> {
        self.visit_source(loc)
    }

    /// Don't write semicolon at the end because expressions can appear as both
    /// part of other node and a statement in the function body
    fn visit_expr(&mut self, loc: Loc, _expr: &mut Expression) -> Result<(), Self::Error> {
        self.visit_source(loc)
    }

    fn visit_ident(&mut self, loc: Loc, _ident: &mut Identifier) -> Result<(), Self::Error> {
        self.visit_source(loc)
    }

    fn visit_emit(&mut self, loc: Loc, _event: &mut Expression) -> Result<(), Self::Error> {
        self.visit_source(loc)?;
        self.visit_stray_semicolon()?;

        Ok(())
    }

    fn visit_var_definition(&mut self, var: &mut VariableDefinition) -> Result<(), Self::Error> {
        self.visit_source(var.loc)?;
        self.visit_stray_semicolon()?;

        Ok(())
    }

    fn visit_var_definition_stmt(
        &mut self,
        loc: Loc,
        _declaration: &mut VariableDeclaration,
        _expr: &mut Option<Expression>,
        _semicolon: bool,
    ) -> Result<(), Self::Error> {
        self.visit_source(loc)?;
        self.visit_stray_semicolon()?;

        Ok(())
    }

    /// Don't write semicolon at the end because variable declarations can appear in both
    /// struct definition and function body as a statement
    fn visit_var_declaration(
        &mut self,
        var: &mut VariableDeclaration,
        _is_assignment: bool,
    ) -> Result<(), Self::Error> {
        self.visit_source(var.loc)
    }

    fn visit_return(
        &mut self,
        loc: Loc,
        _expr: &mut Option<Expression>,
    ) -> Result<(), Self::Error> {
        self.visit_source(loc)?;
        self.visit_stray_semicolon()?;

        Ok(())
    }

    fn visit_revert(
        &mut self,
        loc: Loc,
        _error: &mut Option<Expression>,
        _args: &mut Vec<Expression>,
    ) -> Result<(), Self::Error> {
        self.visit_source(loc)?;
        self.visit_stray_semicolon()?;

        Ok(())
    }

    fn visit_revert_named_args(
        &mut self,
        loc: Loc,
        _error: &mut Option<Expression>,
        _args: &mut Vec<NamedArgument>,
    ) -> Result<(), Self::Error> {
        self.visit_source(loc)?;
        self.visit_stray_semicolon()?;

        Ok(())
    }

    fn visit_break(&mut self, loc: Loc) -> Result<(), Self::Error> {
        self.visit_source(loc)
    }

    fn visit_continue(&mut self, loc: Loc) -> Result<(), Self::Error> {
        self.visit_source(loc)
    }

    #[allow(clippy::type_complexity)]
    fn visit_try(
        &mut self,
        loc: Loc,
        _expr: &mut Expression,
        _returns: &mut Option<(Vec<(Loc, Option<Parameter>)>, Box<Statement>)>,
        _clauses: &mut Vec<CatchClause>,
    ) -> Result<(), Self::Error> {
        self.visit_source(loc)
    }

    fn visit_if(
        &mut self,
        loc: Loc,
        _cond: &mut Expression,
        _if_branch: &mut Box<Statement>,
        _else_branch: &mut Option<Box<Statement>>,
    ) -> Result<(), Self::Error> {
        self.visit_source(loc)
    }

    fn visit_do_while(
        &mut self,
        loc: Loc,
        _body: &mut Statement,
        _cond: &mut Expression,
    ) -> Result<(), Self::Error> {
        self.visit_source(loc)
    }

    fn visit_while(
        &mut self,
        loc: Loc,
        _cond: &mut Expression,
        _body: &mut Statement,
    ) -> Result<(), Self::Error> {
        self.visit_source(loc)
    }

    fn visit_for(
        &mut self,
        loc: Loc,
        _init: &mut Option<Box<Statement>>,
        _cond: &mut Option<Box<Expression>>,
        _update: &mut Option<Box<Statement>>,
        _body: &mut Option<Box<Statement>>,
    ) -> Result<(), Self::Error> {
        self.visit_source(loc)
    }

    fn visit_function(&mut self, func: &mut FunctionDefinition) -> Result<(), Self::Error> {
        self.visit_source(func.loc())?;
        if func.body.is_none() {
            self.visit_stray_semicolon()?;
        }

        Ok(())
    }

    fn visit_function_attribute(
        &mut self,
        attribute: &mut FunctionAttribute,
    ) -> Result<(), Self::Error> {
        self.visit_source(attribute.loc())?;
        Ok(())
    }

    fn visit_var_attribute(
        &mut self,
        attribute: &mut VariableAttribute,
    ) -> Result<(), Self::Error> {
        self.visit_source(attribute.loc())?;
        Ok(())
    }

    fn visit_base(&mut self, base: &mut Base) -> Result<(), Self::Error> {
        self.visit_source(base.loc)
    }

    fn visit_parameter(&mut self, parameter: &mut Parameter) -> Result<(), Self::Error> {
        self.visit_source(parameter.loc)
    }

    fn visit_struct(&mut self, structure: &mut StructDefinition) -> Result<(), Self::Error> {
        self.visit_source(structure.loc)?;

        Ok(())
    }

    fn visit_event(&mut self, event: &mut EventDefinition) -> Result<(), Self::Error> {
        self.visit_source(event.loc)?;
        self.visit_stray_semicolon()?;

        Ok(())
    }

    fn visit_event_parameter(&mut self, param: &mut EventParameter) -> Result<(), Self::Error> {
        self.visit_source(param.loc)
    }

    fn visit_error(&mut self, error: &mut ErrorDefinition) -> Result<(), Self::Error> {
        self.visit_source(error.loc)?;
        self.visit_stray_semicolon()?;

        Ok(())
    }

    fn visit_error_parameter(&mut self, param: &mut ErrorParameter) -> Result<(), Self::Error> {
        self.visit_source(param.loc)
    }

    fn visit_type_definition(&mut self, def: &mut TypeDefinition) -> Result<(), Self::Error> {
        self.visit_source(def.loc)
    }

    fn visit_stray_semicolon(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_opening_paren(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_closing_paren(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_newline(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_using(&mut self, using: &mut Using) -> Result<(), Self::Error> {
        self.visit_source(using.loc)?;
        self.visit_stray_semicolon()?;

        Ok(())
    }
}

/// All `solang::pt::*` types, such as [Statement](solang::pt::Statement) should implement the
/// [Visitable] trait that accepts a trait [Visitor] implementation, which has various callback
/// handles for Solidity Parse Tree nodes.
///
/// We want to take a `&mut self` to be able to implement some advanced features in the future such
/// as modifying the Parse Tree before formatting it.
pub trait Visitable {
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor;
}

impl<T> Visitable for &mut T
where
    T: Visitable,
{
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor,
    {
        T::visit(self, v)
    }
}

impl<T> Visitable for Option<T>
where
    T: Visitable,
{
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor,
    {
        if let Some(inner) = self.as_mut() {
            inner.visit(v)
        } else {
            Ok(())
        }
    }
}

impl Visitable for SourceUnitPart {
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor,
    {
        match self {
            SourceUnitPart::ContractDefinition(contract) => v.visit_contract(contract),
            SourceUnitPart::PragmaDirective(_, ident, str) => v.visit_pragma(ident, str),
            SourceUnitPart::ImportDirective(import) => import.visit(v),
            SourceUnitPart::EnumDefinition(enumeration) => v.visit_enum(enumeration),
            SourceUnitPart::StructDefinition(structure) => v.visit_struct(structure),
            SourceUnitPart::EventDefinition(event) => v.visit_event(event),
            SourceUnitPart::ErrorDefinition(error) => v.visit_error(error),
            SourceUnitPart::FunctionDefinition(function) => v.visit_function(function),
            SourceUnitPart::VariableDefinition(variable) => v.visit_var_definition(variable),
            SourceUnitPart::TypeDefinition(def) => v.visit_type_definition(def),
            SourceUnitPart::StraySemicolon(_) => v.visit_stray_semicolon(),
            SourceUnitPart::DocComment(doc) => v.visit_doc_comment(doc),
            SourceUnitPart::Using(using) => v.visit_using(using),
        }
    }
}

impl Visitable for Import {
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor,
    {
        match self {
            Import::Plain(import, loc) => v.visit_import_plain(*loc, import),
            Import::GlobalSymbol(global, import_as, loc) => {
                v.visit_import_global(*loc, global, import_as)
            }
            Import::Rename(from, imports, loc) => v.visit_import_renames(*loc, imports, from),
        }
    }
}

impl Visitable for ContractPart {
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor,
    {
        match self {
            ContractPart::StructDefinition(structure) => v.visit_struct(structure),
            ContractPart::EventDefinition(event) => v.visit_event(event),
            ContractPart::ErrorDefinition(error) => v.visit_error(error),
            ContractPart::EnumDefinition(enumeration) => v.visit_enum(enumeration),
            ContractPart::VariableDefinition(variable) => v.visit_var_definition(variable),
            ContractPart::FunctionDefinition(function) => v.visit_function(function),
            ContractPart::TypeDefinition(def) => v.visit_type_definition(def),
            ContractPart::StraySemicolon(_) => v.visit_stray_semicolon(),
            ContractPart::Using(using) => v.visit_using(using),
            ContractPart::DocComment(doc) => v.visit_doc_comment(doc),
        }
    }
}

impl Visitable for Statement {
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor,
    {
        match self {
            Statement::Block { loc, unchecked, statements } => {
                v.visit_block(*loc, *unchecked, statements)
            }
            Statement::Assembly { loc, dialect, block } => v.visit_assembly(*loc, dialect, block),
            Statement::Args(loc, args) => v.visit_args(*loc, args),
            Statement::If(loc, cond, if_branch, else_branch) => {
                v.visit_if(*loc, cond, if_branch, else_branch)
            }
            Statement::While(loc, cond, body) => v.visit_while(*loc, cond, body),
            Statement::Expression(loc, expr) => {
                v.visit_expr(*loc, expr)?;
                v.visit_stray_semicolon()
            }
            Statement::VariableDefinition(loc, declaration, expr) => {
                v.visit_var_definition_stmt(*loc, declaration, expr, true)
            }
            Statement::For(loc, init, cond, update, body) => {
                v.visit_for(*loc, init, cond, update, body)
            }
            Statement::DoWhile(loc, body, cond) => v.visit_do_while(*loc, body, cond),
            Statement::Continue(loc) => v.visit_continue(*loc),
            Statement::Break(loc) => v.visit_break(*loc),
            Statement::Return(loc, expr) => v.visit_return(*loc, expr),
            Statement::Revert(loc, error, args) => v.visit_revert(*loc, error, args),
            Statement::RevertNamedArgs(loc, error, args) => {
                v.visit_revert_named_args(*loc, error, args)
            }
            Statement::Emit(loc, event) => v.visit_emit(*loc, event),
            Statement::Try(loc, expr, returns, clauses) => {
                v.visit_try(*loc, expr, returns, clauses)
            }
            Statement::DocComment(doc) => v.visit_doc_comment(doc),
        }
    }
}

impl Visitable for Loc {
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor,
    {
        v.visit_source(*self)
    }
}

impl Visitable for Expression {
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor,
    {
        v.visit_expr(LineOfCode::loc(self), self)
    }
}

impl Visitable for Identifier {
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor,
    {
        v.visit_ident(self.loc, self)
    }
}

impl Visitable for VariableDeclaration {
    fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
    where
        V: Visitor,
    {
        v.visit_var_declaration(self, false)
    }
}

macro_rules! impl_visitable {
    ($type:ty, $func:ident) => {
        impl Visitable for $type {
            fn visit<V>(&mut self, v: &mut V) -> Result<(), V::Error>
            where
                V: Visitor,
            {
                v.$func(self)
            }
        }
    };
}

impl_visitable!(DocComment, visit_doc_comment);
impl_visitable!(SourceUnit, visit_source_unit);
impl_visitable!(FunctionAttribute, visit_function_attribute);
impl_visitable!(VariableAttribute, visit_var_attribute);
impl_visitable!(Parameter, visit_parameter);
impl_visitable!(Base, visit_base);
impl_visitable!(EventParameter, visit_event_parameter);
impl_visitable!(ErrorParameter, visit_error_parameter);
