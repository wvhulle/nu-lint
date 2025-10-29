use std::collections::HashMap;

use crate::{
    RuleViolation,
    context::LintContext,
    external_command::{BuiltinAlternative, Fix, extract_external_args},
    lint::{Replacement, Severity},
    rule::{Rule, RuleCategory},
};

/// Map of less common system and text processing commands to their Nushell
/// built-in equivalents
#[allow(clippy::too_many_lines)]
fn get_builtin_alternatives() -> HashMap<&'static str, BuiltinAlternative> {
    let mut map = HashMap::new();

    // System information commands
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
             information",
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

    // Basic file system utilities
    map.insert("pwd", BuiltinAlternative::simple("pwd"));
    map.insert("cd", BuiltinAlternative::simple("cd"));
    map.insert("mkdir", BuiltinAlternative::simple("mkdir"));
    map.insert("rm", BuiltinAlternative::simple("rm"));
    map.insert("mv", BuiltinAlternative::simple("mv"));
    map.insert("cp", BuiltinAlternative::simple("cp"));
    map.insert("touch", BuiltinAlternative::simple("touch"));
    map.insert(
        "echo",
        BuiltinAlternative::with_note("print", "Use 'print' for output"),
    );
    map.insert("printf", BuiltinAlternative::simple("print"));

    // Text transformation commands
    map.insert(
        "sed",
        BuiltinAlternative::with_note(
            "str replace",
            "Use 'str replace' for find and replace operations",
        ),
    );
    map.insert(
        "awk",
        BuiltinAlternative::with_note(
            "where, select, or each",
            "Use 'where' for filtering, 'select' for columns, or 'each' for row-by-row processing",
        ),
    );
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
#[allow(clippy::too_many_lines)]
fn build_fix(
    cmd_text: &str,
    alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    let args_text = extract_external_args(args, context);

    // Build replacement based on command
    let (new_text, description) = match cmd_text {
        // Simple replacements
        "whoami" | "clear" | "exit" | "stat" | "pwd" | "mkdir" | "rm" | "mv" | "cp" | "touch"
        | "sleep" | "kill" => {
            let repl = if args_text.is_empty() {
                cmd_text.to_string()
            } else {
                format!("{} {}", cmd_text, args_text.join(" "))
            };
            (repl, format!("Use Nu's built-in '{}'", alternative.command))
        }
        "cd" => {
            let repl = if args_text.is_empty() {
                "cd".to_string()
            } else {
                format!("cd {}", args_text[0])
            };
            (repl, "Use Nu's built-in 'cd'".to_string())
        }
        "env" | "printenv" => {
            let (repl, desc) = if args_text.is_empty() {
                (
                    "$env".to_string(),
                    "Use '$env' to access all environment variables as a record".to_string(),
                )
            } else {
                (
                    format!("$env.{}", args_text[0]),
                    format!(
                        "Use '$env.{}' to access environment variable directly",
                        args_text[0]
                    ),
                )
            };
            (repl, desc)
        }
        "date" => (
            "date now".to_string(),
            "Use 'date now' which returns a datetime object with timezone support".to_string(),
        ),
        "hostname" => (
            "(sys host).hostname".to_string(),
            "Use '(sys host).hostname' for hostname, or 'sys net | get ip' for IP addresses"
                .to_string(),
        ),
        "uname" => (
            "sys host".to_string(),
            "Use 'sys host' which returns structured system information (name, kernel_version, \
             os_version, etc.)"
                .to_string(),
        ),
        "man" => {
            let repl = if let Some(cmd) = args_text.first() {
                format!("help {cmd}")
            } else {
                "help commands".to_string()
            };
            (
                repl,
                "Use 'help <command>' for command help, or 'help commands' to list all commands"
                    .to_string(),
            )
        }
        "which" | "type" => {
            let repl = if let Some(cmd) = args_text.first() {
                format!("which {cmd}")
            } else {
                "which".to_string()
            };
            (
                repl,
                "Use Nu's built-in 'which' to find command locations".to_string(),
            )
        }
        "read" => {
            let (repl, desc) = if args_text.contains(&"-s".to_string())
                || args_text.contains(&"--silent".to_string())
            {
                (
                    "input -s".to_string(),
                    "Use 'input -s' for secure password input (hidden)".to_string(),
                )
            } else {
                (
                    "input".to_string(),
                    "Use 'input' to read user input".to_string(),
                )
            };
            (repl, desc)
        }
        "echo" | "printf" => (
            if args_text.is_empty() {
                "print".to_string()
            } else {
                format!("print {}", args_text.join(" "))
            },
            "Use 'print' for output (supports structured data)".to_string(),
        ),

        // Text transformation commands
        "sed" => {
            let desc = "Use 'str replace' for find and replace: 'str replace <pattern> \
                        <replacement>' works on structured data"
                .to_string();
            ("str replace".to_string(), desc)
        }
        "awk" => {
            let desc = "Use Nu's data pipeline: 'where' for filtering, 'select' for columns, or \
                        'each' for row processing"
                .to_string();
            ("where | select | each".to_string(), desc)
        }
        "cut" => {
            let desc = "Use 'select' to choose columns from structured data".to_string();
            ("select".to_string(), desc)
        }
        "wc" => {
            let (repl, desc) = if args_text.contains(&"-l".to_string()) {
                (
                    "lines | length".to_string(),
                    "Use 'lines | length' to count lines in a file".to_string(),
                )
            } else {
                (
                    "length".to_string(),
                    "Use 'length' for item count or 'str length' for character count".to_string(),
                )
            };
            (repl, desc)
        }
        "tr" => {
            let desc = "Use 'str replace' for character replacement, or 'str upcase'/'str \
                        downcase' for case conversion"
                .to_string();
            ("str replace".to_string(), desc)
        }
        "tee" => {
            let repl = if let Some(file) = args_text.first() {
                format!("tee {{ save {file} }}")
            } else {
                "tee { save ... }".to_string()
            };
            let desc = "Use 'tee { save file.txt }' to save data while passing it through the \
                        pipeline"
                .to_string();
            (repl, desc)
        }
        "rev" => (
            "str reverse".to_string(),
            "Use 'str reverse' for string reversal or 'reverse' for list reversal".to_string(),
        ),
        _ => (
            alternative.command.to_string(),
            format!("Use Nu's built-in '{}'", alternative.command),
        ),
    };

    Fix {
        description: description.into(),
        replacements: vec![Replacement {
            span: expr_span,
            new_text: new_text.into(),
        }],
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    crate::external_command::detect_external_commands(
        context,
        "prefer_builtin_other",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_builtin_other",
        RuleCategory::Idioms,
        Severity::Info,
        "Avoid external commands when Nushell built-ins are available (env, date, whoami, man, \
         sed, awk, cut, wc, tr, tee, etc.)",
        check,
    )
}

#[cfg(test)]
mod tests;
