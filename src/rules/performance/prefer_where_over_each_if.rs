use crate::context::{LintContext, Rule, RuleCategory, Severity, Violation};
use regex::Regex;

pub struct PreferWhereOverEachIf;

impl PreferWhereOverEachIf {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PreferWhereOverEachIf {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PreferWhereOverEachIf {
    fn id(&self) -> &str {
        "P001"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Performance
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &str {
        "Use 'where' for filtering instead of 'each' with 'if'"
    }

    fn check(&self, context: &LintContext) -> Vec<Violation> {
        // Look for "each { |item| if $item..." pattern with more detail
        let pattern = Regex::new(r"each\s*\{\s*\|([^}|]+)\|\s*if\s+([^}]+)\}").unwrap();

        context.violations_from_regex_if(&pattern, self.id(), self.severity(), |mat| {
            if let Some(caps) = pattern.captures(mat.as_str()) {
                let _var = &caps[1].trim();
                let condition_and_body = &caps[2];

                // Split on the first opening brace to separate condition from body
                if let Some(brace_pos) = condition_and_body.find('{') {
                    let _condition = &condition_and_body[..brace_pos].trim();
                    let body = &condition_and_body[brace_pos + 1..].trim();

                    // Only suggest 'where' if this is pure filtering
                    if Self::is_pure_filtering(body) {
                        Some((
                            "Consider using 'where' for filtering instead of 'each' with 'if'"
                                .to_string(),
                            Some(
                                "Use '$list | where <condition>' for better performance"
                                    .to_string(),
                            ),
                        ))
                    } else {
                        None
                    }
                } else {
                    // No body found - might be a simple condition
                    // Could be filtering if condition returns a value directly
                    if Self::looks_like_simple_filter(condition_and_body) {
                        Some((
                            "Consider using 'where' for filtering instead of 'each' with 'if'"
                                .to_string(),
                            Some(
                                "Use '$list | where <condition>' for better performance"
                                    .to_string(),
                            ),
                        ))
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        })
    }
}

impl PreferWhereOverEachIf {
    fn is_pure_filtering(body: &str) -> bool {
        let body = body.trim();

        // If body is empty or just returns the item, it's filtering
        if body.is_empty()
            || body == "$it"
            || body.starts_with("$") && body.contains(".") && !body.contains("=")
        {
            return true;
        }

        // Check for side effects that indicate processing, not filtering
        let has_side_effects = body.contains("print")
            || body.contains("save")
            || body.contains("download")
            || body.contains("^")
            || body.contains("exit")
            || body.contains("=")
            || body.contains("mut ");

        !has_side_effects
    }

    fn looks_like_simple_filter(condition_and_body: &str) -> bool {
        let text = condition_and_body.trim();

        // Simple boolean conditions without side effects
        !text.contains("print")
            && !text.contains("save")
            && !text.contains("download")
            && !text.contains("^")
            && !text.contains("=")
            && (text.contains("==")
                || text.contains("!=")
                || text.contains("<")
                || text.contains(">"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_filtering_flagged() {
        let rule = PreferWhereOverEachIf::new();

        let filtering_code = r#"
$items | each { |item| if $item.is_valid { $item } }
"#;
        let context = LintContext::test_from_source(filtering_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should flag pure filtering"
        );
    }

    #[test]
    fn test_processing_with_side_effects_not_flagged() {
        let rule = PreferWhereOverEachIf::new();

        let processing_code = r#"
$items | each { |i| if $i.marked_for_download { download $i } }
"#;
        let context = LintContext::test_from_source(processing_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag processing with side effects"
        );
    }

    #[test]
    fn test_print_side_effect_not_flagged() {
        let rule = PreferWhereOverEachIf::new();

        let print_code = r#"
$files | each { |file| if ($file | path exists) { print $"Processing ($file)" } }
"#;
        let context = LintContext::test_from_source(print_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag code with print side effects"
        );
    }

    #[test]
    fn test_external_command_not_flagged() {
        let rule = PreferWhereOverEachIf::new();

        let external_code = r#"
$files | each { |file| if ($file | path exists) { ^some-tool $file } }
"#;
        let context = LintContext::test_from_source(external_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag code with external commands"
        );
    }

    #[test]
    fn test_simple_boolean_condition_flagged() {
        let rule = PreferWhereOverEachIf::new();

        let simple_filter = r#"
$numbers | each { |n| if $n > 10 { $n } }
"#;
        let context = LintContext::test_from_source(simple_filter);
        assert!(
            !rule.check(&context).is_empty(),
            "Should flag simple boolean filtering"
        );
    }

    #[test]
    fn test_assignment_not_flagged() {
        let rule = PreferWhereOverEachIf::new();

        let assignment_code = r#"
$items | each { |item| if $item.valid { $processed = process $item; $processed } }
"#;
        let context = LintContext::test_from_source(assignment_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag code with assignments"
        );
    }
}
