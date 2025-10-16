use crate::ast_walker::{AstVisitor, VisitContext};
use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use nu_protocol::ast::{Call, Expr};

pub struct PreferParseOverEachSplit;

impl PreferParseOverEachSplit {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreferParseOverEachSplit {
    fn default() -> Self {
        Self::new()
    }
}

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

    /// Check if a call is to the 'each' command
    fn is_each_call(&self, call: &Call, context: &VisitContext) -> bool {
        let decl = context.working_set.get_decl(call.decl_id);
        decl.name() == "each"
    }

    /// Check if a call is to the 'split row' command
    fn is_split_row_call(&self, call: &Call, context: &VisitContext) -> bool {
        let decl = context.working_set.get_decl(call.decl_id);
        let name = decl.name();
        // Check for both "split row" and "split" with "row" as first argument
        name == "split row" || name == "split"
    }

    /// Check if a block contains a 'split row' call
    fn block_contains_split_row(
        &self,
        block_id: nu_protocol::BlockId,
        context: &VisitContext,
    ) -> bool {
        let block = context.get_block(block_id);

        // Walk through all expressions in the block looking for 'split row' calls
        for pipeline in &block.pipelines {
            for element in &pipeline.elements {
                if self.expr_contains_split_row(&element.expr, context) {
                    return true;
                }
            }
        }

        false
    }

    /// Recursively check if an expression contains a 'split row' call
    fn expr_contains_split_row(
        &self,
        expr: &nu_protocol::ast::Expression,
        context: &VisitContext,
    ) -> bool {
        match &expr.expr {
            Expr::Call(call) => {
                if self.is_split_row_call(call, context) {
                    return true;
                }
                // Check arguments
                for arg in &call.arguments {
                    match arg {
                        nu_protocol::ast::Argument::Positional(e) => {
                            if self.expr_contains_split_row(e, context) {
                                return true;
                            }
                        }
                        nu_protocol::ast::Argument::Named(named) => {
                            if let Some(e) = &named.2 {
                                if self.expr_contains_split_row(e, context) {
                                    return true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                false
            }
            Expr::Block(block_id) | Expr::Closure(block_id) | Expr::Subexpression(block_id) => {
                self.block_contains_split_row(*block_id, context)
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
        // Check if this is an 'each' call
        if self.is_each_call(call, context) {
            // Check if any of the arguments is a closure/block containing 'split row'
            for arg in &call.arguments {
                if let nu_protocol::ast::Argument::Positional(expr) = arg {
                    match &expr.expr {
                        Expr::Closure(block_id) | Expr::Block(block_id) => {
                            if self.block_contains_split_row(*block_id, context) {
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
                        _ => {}
                    }
                }
            }
        }

        // Continue walking
        crate::ast_walker::walk_call(self, call, context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_engine_with_stdlib() -> nu_protocol::engine::EngineState {
        let engine_state = nu_cmd_lang::create_default_context();
        nu_command::add_shell_command_context(engine_state)
    }

    #[test]
    fn test_each_split_row_detected() {
        use crate::parser::parse_source;

        let rule = PreferParseOverEachSplit::new();

        // Use a simpler test case without external commands
        let bad_code = r#"["foo bar", "baz qux"] | each { |x| $x | split row " " }"#;
        let engine_state = create_engine_with_stdlib();
        let (block, working_set) = parse_source(&engine_state, bad_code.as_bytes()).unwrap();
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

        let rule = PreferParseOverEachSplit::new();

        let bad_code = r#"$lines | each { split row "," }"#;
        let engine_state = create_engine_with_stdlib();
        let (block, working_set) = parse_source(&engine_state, bad_code.as_bytes()).unwrap();
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

        let rule = PreferParseOverEachSplit::new();

        let good_code = r#"["foo bar", "baz qux"] | parse "{value} {description}""#;
        let engine_state = create_engine_with_stdlib();
        let (block, working_set) = parse_source(&engine_state, good_code.as_bytes()).unwrap();
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

        let rule = PreferParseOverEachSplit::new();

        let good_code = r#"let parts = $line | split row " ""#;
        let engine_state = create_engine_with_stdlib();
        let (block, working_set) = parse_source(&engine_state, good_code.as_bytes()).unwrap();
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

        let rule = PreferParseOverEachSplit::new();

        let good_code = r#"$items | each { |item| $item.name }"#;
        let engine_state = create_engine_with_stdlib();
        let (block, working_set) = parse_source(&engine_state, good_code.as_bytes()).unwrap();
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
