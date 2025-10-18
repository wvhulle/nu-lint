use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
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

fn check(context: &LintContext) -> Vec<Violation> {
    let complete_pat = complete_pattern();
    let var_pat = var_pattern();

    context.violations_from_regex(
        complete_pat,
        "consistent_error_handling",
        Severity::Warning,
        |mat| {
            let caps = var_pat.captures(mat.as_str())?;
            let var_name = &caps[1];

            let remaining_source = &context.source[mat.end()..];
            let next_100_chars = &remaining_source[..remaining_source.len().min(200)];

            let exit_code_check = format!(r"\${}\s*\.\s*exit_code", regex::escape(var_name));

            if Regex::new(&exit_code_check)
                .unwrap()
                .is_match(next_100_chars)
            {
                None
            } else {
                Some((
                    format!(
                        "External command result '{var_name}' stored but exit code not checked"
                    ),
                    Some(
                        "Check 'exit_code' field to handle command failures: if $result.exit_code \
                         != 0 { ... }"
                            .to_string(),
                    ),
                ))
            }
        },
    )
}

pub fn rule() -> Rule {
    Rule::new(
        "consistent_error_handling",
        RuleCategory::ErrorHandling,
        Severity::Warning,
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
