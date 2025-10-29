use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn complete_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN
        .get_or_init(|| Regex::new(r"let\s+(\w+)\s*=\s*\([^)]*\^[^)]*\|\s*complete\s*\)").unwrap())
}

fn var_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"let\s+(\w+)").unwrap())
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let complete_pat = complete_pattern();
    let var_pat = var_pattern();

    complete_pat
        .captures_iter(context.source)
        .filter_map(|mat| {
            let caps = var_pat.captures(mat.get(0)?.as_str())?;
            let var_name = &caps[1];
            let full_match = mat.get(0)?;

            let remaining_source = &context.source[full_match.end()..];
            let next_100_chars = &remaining_source[..remaining_source.len().min(200)];

            let exit_code_check = format!(r"\${}\s*\.\s*exit_code", regex::escape(var_name));

            if Regex::new(&exit_code_check)
                .unwrap()
                .is_match(next_100_chars)
            {
                None
            } else {
                Some(
                    RuleViolation::new_dynamic(
                        "consistent_error_handling",
                        format!(
                            "External command result '{var_name}' stored but exit code not checked"
                        ),
                        nu_protocol::Span::new(full_match.start(), full_match.end()),
                    )
                    .with_suggestion_static(
                        "Check 'exit_code' field to handle command failures: if $result.exit_code \
                         != 0 { ... }",
                    ),
                )
            }
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "consistent_error_handling",
        RuleCategory::ErrorHandling,
        Severity::Error,
        "Check external command results consistently for better error handling",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
