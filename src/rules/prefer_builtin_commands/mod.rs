use std::collections::HashMap;

use crate::{
    context::LintContext,
    external_command::{BuiltinAlternative, ExternalCommandVisitor},
    lint::{Fix, Replacement, Severity, Violation},
    rule::{RegexRule, RuleCategory, RuleMetadata},
    visitor::VisitContext,
};

pub struct AvoidExternalFileTools;

impl AvoidExternalFileTools {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

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
                "Use 'ls **/*.ext' for recursive file matching, 'glob **/*.ext' for pattern \
                 matching, or 'ls' with pipes for complex filtering",
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
}

impl Default for AvoidExternalFileTools {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleMetadata for AvoidExternalFileTools {
    fn id(&self) -> &'static str {
        "avoid_external_file_tools"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Idioms
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Avoid external file tools when Nushell built-ins are available (ls, cat, grep, head, \
         tail, sort, uniq, find)"
    }
}

impl RegexRule for AvoidExternalFileTools {
    fn check(&self, context: &LintContext) -> Vec<Violation> {
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

/// Build a simple fix for common external command replacements
fn build_fix(
    cmd_text: &str,
    alternative: &BuiltinAlternative,
    args: &[nu_protocol::ast::ExternalArgument],
    expr_span: nu_protocol::Span,
    context: &VisitContext,
) -> Fix {
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
                format!("open --raw {file}")
            } else {
                alternative.command.to_string()
            }
        }
        "grep" => {
            // For simple cases, provide better suggestions
            if args_text.len() == 1 && !args_text[0].starts_with('-') {
                // Simple text search: grep "pattern" -> find "pattern"
                format!("find \"{}\"", args_text[0])
            } else {
                // Complex case with flags - suggest the alternative without specific fix
                "where $it =~ \"pattern\"".to_string()
            }
        }
        "head" | "tail" => {
            // ^head -5 -> first 5 or ^tail -5 -> last 5
            let builtin = if cmd_text == "head" { "first" } else { "last" };
            if let Some(num_arg) = args_text.iter().find(|a| a.starts_with('-') && a.len() > 1) {
                let num = &num_arg[1..];
                format!("{builtin} {num}")
            } else {
                format!("{builtin} 10")
            }
        }
        "sort" | "uniq" => {
            // Direct replacement
            cmd_text.to_string()
        }
        "find" => {
            // Provide better find replacements based on common usage patterns
            if args_text.iter().any(|arg| arg.contains("*.")) {
                // find . -name "*.rs" -> ls **/*.rs
                if let Some(pattern) = args_text.iter().find(|arg| arg.contains("*.")) {
                    format!("ls **/{}", pattern.trim_matches('"'))
                } else {
                    "ls **/*".to_string()
                }
            } else if args_text.len() == 1 && !args_text[0].starts_with('-') {
                // find dirname -> ls dirname/**/*
                format!("ls {}/**/*", args_text[0])
            } else {
                // Default case
                "ls **/*".to_string()
            }
        }
        _ => alternative.command.to_string(),
    };

    Fix {
        description: format!("Replace '^{}' with '{}'", cmd_text, alternative.command),
        replacements: vec![Replacement {
            span: expr_span,
            new_text,
        }],
    }
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
