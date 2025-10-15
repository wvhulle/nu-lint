use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};

/// AST-based version of BP009 - Check for too many positional parameters
///
/// This rule inspects function definitions in the AST to accurately count
/// positional parameters, excluding flags.
pub struct MaxPositionalParamsAst {
    max_params: usize,
}

impl MaxPositionalParamsAst {
    pub fn new() -> Self {
        Self { max_params: 2 }
    }
}

impl Default for MaxPositionalParamsAst {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MaxPositionalParamsAst {
    fn id(&self) -> &str {
        "BP009-AST"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &str {
        "Custom functions should have ≤ 2 positional parameters (AST-based)"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        context
            .new_user_functions()
            .filter_map(|(_, decl)| {
                let signature = decl.signature();

                // Count only positional parameters (not flags)
                let positional_count = signature.required_positional.len()
                    + signature.optional_positional.len()
                    + usize::from(signature.rest_positional.is_some());

                // Only create violation if count exceeds threshold
                (positional_count > self.max_params).then(|| Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: format!(
                        "Function '{}' has {} positional parameters, should have ≤ {}",
                        signature.name, positional_count, self.max_params
                    ),
                    span: context.find_declaration_span(&signature.name),
                    suggestion: Some(
                        "Consider using named flags (--flag) for parameters beyond the first 2"
                            .to_string(),
                    ),
                    fix: None,
                    file: None,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_source;
    use nu_protocol::engine::EngineState;

    #[test]
    fn test_too_many_positional_params_ast() {
        let rule = MaxPositionalParamsAst::new();

        let code = r#"
def too-many-params [
    param1: string
    param2: int
    param3: bool
    param4: string
] {
    print $param1
}
"#;

        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, code.as_bytes()).unwrap();
        let context = LintContext {
            source: code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(
            !violations.is_empty(),
            "Should detect too many positional params"
        );
        assert_eq!(violations[0].rule_id, "BP009-AST");
    }

    #[test]
    fn test_acceptable_params_with_flags_ast() {
        let rule = MaxPositionalParamsAst::new();

        let code = r#"
def good-params [
    param1: string
    param2: int
    --flag1: bool
    --flag2: string
] {
    print $param1
}
"#;

        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, code.as_bytes()).unwrap();
        let context = LintContext {
            source: code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert_eq!(
            violations.len(),
            0,
            "Should not flag params when using flags"
        );
    }

    #[test]
    fn test_no_params_ast() {
        let rule = MaxPositionalParamsAst::new();

        let code = r#"
def no-params [] {
    print "hello"
}
"#;

        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, code.as_bytes()).unwrap();
        let context = LintContext {
            source: code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0, "Should not flag no params");
    }
}
