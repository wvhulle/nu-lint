use std::collections::HashMap;

use crate::{
    RuleViolation,
    context::LintContext,
    external_command::{BuiltinAlternative, Fix, extract_external_args},
    lint::{Replacement, Severity},
    rule::{Rule, RuleCategory},
};

/// Map of common file and text operations to their Nushell built-in
/// equivalents Based on <https://www.nushell.sh/book/coming_from_bash.html#command-equivalents>
///
/// This rule focuses on the most commonly used commands when migrating from
/// bash. See also: BP013 (text transformation), BP014 (system commands)
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
        BuiltinAlternative::with_note(
            "ls or glob",
            "Use 'ls **/*.ext' for recursive file matching, 'glob **/*.ext' for pattern matching, \
             or 'ls' with pipes for complex filtering",
        ),
    );

    // Common text processing operations
    map.insert(
        "grep",
        BuiltinAlternative::with_note(
            "where or find",
            "Use 'where $it =~ <pattern>' for regex filtering, 'find <substring>' for text \
             search, or 'search <term>' for full-text search across structured data",
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

/// Build a simple fix for common external command replacements
fn build_fix(
    cmd_text: &str,
    alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &LintContext,
) -> Fix {
    // Extract arguments
    let args_text = extract_external_args(args, context);

    // Build replacement and description based on command
    let (new_text, description) = match cmd_text {
        "ls" => {
            // ^ls -la -> ls -la (remove ^)
            let replacement = if args_text.is_empty() {
                "ls".to_string()
            } else {
                format!("ls {}", args_text.join(" "))
            };
            let desc = "Use Nu's built-in 'ls' which returns structured table data with name, type, size, and modified columns".to_string();
            (replacement, desc)
        }
        "cat" => {
            // ^cat file.txt -> open --raw file.txt
            let replacement = if let Some(file) = args_text.iter().find(|a| !a.starts_with('-')) {
                format!("open --raw {file}")
            } else {
                alternative.command.to_string()
            };
            let desc = "Use 'open --raw' for plain text, or just 'open' to auto-parse structured files (JSON, TOML, CSV, etc.)".to_string();
            (replacement, desc)
        }
        "grep" => {
            // For simple cases, provide better suggestions
            if args_text.len() == 1 && !args_text[0].starts_with('-') {
                // Simple text search: grep "pattern" -> find "pattern"
                let replacement = format!("find \"{}\"", args_text[0]);
                let desc =
                    "Use 'find' which is case-insensitive by default and works on structured data"
                        .to_string();
                (replacement, desc)
            } else if args_text.contains(&"-i".to_string()) {
                // grep -i -> find (case-insensitive is default in Nu)
                let replacement = "where $it =~ \"pattern\"".to_string();
                let desc = "Use 'find' (case-insensitive by default) or 'where $it =~ pattern' for regex filtering. The -i flag is redundant in Nu".to_string();
                (replacement, desc)
            } else {
                // Complex case with flags - suggest the alternative without specific fix
                let replacement = "where $it =~ \"pattern\"".to_string();
                let desc = "Use 'where $it =~ pattern' for regex filtering or 'find' for simple text search".to_string();
                (replacement, desc)
            }
        }
        "head" | "tail" => {
            // ^head -5 -> first 5 or ^tail -5 -> last 5
            let builtin = if cmd_text == "head" { "first" } else { "last" };
            let replacement = if let Some(num_arg) =
                args_text.iter().find(|a| a.starts_with('-') && a.len() > 1)
            {
                let num = &num_arg[1..];
                format!("{builtin} {num}")
            } else {
                format!("{builtin} 10")
            };
            let desc = format!(
                "Use '{builtin}' with cleaner syntax: '{builtin} N' instead of '{cmd_text} -N'"
            );
            (replacement, desc)
        }
        "sort" => {
            // Direct replacement
            let desc = "Use Nu's built-in 'sort' which works on any data type and supports natural sorting with -n flag".to_string();
            (cmd_text.to_string(), desc)
        }
        "uniq" => {
            let desc = "Use Nu's built-in 'uniq' which works on structured data and supports 'uniq-by' for specific columns".to_string();
            (cmd_text.to_string(), desc)
        }
        "find" => {
            // Provide better find replacements based on common usage patterns
            let (replacement, desc) = if args_text.iter().any(|arg| arg.contains("*.")) {
                // find . -name "*.rs" -> ls **/*.rs
                let repl = if let Some(pattern) = args_text.iter().find(|arg| arg.contains("*.")) {
                    format!("ls **/{}", pattern.trim_matches('"'))
                } else {
                    "ls **/*".to_string()
                };
                (
                    repl,
                    "Use 'ls' with glob patterns (**/*.ext) for recursive file searches"
                        .to_string(),
                )
            } else if args_text.len() == 1 && !args_text[0].starts_with('-') {
                // find dirname -> ls dirname/**/*
                (
                    format!("ls {}/**/*", args_text[0]),
                    "Use 'ls' with glob patterns for directory traversal".to_string(),
                )
            } else {
                // Default case
                ("ls **/*".to_string(), "Use 'ls' with glob patterns for file finding, or 'glob' for more complex patterns".to_string())
            };
            (replacement, desc)
        }
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
        "avoid_external_file_tools",
        &get_builtin_alternatives(),
        Some(build_fix),
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "avoid_external_file_tools",
        RuleCategory::Idioms,
        Severity::Info,
        "Avoid external file tools when Nushell built-ins are available (ls, cat, grep, head, \
         tail, sort, uniq, find)",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
