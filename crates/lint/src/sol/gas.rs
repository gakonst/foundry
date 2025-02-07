use std::ops::ControlFlow;

use solar_ast::{visit::Visit, Expr, ExprKind};

use super::AsmKeccak256;

impl<'ast> Visit<'ast> for AsmKeccak256 {
    type BreakValue = ();

    fn visit_expr(&mut self, expr: &'ast Expr<'ast>) -> ControlFlow<Self::BreakValue> {
        if let ExprKind::Call(expr, _) = &expr.kind {
            if let ExprKind::Ident(ident) = &expr.kind {
                if ident.name.as_str() == "keccak256" {
                    self.results.push(expr.span);
                }
            }
        }
        self.walk_expr(expr);
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod test {
    use solar_ast::{visit::Visit, Arena};
    use solar_interface::{ColorChoice, Session};
    use std::path::Path;

    use super::AsmKeccak256;

    #[test]
    fn test_keccak256() -> eyre::Result<()> {
        let sess = Session::builder().with_buffer_emitter(ColorChoice::Auto).build();

        let _ = sess.enter(|| -> solar_interface::Result<()> {
            let arena = Arena::new();

            let mut parser =
                solar_parse::Parser::from_file(&sess, &arena, Path::new("testdata/Keccak256.sol"))?;

            // Parse the file.
            let ast = parser.parse_file().map_err(|e| e.emit())?;

            let mut pattern = AsmKeccak256::default();
            pattern.visit_source_unit(&ast);

            assert_eq!(pattern.results.len(), 2);

            Ok(())
        });

        Ok(())
    }
}
