use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct PreferReduceOverFold;

impl PreferReduceOverFold {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreferReduceOverFold {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferReduceOverFold {
    fn id(&self) -> &str {
        "BP010"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Use 'reduce' for accumulating values in functional style"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Pattern: mut accumulator = initial_value; for item in collection { accumulator = ... }
        let fold_pattern =
            Regex::new(r"mut\s+(\w+)\s*=\s*([^\n]+)\s*\n[^\n]*for\s+\w+\s+in\s+[^\{]+\{").unwrap();

        violations.extend(context.violations_from_regex_if(&fold_pattern, self.id(), self.severity(), |mat| {
            let caps = fold_pattern.captures(mat.as_str())?;
            let acc_name = &caps[1];

            // Look ahead to see if the accumulator is modified in the loop
            let remaining = &context.source[mat.end()..];
            let body_end = remaining.find('}').unwrap_or(200.min(remaining.len()));
            let body = &remaining[..body_end];

            // Check if accumulator is being updated
            let acc_update_pattern = format!(r"\${}\s*=", regex::escape(acc_name));
            if Regex::new(&acc_update_pattern).unwrap().is_match(body) {
                Some((
                    format!(
                        "For loop accumulating into '{}' - consider using 'reduce' for functional fold",
                        acc_name
                    ),
                    Some("Use '$collection | reduce { |item, acc| ... }' for functional accumulation".to_string()),
                ))
            } else {
                None
            }
        }));

        // Also detect while loops with accumulation
        let while_fold =
            Regex::new(r"mut\s+(\w+)\s*=\s*[^\n]+\s*\n[^\n]*while\s+[^\{]+\{").unwrap();

        violations.extend(context.violations_from_regex_if(
            &while_fold,
            self.id(),
            self.severity(),
            |mat| {
                let caps = while_fold.captures(mat.as_str())?;
                let acc_name = &caps[1];

                let remaining = &context.source[mat.end()..];
                let body_end = remaining.find('}').unwrap_or(200.min(remaining.len()));
                let body = &remaining[..body_end];

                let acc_update_pattern = format!(r"\${}\s*[+\-*/]=", regex::escape(acc_name));
                if Regex::new(&acc_update_pattern).unwrap().is_match(body) {
                    // Skip if already flagged by BP003 (while with counter)
                    let is_simple_counter = body
                        .contains(&format!("${} = ${} + 1", acc_name, acc_name))
                        || body.contains(&format!("${} += 1", acc_name));

                    if !is_simple_counter {
                        Some((
                            format!(
                                "While loop accumulating into '{}' - consider functional approach",
                                acc_name
                            ),
                            Some(
                                "Consider using 'reduce' or pipeline operations for accumulation"
                                    .to_string(),
                            ),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
        ));

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_loop_accumulation_detected() {
        let rule = PreferReduceOverFold::new();

        let bad_code = r#"
mut sum = 0
for item in $items {
    $sum = $sum + $item
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect for loop accumulation"
        );
    }

    #[test]
    fn test_reduce_not_flagged() {
        let rule = PreferReduceOverFold::new();

        let good_code = r#"
let sum = ($items | reduce { |item, acc| $acc + $item })
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(rule.check(&context).len(), 0, "Should not flag reduce");
    }

    #[test]
    fn test_immutable_loop_not_flagged() {
        let rule = PreferReduceOverFold::new();

        let good_code = r#"
let sum = 0
for item in $items {
    echo $item
}
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag non-accumulating loops"
        );
    }
}
