use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

fn check(context: &LintContext) -> Vec<Violation> {
    let def_pattern = Regex::new(r"(?m)^[ \t]*def\s+([a-zA-Z_-][a-zA-Z0-9_-]*)\s*\[").unwrap();

    context.violations_from_regex(
        &def_pattern,
        "missing_command_docs",
        Severity::Warning,
        |mat| {
            let caps = def_pattern.captures(mat.as_str())?;
            let cmd_name = caps.get(1)?.as_str();
            let def_start = mat.start();

            // Check if there's a comment (starting with #) on the line before
            let lines_before: Vec<&str> = context.source[..def_start].lines().collect();
            let has_doc = if let Some(prev_line) = lines_before.last() {
                prev_line.trim().starts_with('#')
            } else {
                false
            };

            if has_doc {
                None
            } else {
                Some((
                    format!("Command '{cmd_name}' is missing documentation comments"),
                    Some("Add a comment starting with # above the def statement".to_string()),
                ))
            }
        },
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "missing_command_docs",
        RuleCategory::Documentation,
        Severity::Warning,
        "Custom commands should have documentation comments",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
