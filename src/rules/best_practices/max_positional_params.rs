use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};

pub struct MaxPositionalParams {
    max_positional: usize,
}

impl MaxPositionalParams {
    #[must_use]
    pub fn new() -> Self {
        Self { max_positional: 2 }
    }
}

impl Default for MaxPositionalParams {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MaxPositionalParams {
    fn id(&self) -> &'static str {
        "BP009"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Custom commands should have ≤ 2 positional parameters"
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
                (positional_count > self.max_positional).then(|| Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: format!(
                        "Command has {} positional parameters, should have ≤ {}",
                        positional_count, self.max_positional
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
    fn test_too_many_positional() {
        let rule = MaxPositionalParams::new();

        let code = r#"
def command [
    a: string
    b: int
    c: bool
    d: string
] {
    print "test"
}
"#;

        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, code.as_bytes());
        let context = LintContext {
            source: code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_acceptable_positional() {
        let rule = MaxPositionalParams::new();

        let code = r#"
def command [
    a: string
    b: int
    --flag-c: bool
    --flag-d: string
] {
    print "test"
}
"#;

        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, code.as_bytes());
        let context = LintContext {
            source: code,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0);
    }
}
