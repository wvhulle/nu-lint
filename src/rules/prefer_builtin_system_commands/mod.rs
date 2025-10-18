use std::collections::HashMap;

use crate::{
    context::LintContext,
    external_command::{BuiltinAlternative, ExternalCommandVisitor},
    lint::{Fix, Replacement, Severity, Violation},
    rule::{Rule, RuleCategory},
    visitor::VisitContext,
};

/// Map of system commands to their Nushell built-in equivalents
/// Based on <https://www.nushell.sh/book/coming_from_bash.html#command-equivalents>
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
            "(sys host).hostname",
            "Use '(sys host).hostname' to get hostname, or 'sys host' for detailed host \
             information. For IP addresses, use 'sys net | get ip'",
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

/// Build a simple fix for system command replacements
fn build_fix(
    cmd_text: &str,
    alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &VisitContext,
) -> Fix {
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
        "hostname" => {
            // hostname without args -> just the hostname
            if args_text.is_empty() {
                "(sys host).hostname".to_string()
            } else {
                // Let the custom suggestion handle hostname -I
                "sys host".to_string()
            }
        }
        "uname" => "sys host".to_string(),
        "man" => {
            if let Some(cmd) = args_text.first() {
                format!("help {cmd}")
            } else {
                "help commands".to_string()
            }
        }
        "which" | "type" => {
            if let Some(cmd) = args_text.first() {
                format!("which {cmd}")
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

    Fix {
        description: format!("Replace '^{}' with '{}'", cmd_text, alternative.command).into(),
        replacements: vec![Replacement {
            span: expr_span,
            new_text: new_text.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
    let mut visitor = ExternalCommandVisitor::new(
        "avoid_external_system_tools",
        Severity::Info,
        get_builtin_alternatives(),
        Some(build_fix),
    );
    context.walk_ast(&mut visitor);
    visitor.into_violations()
}

pub fn rule() -> Rule {
    Rule::new(
        "avoid_external_system_tools",
        RuleCategory::Idioms,
        Severity::Info,
        "Avoid external system tools when Nushell built-ins are available (env, date, whoami, \
         man, which, cd, pwd, etc.)",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
