use crate::context::LintContext;
use crate::lint::{Severity, Violation};
use crate::rule::{Rule, RuleCategory};
use regex::Regex;
use std::sync::OnceLock;

#[derive(Default)]
pub struct PreferWhereOverEachIf;

impl PreferWhereOverEachIf {
    fn each_if_pattern() -> &'static Regex {
        static PATTERN: OnceLock<Regex> = OnceLock::new();
        PATTERN.get_or_init(|| Regex::new(r"each\s*\{\s*\|([^}|]+)\|\s*if\s+([^}]+)\}").unwrap())
    }

    // Side effects that indicate this is processing, not filtering
    const SIDE_EFFECTS: &'static [&'static str] =
        &["print", "save", "download", "^", "exit", "=", "mut "];
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
        context.violations_from_regex_if(
            Self::each_if_pattern(),
            self.id(),
            self.severity(),
            |mat| {
                let caps = Self::each_if_pattern().captures(mat.as_str())?;
                let condition_and_body = caps.get(2)?.as_str();

                let body = condition_and_body
                    .find('{')
                    .map_or(condition_and_body, |pos| &condition_and_body[pos + 1..])
                    .trim();

                // Check if this is pure filtering (no side effects)
                (!Self::has_side_effects(body)).then(|| {
                    (
                        "Consider using 'where' for filtering instead of 'each' with 'if'"
                            .to_string(),
                        Some("Use '$list | where <condition>' for better performance".to_string()),
                    )
                })
            },
        )
    }
}

impl PreferWhereOverEachIf {
    fn has_side_effects(code: &str) -> bool {
        Self::SIDE_EFFECTS
            .iter()
            .any(|&effect| code.contains(effect))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_filtering_flagged() {
        let rule = PreferWhereOverEachIf::default();

        let filtering_code = r"
$items | each { |item| if $item.is_valid { $item } }
";
        let context = LintContext::test_from_source(filtering_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should flag pure filtering"
        );
    }

    #[test]
    fn test_processing_with_side_effects_not_flagged() {
        let rule = PreferWhereOverEachIf::default();

        let processing_code = r"
$items | each { |i| if $i.marked_for_download { download $i } }
";
        let context = LintContext::test_from_source(processing_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag processing with side effects"
        );
    }

    #[test]
    fn test_print_side_effect_not_flagged() {
        let rule = PreferWhereOverEachIf::default();

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
        let rule = PreferWhereOverEachIf::default();

        let external_code = r"
$files | each { |file| if ($file | path exists) { ^some-tool $file } }
";
        let context = LintContext::test_from_source(external_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag code with external commands"
        );
    }

    #[test]
    fn test_simple_boolean_condition_flagged() {
        let rule = PreferWhereOverEachIf::default();

        let simple_filter = r"
$numbers | each { |n| if $n > 10 { $n } }
";
        let context = LintContext::test_from_source(simple_filter);
        assert!(
            !rule.check(&context).is_empty(),
            "Should flag simple boolean filtering"
        );
    }

    #[test]
    fn test_assignment_not_flagged() {
        let rule = PreferWhereOverEachIf::default();

        let assignment_code = r"
$items | each { |item| if $item.valid { $processed = process $item; $processed } }
";
        let context = LintContext::test_from_source(assignment_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag code with assignments"
        );
    }
}
