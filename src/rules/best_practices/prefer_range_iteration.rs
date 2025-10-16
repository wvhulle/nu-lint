use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use regex::Regex;

pub struct PreferRangeIteration;

impl PreferRangeIteration {
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
    fn id(&self) -> &str {
        "BP003"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &str {
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
                        "While loop with counter '{}' - consider using range iteration",
                        counter_name
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
mod tests {
    use super::*;

    #[test]
    fn test_while_counter_detected() {
        let rule = PreferRangeIteration::new();

        let bad_code = r"
mut count = 0
while $count < 10 {
    echo $count
    $count = $count + 1
}
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect while loop with counter"
        );
    }

    #[test]
    fn test_compound_increment_detected() {
        let rule = PreferRangeIteration::new();

        let bad_code = r"
mut attempts = 0
while $attempts < 5 {
    try_something
    $attempts += 1
}
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect compound assignment increment"
        );
    }

    #[test]
    fn test_no_false_positive_on_regular_mut() {
        let rule = PreferRangeIteration::new();

        let good_code = r#"
mut accumulator = 0
for item in $items {
    $accumulator += $item
}
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag non-while loop counters"
        );
    }
}
