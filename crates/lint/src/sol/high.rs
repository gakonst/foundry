use std::ops::ControlFlow;

use solar_ast::{visit::Visit, Expr};

use super::IncorrectShift;

impl<'ast> Visit<'ast> for IncorrectShift {
    type BreakValue = ();
    fn visit_expr(&mut self, expr: &'ast Expr<'ast>) -> ControlFlow<Self::BreakValue> {
        // TODO:
        self.walk_expr(expr);
        ControlFlow::Continue(())
    }
}
