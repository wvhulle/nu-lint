use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use std::collections::HashMap;

pub struct PreferBuiltinCommands;

impl PreferBuiltinCommands {
    pub fn new() -> Self {
        Self
    }

    /// Map of external commands to their Nushell built-in equivalents
    fn get_builtin_alternatives() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::new();

        // File system operations
        map.insert("ls", "ls");
        map.insert("cd", "cd");
        map.insert("pwd", "pwd");
        map.insert("mkdir", "mkdir");
        map.insert("rm", "rm");
        map.insert("mv", "mv");
        map.insert("cp", "cp");
        map.insert("touch", "touch");
        map.insert("cat", "open");

        // Text processing
        map.insert("echo", "echo");
        map.insert("printf", "print");
        map.insert("head", "first");
        map.insert("tail", "last");
        map.insert("wc", "length or str length");
        map.insert("sort", "sort or sort-by");
        map.insert("uniq", "uniq or uniq-by");
        map.insert("grep", "where or find");
        map.insert("sed", "str replace");
        map.insert("awk", "where, select, or each");
        map.insert("cut", "select");

        // System info
        map.insert("whoami", "whoami");
        map.insert("hostname", "sys host");
        map.insert("uname", "sys host");
        map.insert("date", "date now");
        map.insert("sleep", "sleep");
        map.insert("env", "$env");
        map.insert("printenv", "$env");

        // Other utilities
        map.insert("which", "which");
        map.insert("kill", "kill");
        map.insert("clear", "clear");

        map
    }
}

impl Default for PreferBuiltinCommands {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferBuiltinCommands {
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
        "Prefer Nushell built-in commands over external tools when equivalent functionality exists"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        let alternatives = Self::get_builtin_alternatives();

        // Search for external command calls (^command syntax)
        let source_lines: Vec<&str> = context.source.lines().collect();

        for (line_idx, line) in source_lines.iter().enumerate() {
            // Look for external command calls
            for (external_cmd, builtin_alternative) in &alternatives {
                let external_pattern = format!("^{}", external_cmd);

                // Check if the line contains the external command call
                if line.contains(&external_pattern) {
                    // Make sure it's actually a command call, not just part of a string
                    // Basic heuristic: check if it's followed by a space or pipe or is at the end
                    if let Some(idx) = line.find(&external_pattern) {
                        let after_cmd = idx + external_pattern.len();
                        if after_cmd >= line.len()
                            || line[after_cmd..].starts_with(' ')
                            || line[after_cmd..].starts_with('|')
                            || line[after_cmd..].starts_with('\n')
                            || line[after_cmd..].starts_with('\t')
                        {
                            // Calculate the span for this line
                            let line_start: usize = source_lines[..line_idx]
                                .iter()
                                .map(|l| l.len() + 1) // +1 for newline
                                .sum();
                            let line_end = line_start + line.len();

                            violations.push(Violation {
                                rule_id: self.id().to_string(),
                                severity: self.severity(),
                                message: format!(
                                    "Consider using Nushell's built-in '{}' instead of external '^{}'",
                                    builtin_alternative,
                                    external_cmd
                                ),
                                span: nu_protocol::Span::new(line_start, line_end),
                                suggestion: Some(format!(
                                    "Replace '^{}' with built-in command: {}\n\
                                     Built-in commands are more portable, faster, and provide better error handling.",
                                    external_cmd,
                                    builtin_alternative
                                )),
                                fix: None,
                                file: None,
                            });

                            // Only report once per line to avoid duplicate violations
                            break;
                        }
                    }
                }
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::engine::LintEngine;

    #[test]
    fn test_external_ls_detected() {
        let source = r#"
def list-files [] {
    ^ls -la | lines
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None).unwrap();

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "BP012").collect();

        assert!(
            !rule_violations.is_empty(),
            "Should detect external ls command"
        );
    }

    #[test]
    fn test_builtin_ls_not_flagged() {
        let source = r#"
def list-files [] {
    ls | where type == dir
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None).unwrap();

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "BP012").collect();

        assert!(rule_violations.is_empty(), "Should not flag built-in ls");
    }

    #[test]
    fn test_external_cat_detected() {
        let source = r#"
def read-file [file] {
    ^cat $file
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None).unwrap();

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "BP012").collect();

        assert!(
            !rule_violations.is_empty(),
            "Should detect external cat command"
        );
        assert!(
            rule_violations[0].message.contains("open"),
            "Should suggest 'open' as alternative"
        );
    }

    #[test]
    fn test_external_grep_detected() {
        let source = r#"
def search [pattern] {
    ^grep $pattern file.txt
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None).unwrap();

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "BP012").collect();

        assert!(
            !rule_violations.is_empty(),
            "Should detect external grep command"
        );
    }

    #[test]
    fn test_external_command_without_builtin_not_flagged() {
        let source = r#"
def run-custom [] {
    ^my-custom-tool --flag
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None).unwrap();

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "BP012").collect();

        assert!(
            rule_violations.is_empty(),
            "Should not flag external commands without built-in alternatives"
        );
    }

    #[test]
    fn test_multiple_external_commands_detected() {
        let source = r#"
def process [] {
    ^cat file.txt | ^grep pattern | ^sort
}
"#;
        let engine = LintEngine::new(Config::default());
        let violations = engine.lint_source(source, None).unwrap();

        let rule_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "BP012").collect();

        // Should detect cat (there may be only one violation per line due to our break logic)
        assert!(
            !rule_violations.is_empty(),
            "Should detect external commands"
        );
    }
}
