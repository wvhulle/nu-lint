use regex::Regex;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let def_pattern = Regex::new(r"(?m)^[ \t]*def\s+([a-zA-Z_-][a-zA-Z0-9_-]*)\s*\[").unwrap();

    def_pattern
        .find_iter(context.source)
        .filter_map(|mat| {
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
                Some(
                    RuleViolation::new_dynamic(
                        "missing_command_docs",
                        format!("Command '{cmd_name}' is missing documentation comments"),
                        nu_protocol::Span::new(mat.start(), mat.end()),
                    )
                    .with_suggestion_static(
                        "Add a comment starting with # above the def statement",
                    ),
                )
            }
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "missing_command_docs",
        RuleCategory::Documentation,
        Severity::Info,
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
