use crate::ast_walker::{AstVisitor, VisitContext};
use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use nu_protocol::ast::Expr;
use std::collections::HashMap;

pub struct PreferBuiltinSystemCommands;

impl PreferBuiltinSystemCommands {
    pub fn new() -> Self {
        Self
    }

    /// Map of system commands to their Nushell built-in equivalents
    /// Based on https://www.nushell.sh/book/coming_from_bash.html#command-equivalents
    fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
        let mut map = HashMap::new();

        // System information
        map.insert(
            "env",
            BuiltinAlternative::with_note(
                "$env",
                "Use '$env' to access environment variables or 'env' command to view all",
            ),
        );
        map.insert(
            "printenv",
            BuiltinAlternative::with_note("$env", "Use '$env' to access environment variables"),
        );
        map.insert(
            "date",
            BuiltinAlternative::with_note(
                "date now",
                "Use 'date now' or parse dates with 'into datetime'",
            ),
        );
        map.insert("whoami", BuiltinAlternative::simple("whoami"));
        map.insert(
            "hostname",
            BuiltinAlternative::with_note(
                "sys host",
                "Use 'sys host' to get detailed host information",
            ),
        );
        map.insert(
            "uname",
            BuiltinAlternative::with_note("sys host", "Use 'sys host' to get system information"),
        );
        map.insert("stat", BuiltinAlternative::simple("stat"));

        // Process/system control
        map.insert("sleep", BuiltinAlternative::simple("sleep"));
        map.insert("kill", BuiltinAlternative::simple("kill"));
        map.insert("clear", BuiltinAlternative::simple("clear"));
        map.insert("exit", BuiltinAlternative::simple("exit"));

        // Help and utilities
        map.insert(
            "man",
            BuiltinAlternative::with_note(
                "help",
                "Use 'help <command>' or 'help commands' to list all commands",
            ),
        );
        map.insert(
            "which",
            BuiltinAlternative::with_note("which", "Use 'which' to find command locations"),
        );
        map.insert(
            "type",
            BuiltinAlternative::with_note("which", "Use 'which' to find command locations"),
        );
        map.insert(
            "read",
            BuiltinAlternative::with_note(
                "input",
                "Use 'let var = input' or 'let secret = input -s' for password input",
            ),
        );

        // Basic file system utilities (less common)
        map.insert("pwd", BuiltinAlternative::simple("pwd"));
        map.insert("cd", BuiltinAlternative::simple("cd"));
        map.insert("mkdir", BuiltinAlternative::simple("mkdir"));
        map.insert("rm", BuiltinAlternative::simple("rm"));
        map.insert("mv", BuiltinAlternative::simple("mv"));
        map.insert("cp", BuiltinAlternative::simple("cp"));
        map.insert("touch", BuiltinAlternative::simple("touch"));
        map.insert(
            "echo",
            BuiltinAlternative::with_note("echo or print", "Use 'echo' or 'print' for output"),
        );
        map.insert("printf", BuiltinAlternative::simple("print"));

        map
    }
}

struct BuiltinAlternative {
    command: &'static str,
    note: Option<&'static str>,
}

impl BuiltinAlternative {
    fn simple(command: &'static str) -> Self {
        Self {
            command,
            note: None,
        }
    }

    fn with_note(command: &'static str, note: &'static str) -> Self {
        Self {
            command,
            note: Some(note),
        }
    }
}

impl Default for PreferBuiltinSystemCommands {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferBuiltinSystemCommands {
    fn id(&self) -> &str {
        "BP014"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Prefer Nushell built-in commands over external tools for system operations (env, date, whoami, man, which, cd, pwd, etc.)"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut visitor = ExternalCommandVisitor::new(self);
        context.walk_ast(&mut visitor);
        visitor.violations
    }
}

/// AST visitor that detects external command calls that have builtin alternatives
struct ExternalCommandVisitor<'a> {
    rule: &'a PreferBuiltinSystemCommands,
    violations: Vec<Violation>,
    alternatives: HashMap<&'static str, BuiltinAlternative>,
}

impl<'a> ExternalCommandVisitor<'a> {
    fn new(rule: &'a PreferBuiltinSystemCommands) -> Self {
        Self {
            rule,
            violations: Vec::new(),
            alternatives: PreferBuiltinSystemCommands::get_builtin_alternatives(),
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
    fn test_external_env_detected() {
        let rule = PreferBuiltinSystemCommands::new();
        let source = r#"^env"#;
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
        assert!(!violations.is_empty(), "Should detect external env command");
        assert!(violations[0].message.contains("$env"));
    }

    #[test]
    fn test_external_date_detected() {
        let rule = PreferBuiltinSystemCommands::new();
        let source = r#"^date"#;
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
        assert!(
            !violations.is_empty(),
            "Should detect external date command"
        );
        assert!(violations[0].message.contains("date now"));
    }

    #[test]
    fn test_external_man_detected() {
        let rule = PreferBuiltinSystemCommands::new();
        let source = r#"^man ls"#;
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
        assert!(!violations.is_empty(), "Should detect external man command");
        assert!(violations[0].message.contains("help"));
    }

    #[test]
    fn test_external_read_detected() {
        let rule = PreferBuiltinSystemCommands::new();
        let source = r#"^read -p "Enter: ""#;
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
        assert!(
            !violations.is_empty(),
            "Should detect external read command"
        );
        assert!(violations[0].message.contains("input"));
    }

    #[test]
    fn test_builtin_date_not_flagged() {
        let rule = PreferBuiltinSystemCommands::new();
        let source = r#"date now"#;
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
        assert_eq!(violations.len(), 0, "Should not flag built-in date command");
    }
}
