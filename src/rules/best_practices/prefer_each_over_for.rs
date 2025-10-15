use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct PreferEachOverFor;

impl PreferEachOverFor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreferEachOverFor {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferEachOverFor {
    fn id(&self) -> &str {
        "BP008"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::BestPractices
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn description(&self) -> &str {
        "Use 'each' pipeline instead of 'for' loops for functional style"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Pattern: for item in $collection { ... }
        let for_loop_pattern = Regex::new(r"for\s+(\w+)\s+in\s+(\$\w+|\([^\)]+\))\s*\{").unwrap();

        context.violations_from_regex_if(&for_loop_pattern, self.id(), self.severity(), |mat| {
            let caps = for_loop_pattern.captures(mat.as_str())?;
            let item_var = &caps[1];
            let collection = &caps[2];

            // Get a snippet of the loop body to check if it's doing side effects
            let remaining = &context.source[mat.end()..];
            let body_end = remaining.find('}').unwrap_or(100.min(remaining.len()));
            let body = &remaining[..body_end];

            // Check for side effects that make 'for' more appropriate
            let has_external_commands = body.contains('^');
            let has_print = body.contains("print");
            let has_mutation = body.starts_with("mut ") || body.contains(" mut ");

            // Only suggest 'each' if there are no obvious side effects
            if !has_external_commands && !has_print && !has_mutation {
                Some((
                    format!(
                        "For loop iterating '{}' - consider using 'each' for functional style",
                        item_var
                    ),
                    Some(format!(
                        "Use '{} | each {{ |{}| ... }}' for functional iteration",
                        collection, item_var
                    )),
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
    fn test_for_loop_detected() {
        let rule = PreferEachOverFor::new();

        let bad_code = r#"
for item in $items {
    echo $item
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(!rule.check(&context).is_empty(), "Should detect for loop");
    }

    #[test]
    fn test_for_loop_with_external_command_not_flagged() {
        let rule = PreferEachOverFor::new();

        let acceptable_code = r#"
for device in $devices {
    ^bluetoothctl connect $device
}
"#;
        let context = LintContext::test_from_source(acceptable_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag for loops with external commands"
        );
    }

    #[test]
    fn test_for_loop_with_print_not_flagged() {
        let rule = PreferEachOverFor::new();

        let acceptable_code = r#"
for item in $items {
    print $item
}
"#;
        let context = LintContext::test_from_source(acceptable_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag for loops with print"
        );
    }

    #[test]
    fn test_each_not_flagged() {
        let rule = PreferEachOverFor::new();

        let good_code = r#"
$items | each { |item| $item * 2 }
"#;
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag each pipelines"
        );
    }
}
