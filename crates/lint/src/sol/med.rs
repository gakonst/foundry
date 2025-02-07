use std::ops::ControlFlow;

use solar_ast::{visit::Visit, BinOp, BinOpKind, Expr, ExprKind};

use super::DivideBeforeMultiply;

impl<'ast> Visit<'ast> for DivideBeforeMultiply {
    type BreakValue = ();

    fn visit_expr(&mut self, expr: &'ast Expr<'ast>) -> ControlFlow<Self::BreakValue> {
        if let ExprKind::Binary(left_expr, BinOp { kind: BinOpKind::Mul, .. }, _) = &expr.kind {
            if contains_division(left_expr) {
                self.results.push(expr.span);
            }
        }

        self.walk_expr(expr);
        ControlFlow::Continue(())
    }
}

fn contains_division<'ast>(expr: &'ast Expr<'ast>) -> bool {
    match &expr.kind {
        ExprKind::Binary(_, BinOp { kind: BinOpKind::Div, .. }, _) => true,
        ExprKind::Tuple(inner_exprs) => inner_exprs.iter().any(|opt_expr| {
            if let Some(inner_expr) = opt_expr {
                contains_division(inner_expr)
            } else {
                false
            }
        }),
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use solar_ast::{visit::Visit, Arena};
    use solar_interface::{ColorChoice, Session};
    use std::path::Path;

    use super::DivideBeforeMultiply;

    #[test]
    fn test_divide_before_multiply() -> eyre::Result<()> {
        let sess = Session::builder().with_buffer_emitter(ColorChoice::Auto).build();

        let _ = sess.enter(|| -> solar_interface::Result<()> {
            let arena = Arena::new();

            let mut parser = solar_parse::Parser::from_file(
                &sess,
                &arena,
                Path::new("testdata/DivideBeforeMultiply.sol"),
            )?;

            // Parse the file.
            let ast = parser.parse_file().map_err(|e| e.emit())?;

            let mut pattern = DivideBeforeMultiply::default();
            pattern.visit_source_unit(&ast);

            assert_eq!(pattern.results.len(), 6);

            Ok(())
        });

        Ok(())
    }
}
