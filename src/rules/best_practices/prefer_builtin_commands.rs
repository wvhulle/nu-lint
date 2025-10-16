use crate::ast_walker::{AstVisitor, VisitContext};
use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use nu_protocol::ast::Expr;
use std::collections::HashMap;

pub struct PreferBuiltinForCommonCommands;

impl PreferBuiltinForCommonCommands {
    pub fn new() -> Self {
        Self
    }

    /// Map of common file and text operations to their Nushell built-in equivalents
    /// Based on https://www.nushell.sh/book/coming_from_bash.html#command-equivalents
    ///
    /// This rule focuses on the most commonly used commands when migrating from bash.
    /// See also: BP013 (text transformation), BP014 (system commands)
    fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
        let mut map = HashMap::new();

        // Common file system operations
        map.insert("ls", BuiltinAlternative::simple("ls"));
        map.insert(
            "cat",
            BuiltinAlternative::with_note(
                "open --raw",
                "Use 'open' to read files as structured data, or 'open --raw' for plain text",
            ),
        );
        map.insert(
            "find",
            BuiltinAlternative::with_note("ls", "Use 'ls **/*.rs' for recursive pattern matching"),
        );

        // Common text processing operations
        map.insert(
            "grep",
            BuiltinAlternative::with_note(
                "where or find",
                "Use 'where $it =~ <pattern>' for filtering or 'find <substring>' for searching",
            ),
        );
        map.insert(
            "head",
            BuiltinAlternative::with_note("first", "Use 'first N' to get the first N items"),
        );
        map.insert(
            "tail",
            BuiltinAlternative::with_note("last", "Use 'last N' to get the last N items"),
        );
        map.insert("sort", BuiltinAlternative::simple("sort or sort-by"));
        map.insert("uniq", BuiltinAlternative::simple("uniq or uniq-by"));

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

impl Default for PreferBuiltinForCommonCommands {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferBuiltinForCommonCommands {
    fn id(&self) -> &str {
        "BP012"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Use Nushell built-ins instead of external commands for common operations like ls, cat, grep, head, tail, sort, uniq, and find"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut visitor = ExternalCommandVisitor::new(self);
        context.walk_ast(&mut visitor);
        visitor.violations
    }
}

/// AST visitor that detects external command calls that have builtin alternatives
struct ExternalCommandVisitor<'a> {
    rule: &'a PreferBuiltinForCommonCommands,
    violations: Vec<Violation>,
    alternatives: HashMap<&'static str, BuiltinAlternative>,
}

impl<'a> ExternalCommandVisitor<'a> {
    fn new(rule: &'a PreferBuiltinForCommonCommands) -> Self {
        Self {
            rule,
            violations: Vec::new(),
            alternatives: PreferBuiltinForCommonCommands::get_builtin_alternatives(),
        }
    }

