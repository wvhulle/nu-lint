use crate::ast_walker::VisitContext;
use crate::context::{Fix, LintContext, Replacement, Rule, RuleCategory, Severity};
use crate::rules::best_practices::external_command_helper::{
    BuiltinAlternative, ExternalCommandVisitor,
};
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

    fn check(&self, context: &LintContext) -> Vec<crate::context::Violation> {
        let mut visitor = ExternalCommandVisitor::new(
            self.id(),
            self.severity(),
            Self::get_builtin_alternatives(),
            Some(build_fix),
        );
        context.walk_ast(&mut visitor);
        visitor.into_violations()
    }
}

/// Build a simple fix for system command replacements
fn build_fix(
    cmd_text: &str,
    alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &VisitContext,
) -> Option<Fix> {
    let args_text = context.extract_external_args(args);

    // Build replacement based on command
    let new_text = match cmd_text {
        // Commands that are simple replacements
        "whoami" | "clear" | "exit" | "stat" | "pwd" | "mkdir" | "rm" | "mv" | "cp" | "touch" => {
            if args_text.is_empty() {
                cmd_text.to_string()
            } else {
                format!("{} {}", cmd_text, args_text.join(" "))
            }
        }
        "cd" => {
            if args_text.is_empty() {
                "cd".to_string()
            } else {
                format!("cd {}", args_text[0])
            }
        }
        "sleep" | "kill" => {
            // Pass through arguments
            if args_text.is_empty() {
                cmd_text.to_string()
            } else {
                format!("{} {}", cmd_text, args_text.join(" "))
            }
        }
        "env" | "printenv" => {
            // ^env -> $env or env
            if args_text.is_empty() {
                "$env".to_string()
            } else {
                format!("$env.{}", args_text.join(""))
            }
        }
        "date" => "date now".to_string(),
        "hostname" | "uname" => "sys host".to_string(),
        "man" => {
            if let Some(cmd) = args_text.first() {
                format!("help {}", cmd)
            } else {
                "help commands".to_string()
            }
        }
        "which" | "type" => {
            if let Some(cmd) = args_text.first() {
                format!("which {}", cmd)
            } else {
                "which".to_string()
            }
        }
        "read" => {
            // ^read -> input
            if args_text.contains(&"-s".to_string()) || args_text.contains(&"--silent".to_string())
            {
                "input -s".to_string()
            } else {
                "input".to_string()
            }
        }
        "echo" => {
            if args_text.is_empty() {
                "print".to_string()
            } else {
                format!("print {}", args_text.join(" "))
            }
        }
        "printf" => "print".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_source;
    use crate::test_utils::create_engine_with_stdlib;

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
