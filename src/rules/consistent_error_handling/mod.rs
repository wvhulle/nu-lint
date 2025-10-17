use std::sync::OnceLock;

use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

pub struct ConsistentErrorHandling;

impl ConsistentErrorHandling {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    fn complete_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| {
            Regex::new(r"let\s+(\w+)\s*=\s*\([^)]*\^[^)]*\|\s*complete\s*\)").unwrap()
        })
    }

    fn var_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"let\s+(\w+)").unwrap())
    }
}

impl Default for ConsistentErrorHandling {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ConsistentErrorHandling {
    fn id(&self) -> &'static str {
        "consistent_error_handling"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Check external command results consistently for better error handling"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let complete_pattern = Self::complete_pattern();
        let var_pattern = Self::var_pattern();

        context.violations_from_regex_if(complete_pattern, self.id(), self.severity(), |mat| {
            let caps = var_pattern.captures(mat.as_str())?;
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
        })
    }
}
#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
