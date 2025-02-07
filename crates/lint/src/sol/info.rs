use std::ops::ControlFlow;

use regex::Regex;

use solar_ast::{visit::Visit, ItemStruct, VariableDefinition};

use super::{ScreamingSnakeCase, StructPascalCase, VariableMixedCase};

impl<'ast> Visit<'ast> for VariableMixedCase {
    type BreakValue = ();

    fn visit_variable_definition(
        &mut self,
        var: &'ast VariableDefinition<'ast>,
    ) -> ControlFlow<Self::BreakValue> {
        if var.mutability.is_none() {
            if let Some(name) = var.name {
                let name = name.as_str();
                if !is_mixed_case(name) && name.len() > 1 {
                    self.results.push(var.span);
                }
            }
        }

        self.walk_variable_definition(var);
        ControlFlow::Continue(())
    }
}

impl<'ast> Visit<'ast> for ScreamingSnakeCase {
    type BreakValue = ();

    fn visit_variable_definition(
        &mut self,
        var: &'ast VariableDefinition<'ast>,
    ) -> ControlFlow<Self::BreakValue> {
        if let Some(mutability) = var.mutability {
            if mutability.is_constant() || mutability.is_immutable() {
                if let Some(name) = var.name {
                    let name = name.as_str();
                    if !is_screaming_snake_case(name) && name.len() > 1 {
                        self.results.push(var.span);
                    }
                }
            }
        }
        self.walk_variable_definition(var);
        ControlFlow::Continue(())
    }
}

impl<'ast> Visit<'ast> for StructPascalCase {
    type BreakValue = ();

    fn visit_item_struct(
        &mut self,
        strukt: &'ast ItemStruct<'ast>,
    ) -> ControlFlow<Self::BreakValue> {
        let name = strukt.name.as_str();

        if !is_pascal_case(name) && name.len() > 1 {
            self.results.push(strukt.name.span);
        }

        self.walk_item_struct(strukt);
        ControlFlow::Continue(())
    }
}

// Check if a string is mixedCase
pub fn is_mixed_case(s: &str) -> bool {
    let re = Regex::new(r"^[a-z_][a-zA-Z0-9]*$").unwrap();
    re.is_match(s) && s.chars().any(|c| c.is_uppercase())
}

// Check if a string is PascalCase
pub fn is_pascal_case(s: &str) -> bool {
    let re = Regex::new(r"^[A-Z][a-z]+(?:[A-Z][a-z]+)*$").unwrap();
    re.is_match(s)
}

// Check if a string is SCREAMING_SNAKE_CASE
pub fn is_screaming_snake_case(s: &str) -> bool {
    let re = Regex::new(r"^[A-Z_][A-Z0-9_]*$").unwrap();
    re.is_match(s) && s.contains('_')
}

#[cfg(test)]
mod test {
    use solar_ast::{visit::Visit, Arena};
    use solar_interface::{ColorChoice, Session};
    use std::path::Path;

    use crate::sol::StructPascalCase;

    use super::{ScreamingSnakeCase, VariableMixedCase};

    #[test]
    fn test_variable_mixed_case() -> eyre::Result<()> {
        let sess = Session::builder().with_buffer_emitter(ColorChoice::Auto).build();

        let _ = sess.enter(|| -> solar_interface::Result<()> {
            let arena = Arena::new();

            let mut parser = solar_parse::Parser::from_file(
                &sess,
                &arena,
                Path::new("testdata/VariableMixedCase.sol"),
            )?;

            let ast = parser.parse_file().map_err(|e| e.emit())?;

            let mut pattern = VariableMixedCase::default();
            pattern.visit_source_unit(&ast);

            assert_eq!(pattern.results.len(), 6);

            Ok(())
        });

        Ok(())
    }

    #[test]
    fn test_screaming_snake_case() -> eyre::Result<()> {
        let sess = Session::builder().with_buffer_emitter(ColorChoice::Auto).build();

        let _ = sess.enter(|| -> solar_interface::Result<()> {
            let arena = Arena::new();

            let mut parser = solar_parse::Parser::from_file(
                &sess,
                &arena,
                Path::new("testdata/ScreamingSnakeCase.sol"),
            )?;

            let ast = parser.parse_file().map_err(|e| e.emit())?;

            let mut pattern = ScreamingSnakeCase::default();
            pattern.visit_source_unit(&ast);

            assert_eq!(pattern.results.len(), 10);

            Ok(())
        });

        Ok(())
    }

    #[test]
    fn test_struct_pascal_case() -> eyre::Result<()> {
        let sess = Session::builder().with_buffer_emitter(ColorChoice::Auto).build();

        let _ = sess.enter(|| -> solar_interface::Result<()> {
            let arena = Arena::new();

            let mut parser = solar_parse::Parser::from_file(
                &sess,
                &arena,
                Path::new("testdata/StructPascalCase.sol"),
            )?;

            let ast = parser.parse_file().map_err(|e| e.emit())?;

            let mut pattern = StructPascalCase::default();
            pattern.visit_source_unit(&ast);

            assert_eq!(pattern.results.len(), 5);

            Ok(())
        });

        Ok(())
    }
}
