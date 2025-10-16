use crate::ast_walker::{AstVisitor, VisitContext};
use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use nu_protocol::ast::Expr;
use std::collections::HashMap;

pub struct PreferBuiltinTextTransforms;

impl PreferBuiltinTextTransforms {
    pub fn new() -> Self {
        Self
    }

    /// Map of text transformation commands to their Nushell built-in equivalents
    /// Based on https://www.nushell.sh/book/coming_from_bash.html#command-equivalents
    fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
        let mut map = HashMap::new();

        // Text transformation
        map.insert(
            "sed",
            BuiltinAlternative::with_note(
                "str replace",
                "Use 'str replace' for find and replace operations",
            ),
        );
        map.insert("awk", BuiltinAlternative::with_note(
            "where, select, or each",
            "Use 'where' for filtering, 'select' for columns, or 'each' for row-by-row processing"
        ));
        map.insert(
            "cut",
            BuiltinAlternative::with_note("select", "Use 'select' to choose specific columns"),
        );
        map.insert(
            "wc",
            BuiltinAlternative::with_note(
                "length or str length",
                "Use 'length' for item count or 'str length' for character count",
            ),
        );
        map.insert(
            "tee",
            BuiltinAlternative::with_note(
                "tee",
                "Use 'tee { save file.txt }' to save while passing through",
            ),
        );
        map.insert(
            "tr",
            BuiltinAlternative::with_note(
                "str replace",
                "Use 'str replace' or 'str downcase'/'str upcase' for case conversion",
            ),
        );
        map.insert(
            "rev",
            BuiltinAlternative::with_note(
                "str reverse or reverse",
                "Use 'str reverse' for string reversal or 'reverse' for list reversal",
            ),
        );

        map
    }
}

struct BuiltinAlternative {
    command: &'static str,
    note: Option<&'static str>,
}

impl BuiltinAlternative {
    fn with_note(command: &'static str, note: &'static str) -> Self {
        Self {
            command,
            note: Some(note),
        }
    }
}

impl Default for PreferBuiltinTextTransforms {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferBuiltinTextTransforms {
    fn id(&self) -> &str {
        "BP013"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Prefer Nushell built-in commands over external tools for text transformation (sed, awk, cut, wc, tr, tee)"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut visitor = ExternalCommandVisitor::new(self);
        context.walk_ast(&mut visitor);
        visitor.violations
    }
}

/// AST visitor that detects external command calls that have builtin alternatives
struct ExternalCommandVisitor<'a> {
    rule: &'a PreferBuiltinTextTransforms,
    violations: Vec<Violation>,
    alternatives: HashMap<&'static str, BuiltinAlternative>,
}

impl<'a> ExternalCommandVisitor<'a> {
    fn new(rule: &'a PreferBuiltinTextTransforms) -> Self {
        Self {
            rule,
            violations: Vec::new(),
            alternatives: PreferBuiltinTextTransforms::get_builtin_alternatives(),
        }
    }
}

impl<'a> AstVisitor for ExternalCommandVisitor<'a> {
    fn visit_expression(&mut self, expr: &nu_protocol::ast::Expression, context: &VisitContext) {
        // Check for external calls
        if let Expr::ExternalCall(head, _args) = &expr.expr {
            // Get the command name from the head expression
            let cmd_text = context.get_span_contents(head.span);

            // Check if this external command has a builtin alternative
            if let Some(alternative) = self.alternatives.get(cmd_text) {
                let message = format!(
                    "Consider using Nushell's built-in '{}' instead of external '^{}'",
                    alternative.command, cmd_text
                );

                let mut suggestion = format!(
                    "Replace '^{}' with built-in command: {}\n\
                     Built-in commands are more portable, faster, and provide better error handling.",
                    cmd_text,
                    alternative.command
                );

                if let Some(note) = alternative.note {
                    suggestion.push_str(&format!("\n\nNote: {}", note));
                }

                self.violations.push(Violation {
                    rule_id: self.rule.id().to_string(),
                    severity: self.rule.severity(),
                    message,
                    span: expr.span,
                    suggestion: Some(suggestion),
                    fix: None,
                    file: None,
                });
            }
        }

        // Continue walking the AST
        crate::ast_walker::walk_expression(self, expr, context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_source;

    fn create_engine_with_stdlib() -> nu_protocol::engine::EngineState {
        let engine_state = nu_cmd_lang::create_default_context();
        nu_command::add_shell_command_context(engine_state)
    }

    #[test]
    fn test_external_sed_detected() {
        let rule = PreferBuiltinTextTransforms::new();
        let source = r#"^sed 's/foo/bar/' file.txt"#;
        let engine_state = create_engine_with_stdlib();
        let (block, working_set) = parse_source(&engine_state, source.as_bytes()).unwrap();
        let context = LintContext {
            source,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(!violations.is_empty(), "Should detect external sed command");
        assert!(violations[0].message.contains("str replace"));
    }

    #[test]
    fn test_external_awk_detected() {
        let rule = PreferBuiltinTextTransforms::new();
        let source = r#"^awk '{print $1}' file.txt"#;
        let engine_state = create_engine_with_stdlib();
        let (block, working_set) = parse_source(&engine_state, source.as_bytes()).unwrap();
        let context = LintContext {
            source,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(!violations.is_empty(), "Should detect external awk command");
        assert!(
            violations[0].message.contains("where") || violations[0].message.contains("select")
        );
    }

    #[test]
    fn test_external_cut_detected() {
        let rule = PreferBuiltinTextTransforms::new();
        let source = r#"^cut -d ',' -f 1 file.csv"#;
        let engine_state = create_engine_with_stdlib();
        let (block, working_set) = parse_source(&engine_state, source.as_bytes()).unwrap();
        let context = LintContext {
            source,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(!violations.is_empty(), "Should detect external cut command");
        assert!(violations[0].message.contains("select"));
    }

    #[test]
    fn test_external_wc_detected() {
        let rule = PreferBuiltinTextTransforms::new();
        let source = r#"^wc -l file.txt"#;
        let engine_state = create_engine_with_stdlib();
        let (block, working_set) = parse_source(&engine_state, source.as_bytes()).unwrap();
        let context = LintContext {
            source,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert!(!violations.is_empty(), "Should detect external wc command");
        assert!(violations[0].message.contains("length"));
    }

    #[test]
    fn test_builtin_str_replace_not_flagged() {
        let rule = PreferBuiltinTextTransforms::new();
        let source = r#""hello" | str replace "h" "H""#;
        let engine_state = create_engine_with_stdlib();
        let (block, working_set) = parse_source(&engine_state, source.as_bytes()).unwrap();
        let context = LintContext {
            source,
            ast: &block,
            engine_state: &engine_state,
            working_set: &working_set,
            file_path: None,
        };

        let violations = rule.check(&context);
        assert_eq!(violations.len(), 0, "Should not flag built-in str replace");
    }
}
