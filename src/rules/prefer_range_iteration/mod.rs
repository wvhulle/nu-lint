use regex::Regex;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

pub struct PreferRangeIteration;

impl PreferRangeIteration {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreferRangeIteration {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferRangeIteration {
    fn id(&self) -> &'static str {
        "prefer_range_iteration"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &'static str {
        "Prefer range iteration over while loops with counters"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Pattern: mut counter = 0, while counter < max, counter increment
        let mut_counter_pattern = Regex::new(r"mut\s+(\w+)\s*=\s*0").unwrap();

        context.violations_from_regex_if(&mut_counter_pattern, self.id(), self.severity(), |mat| {
            let counter_name = mat.as_str().split_whitespace().nth(1)?;

            // Check if there's a while loop using this counter
            let while_pattern = format!(r"while\s+\${}?\s*<", regex::escape(counter_name));
            let increment_pattern = format!(
                r"\${}?\s*=\s*\${}\s*\+\s*1|\${}?\s*\+=\s*1",
                regex::escape(counter_name),
                regex::escape(counter_name),
                regex::escape(counter_name)
            );

            if Regex::new(&while_pattern).unwrap().is_match(context.source)
                && Regex::new(&increment_pattern)
                    .unwrap()
                    .is_match(context.source)
            {
                Some((
                    format!(
                        "While loop with counter '{counter_name}' - consider using range iteration"
                    ),
                    Some(
                        "Use '1..$max | each { |i| ... }' instead of while loop with counter"
                            .to_string(),
                    ),
                ))
            } else {
                None
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