    /// Build a simple fix for common external command replacements
    fn build_simple_fix(
        &self,
        cmd_text: &str,
        alternative: &BuiltinAlternative,
        args: &[nu_protocol::ast::ExternalArgument],
        expr_span: nu_protocol::Span,
        context: &VisitContext,
    ) -> Option<crate::context::Fix> {
        use crate::context::{Fix, Replacement};

        // Extract arguments
        let args_text = context.extract_external_args(args);

        // Build replacement based on command
        let new_text = match cmd_text {
            "ls" => {
                // ^ls -la -> ls -la (remove ^)
                if args_text.is_empty() {
                    "ls".to_string()
                } else {
                    format!("ls {}", args_text.join(" "))
                }
            }
            "cat" => {
                // ^cat file.txt -> open --raw file.txt
                if let Some(file) = args_text.iter().find(|a| !a.starts_with('-')) {
                    format!("open --raw {}", file)
                } else {
                    alternative.command.to_string()
                }
            }
            "grep" => {
                // Complex - just suggest the alternative
                alternative.command.to_string()
            }
            "head" | "tail" => {
                // ^head -5 -> first 5 or ^tail -5 -> last 5
                let builtin = if cmd_text == "head" { "first" } else { "last" };
                if let Some(num_arg) = args_text.iter().find(|a| a.starts_with('-') && a.len() > 1) {
                    let num = &num_arg[1..];
                    format!("{} {}", builtin, num)
                } else {
                    format!("{} 10", builtin)
                }
            }
            "sort" | "uniq" => {
                // Direct replacement
                cmd_text.to_string()
            }
            "find" => {
                // ^find -> ls (simplified)
                "ls **/*".to_string()
            }
            _ => alternative.command.to_string(),
        };

        Some(Fix {
            description: format!("Replace '^{}' with '{}'", cmd_text, alternative.command),
            replacements: vec![Replacement {
                span: expr_span,
                new_text,
            }],
        })
    }
}

impl<'a> AstVisitor for ExternalCommandVisitor<'a> {
    fn visit_expression(&mut self, expr: &nu_protocol::ast::Expression, context: &VisitContext) {
        // Check for external calls
        if let Expr::ExternalCall(head, args) = &expr.expr {
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

                // Build fix
                let fix = self.build_simple_fix(cmd_text, alternative, args, expr.span, context);

                self.violations.push(Violation {
                    rule_id: self.rule.id().to_string(),
                    severity: self.rule.severity(),
                    message,
                    span: expr.span,
                    suggestion: Some(suggestion),
                    fix,
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
    fn test_external_ls_detected() {
        let rule = PreferBuiltinForCommonCommands::new();
        let source = r#"^ls -la"#;
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
        assert!(!violations.is_empty(), "Should detect external ls command");
        assert!(violations[0].message.contains("ls"));
    }

    #[test]
    fn test_builtin_ls_not_flagged() {
        let rule = PreferBuiltinForCommonCommands::new();
        let source = r#"ls | where type == dir"#;
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
        assert_eq!(violations.len(), 0, "Should not flag built-in ls");
    }

    #[test]
    fn test_external_cat_detected() {
        let rule = PreferBuiltinForCommonCommands::new();
        // Based on Nushell docs: cat <path> -> open --raw <path>
        let source = r#"^cat file.txt"#;
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
        assert!(!violations.is_empty(), "Should detect external cat command");
        assert!(
            violations[0].message.contains("open"),
            "Should suggest 'open' as alternative"
        );
    }

    #[test]
    fn test_external_grep_detected() {
        let rule = PreferBuiltinForCommonCommands::new();
        // Based on Nushell docs: grep <pattern> -> where $it =~ <substring> or find <substring>
        let source = r#"^grep pattern file.txt"#;
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
            "Should detect external grep command"
        );
        assert!(
            violations[0].message.contains("where") || violations[0].message.contains("find"),
            "Should suggest 'where' or 'find' as alternative"
        );
    }

    #[test]
    fn test_external_head_detected() {
        let rule = PreferBuiltinForCommonCommands::new();
        // Based on Nushell docs: command | head -5 -> command | first 5
        let source = r#"^head -5 file.txt"#;
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
            "Should detect external head command"
        );
        assert!(
            violations[0].message.contains("first"),
            "Should suggest 'first' as alternative"
        );
    }

    #[test]
    fn test_external_command_without_builtin_not_flagged() {
        let rule = PreferBuiltinForCommonCommands::new();
        let source = r#"^my-custom-tool --flag"#;
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
        assert_eq!(
            violations.len(),
            0,
            "Should not flag external commands without built-in alternatives"
        );
    }

    #[test]
    fn test_multiple_external_commands_detected() {
        let rule = PreferBuiltinForCommonCommands::new();
        // Pipeline with multiple external commands
        let source = r#"^cat file.txt | ^grep pattern | ^sort"#;
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
        // Should detect all three external commands
        assert!(
            violations.len() >= 3,
            "Should detect multiple external commands, found {}",
            violations.len()
        );
    }

    #[test]
    fn test_external_find_detected() {
        let rule = PreferBuiltinForCommonCommands::new();
        // Based on Nushell docs: find . -name *.rs -> ls **/*.rs
        let source = r#"^find . -name "*.rs""#;
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
            "Should detect external find command"
        );
        assert!(
            violations[0].message.contains("ls"),
            "Should suggest 'ls' as alternative"
        );
    }

    #[test]
    fn test_ls_fix_provided() {
        let rule = PreferBuiltinForCommonCommands::new();
        let source = r#"^ls -la"#;
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
        assert!(!violations.is_empty(), "Should detect external ls");
        assert!(violations[0].fix.is_some(), "Should provide a fix");
        
        let fix = violations[0].fix.as_ref().unwrap();
        // ls keeps arguments as they might be valid for built-in ls
        assert!(fix.replacements[0].new_text.starts_with("ls"), "Should suggest 'ls'");
    }

    #[test]
    fn test_cat_fix_provided() {
        let rule = PreferBuiltinForCommonCommands::new();
        let source = r#"^cat file.txt"#;
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
        assert!(!violations.is_empty(), "Should detect external cat");
        assert!(violations[0].fix.is_some(), "Should provide a fix");
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert!(
            fix.replacements[0].new_text.contains("open") &&
            fix.replacements[0].new_text.contains("file.txt"),
            "Should suggest 'open file.txt'"
        );
    }

    #[test]
    fn test_grep_fix_provided() {
        let rule = PreferBuiltinForCommonCommands::new();
        let source = r#"^grep pattern file.txt"#;
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
        assert!(!violations.is_empty(), "Should detect external grep");
        assert!(violations[0].fix.is_some(), "Should provide a fix");
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert!(
            fix.replacements[0].new_text.contains("find"),
            "Should suggest find command"
        );
    }

    #[test]
    fn test_head_fix_provided() {
        let rule = PreferBuiltinForCommonCommands::new();
        let source = r#"^head -5 file.txt"#;
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
        assert!(!violations.is_empty(), "Should detect external head");
        assert!(violations[0].fix.is_some(), "Should provide a fix");
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert!(
            fix.replacements[0].new_text.contains("first"),
            "Should suggest 'first' command"
        );
    }

    #[test]
    fn test_tail_fix_provided() {
        let rule = PreferBuiltinForCommonCommands::new();
        let source = r#"^tail -3 file.txt"#;
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
        assert!(!violations.is_empty(), "Should detect external tail");
        assert!(violations[0].fix.is_some(), "Should provide a fix");
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert!(
            fix.replacements[0].new_text.contains("last"),
            "Should suggest 'last' command"
        );
    }
}
