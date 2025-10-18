use nu_protocol::ast::{Call, Expr};

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
    visitor::{AstVisitor, VisitContext},
};

/// AST visitor that detects 'each' calls containing 'split row'
pub struct EachSplitVisitor {
    violations: Vec<Violation>,
}

impl Default for EachSplitVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl EachSplitVisitor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    fn is_command(call: &Call, context: &VisitContext, name: &str) -> bool {
        context.working_set.get_decl(call.decl_id).name() == name
    }

    fn block_contains_split_row(
        &self,
        block_id: nu_protocol::BlockId,
        context: &VisitContext,
    ) -> bool {
        context
            .get_block(block_id)
            .pipelines
            .iter()
            .flat_map(|pipeline| &pipeline.elements)
            .any(|element| self.expr_contains_split_row(&element.expr, context))
    }

    fn expr_contains_split_row(
        &self,
        expr: &nu_protocol::ast::Expression,
        context: &VisitContext,
    ) -> bool {
        match &expr.expr {
            Expr::Call(call) => {
                let name = context.working_set.get_decl(call.decl_id).name();
                (name == "split row" || name == "split")
                    || call.arguments.iter().any(|arg| match arg {
                        nu_protocol::ast::Argument::Positional(e)
                        | nu_protocol::ast::Argument::Named((_, _, Some(e))) => {
                            self.expr_contains_split_row(e, context)
                        }
                        _ => false,
                    })
            }
            Expr::Block(id) | Expr::Closure(id) | Expr::Subexpression(id) => {
                self.block_contains_split_row(*id, context)
            }
            Expr::FullCellPath(cell_path) => self.expr_contains_split_row(&cell_path.head, context),
            Expr::BinaryOp(left, _, right) => {
                self.expr_contains_split_row(left, context)
                    || self.expr_contains_split_row(right, context)
            }
            Expr::UnaryNot(inner) => self.expr_contains_split_row(inner, context),
            _ => false,
        }
    }

    #[must_use]
    pub fn into_violations(self) -> Vec<Violation> {
        self.violations
    }
}

impl AstVisitor for EachSplitVisitor {
    fn visit_call(&mut self, call: &Call, context: &VisitContext) {
        if Self::is_command(call, context, "each") {
            let has_split = call
                .arguments
                .iter()
                .filter_map(|arg| match arg {
                    nu_protocol::ast::Argument::Positional(expr) => Some(expr),
                    _ => None,
                })
                .any(|expr| match &expr.expr {
                    Expr::Closure(id) | Expr::Block(id) => {
                        self.block_contains_split_row(*id, context)
                    }
                    _ => false,
                });

            if has_split {
                self.violations.push(Violation {
                    rule_id: "prefer_parse_over_each_split".to_string().into(),
                    severity: Severity::Info,
                    message: "Manual splitting with 'each' and 'split row' - consider using \
                              'parse'"
                        .to_string()
                        .into(),
                    span: call.span(),
                    suggestion: Some(
                        "Use 'parse \"{field1} {field2}\"' for structured text extraction instead \
                         of 'each' with 'split row'"
                            .to_string()
                            .into(),
                    ),
                    fix: None,
                    file: None,
                });
            }
        }

        crate::visitor::walk_call(self, call, context);
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut visitor = EachSplitVisitor::new();
    context.walk_ast(&mut visitor);
    visitor.into_violations()
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_parse_over_each_split",
        RuleCategory::Idioms,
        Severity::Info,
        "Prefer 'parse' over 'each' with 'split row' for structured text processing",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
