use crate::ast_walker::{AstVisitor, VisitContext};
use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use nu_protocol::ast::{Call, Expr};

#[derive(Default)]
pub struct PreferParseOverEachSplit;

impl Rule for PreferParseOverEachSplit {
    fn id(&self) -> &str {
        "P003"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Performance
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Prefer 'parse' over 'each' with 'split row' for structured text processing"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut visitor = EachSplitVisitor::new(self);
        context.walk_ast(&mut visitor);
        visitor.violations
    }
}

/// AST visitor that detects 'each' calls containing 'split row'
struct EachSplitVisitor<'a> {
    rule: &'a PreferParseOverEachSplit,
    violations: Vec<Violation>,
}

impl<'a> EachSplitVisitor<'a> {
    fn new(rule: &'a PreferParseOverEachSplit) -> Self {
        Self {
            rule,
            violations: Vec::new(),
        }
    }

    fn is_command(&self, call: &Call, context: &VisitContext, name: &str) -> bool {
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
}

impl<'a> AstVisitor for EachSplitVisitor<'a> {
    fn visit_call(&mut self, call: &Call, context: &VisitContext) {
        if self.is_command(call, context, "each") {
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
                    rule_id: self.rule.id().to_string(),
                    severity: self.rule.severity(),
                    message: "Manual splitting with 'each' and 'split row' - consider using 'parse'".to_string(),
                    span: call.span(),
                    suggestion: Some(
                        "Use 'parse \"{field1} {field2}\"' for structured text extraction instead of 'each' with 'split row'".to_string()
                    ),
                    fix: None,
                    file: None,
                });
            }
        }

        crate::ast_walker::walk_call(self, call, context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::LintEngineBuilder;

    #[test]
    fn test_each_split_row_detected() {
        use crate::parser::parse_source;

        let rule = PreferParseOverEachSplit::default();

        // Use a simpler test case without external commands
        let bad_code = r#"["foo bar", "baz qux"] | each { |x| $x | split row " " }"#;
        let engine_state = LintEngineBuilder::engine_state();
        let (block, working_set) = parse_source(engine_state, bad_code.as_bytes());
        let context = LintContext {
            source: bad_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);

        assert!(
            !violations.is_empty(),
            "Should detect each with split row pattern"
        );
    }

    #[test]
    fn test_each_split_row_inline_detected() {
        use crate::parser::parse_source;

        let rule = PreferParseOverEachSplit::default();

        let bad_code = r#"$lines | each { split row "," }"#;
        let engine_state = LintEngineBuilder::engine_state();
        let (block, working_set) = parse_source(engine_state, bad_code.as_bytes());
        let context = LintContext {
            source: bad_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(
            !violations.is_empty(),
            "Should detect inline split row in each"
        );
    }

    #[test]
    fn test_parse_command_not_flagged() {
        use crate::parser::parse_source;

        let rule = PreferParseOverEachSplit::default();

        let good_code = r#"["foo bar", "baz qux"] | parse "{value} {description}""#;
        let engine_state = LintEngineBuilder::engine_state();
        let (block, working_set) = parse_source(engine_state, good_code.as_bytes());
        let context = LintContext {
            source: good_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag parse command usage"
        );
    }

    #[test]
    fn test_split_row_without_each_not_flagged() {
        use crate::parser::parse_source;

        let rule = PreferParseOverEachSplit::default();

        let good_code = r#"let parts = $line | split row " ""#;
        let engine_state = LintEngineBuilder::engine_state();
        let (block, working_set) = parse_source(engine_state, good_code.as_bytes());
        let context = LintContext {
            source: good_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag split row without each"
        );
    }

    #[test]
    fn test_each_without_split_row_not_flagged() {
        use crate::parser::parse_source;

        let rule = PreferParseOverEachSplit::default();

        let good_code = r"$items | each { |item| $item.name }";
        let engine_state = LintEngineBuilder::engine_state();
        let (block, working_set) = parse_source(engine_state, good_code.as_bytes());
        let context = LintContext {
            source: good_code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag each without split row"
        );
    }
}
