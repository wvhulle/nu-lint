use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};

pub struct CompletionFunctionNaming;

impl CompletionFunctionNaming {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for CompletionFunctionNaming {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for CompletionFunctionNaming {
    fn id(&self) -> &'static str {
        "S014"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Completion functions should use 'nu-complete' prefix for clarity"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Get all custom function definitions
        let functions = context.new_user_functions();

        for (_decl_id, decl) in functions {
            let func_name = &decl.signature().name;

            // Check if the function name suggests it's a completion function
            // but doesn't follow the nu-complete pattern
            let name_lower = func_name.to_lowercase();

            // Heuristics for completion functions:
            // - Contains "complete" or "completion"
            // - Used in completions context (we'd need to check usage)
            if (name_lower.contains("complete") || name_lower.contains("completion"))
                && !func_name.starts_with("nu-complete ")
            {
                let span = context.find_declaration_span(func_name);

                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    message: format!(
                        "Completion function '{func_name}' should use 'nu-complete' prefix"
                    ),
                    span,
                    suggestion: Some(format!(
                        "Consider renaming to: nu-complete {}",
                        func_name
                            .replace("complete", "")
                            .replace("completion", "")
                            .trim()
                    )),
                    fix: None,
                    file: None,
                });
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_source;
    use nu_protocol::engine::EngineState;

    #[test]
    fn test_bad_completion_naming_detected() {
        let rule = CompletionFunctionNaming::new();

        let bad_code = r"def complete-branches [] { ^git branch }";
        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, bad_code.as_bytes());
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
            "Should detect bad completion function naming"
        );
    }

    #[test]
    fn test_good_completion_naming_not_flagged() {
        let rule = CompletionFunctionNaming::new();

        let good_code = r#"def "nu-complete git branches" [] { ^git branch }"#;
        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, good_code.as_bytes());
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
            "Should not flag proper nu-complete naming"
        );
    }

    #[test]
    fn test_non_completion_function_not_flagged() {
        let rule = CompletionFunctionNaming::new();

        let good_code = r#"def process-data [] { print "hello" }"#;
        let engine_state = EngineState::new();
        let (block, working_set) = parse_source(&engine_state, good_code.as_bytes());
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
            "Should not flag non-completion functions"
        );
    }
}
