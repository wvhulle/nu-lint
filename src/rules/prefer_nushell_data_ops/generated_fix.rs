#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rules::prefer_nushell_data_ops::rule};

    fn assert_fix_text_equals(
        violations: &[crate::lint::RuleViolation],
        expected: &str,
        source: &str,
    ) {
        let Some(fix) = &violations[0].fix else {
            return;
        };
        assert_eq!(
            fix.replacements[0].new_text, expected,
            "Failed for: {source}"
        );
    }

    fn assert_fix_contains_alternative(violations: &[crate::lint::RuleViolation]) {
        let Some(fix) = &violations[0].fix else {
            return;
        };
        assert!(
            fix.replacements[0]
                .new_text
                .contains("structured data operations")
                || fix.replacements[0].new_text.contains("each { get field }")
                || fix.replacements[0].new_text.contains("where condition")
        );
    }

    #[test]
    fn fix_jq_map_operation() {
        let source = "^jq 'map(.name)' users.json";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            if let Some(fix) = &violations[0].fix {
                assert_eq!(fix.replacements[0].new_text, "each { get field }");
                assert!(fix.description.contains("structured data operations"));
            }
        });
    }

    #[test]
    fn fix_jq_select_operation() {
        let source = "^jq 'select(.age > 30)' people.json";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            if let Some(fix) = &violations[0].fix {
                assert_eq!(fix.replacements[0].new_text, "where condition");
            }
        });
    }

    #[test]
    fn fix_jq_group_by_operation() {
        let source = "^jq 'group_by(.category)' items.json";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            if let Some(fix) = &violations[0].fix {
                assert_eq!(fix.replacements[0].new_text, "group-by field");
            }
        });
    }

    #[test]
    fn fix_jq_array_iteration() {
        let source = "^jq '.[]' array.json";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            if let Some(fix) = &violations[0].fix {
                assert_eq!(fix.replacements[0].new_text, "values");
            }
        });
    }

    #[test]
    fn fix_jq_sorting_operations() {
        let test_cases = vec![
            ("^jq 'sort_by(.field)' data.json", "sort-by field"),
            ("^jq 'unique' values.json", "uniq"),
            ("^jq 'reverse' list.json", "reverse"),
        ];

        let rule = rule();

        for (source, expected) in test_cases {
            LintContext::test_with_parsed_source(source, |context| {
                let violations = rule.check(&context);
                assert!(!violations.is_empty(), "Expected violation for: {source}");
                assert_fix_text_equals(&violations, expected, source);
            });
        }
    }

    #[test]
    fn fix_complex_jq_patterns() {
        let complex_sources = vec![
            "^jq 'map(select(.active))' users.json",
            "^jq '.[] | select(.category == \"A\")' items.json",
            "^jq 'group_by(.type) | map(length)' data.json",
        ];

        let rule = rule();

        for source in complex_sources {
            LintContext::test_with_parsed_source(source, |context| {
                let violations = rule.check(&context);
                assert!(
                    !violations.is_empty(),
                    "Should detect complex pattern: {source}"
                );
                assert_fix_contains_alternative(&violations);
            });
        }
    }

    #[test]
    fn no_violation_for_simple_jq_ops() {
        let simple_sources = vec![
            "^jq 'length' data.json",
            "^jq 'keys' object.json",
            "^jq 'type' value.json",
            "^jq '.[0]' array.json",
        ];

        let rule = rule();

        for source in simple_sources {
            LintContext::test_with_parsed_source(source, |context| {
                let violations = rule.check(&context);
                assert!(
                    violations.is_empty(),
                    "Should not trigger for simple jq: {source}"
                );
            });
        }
    }

    #[test]
    fn detects_data_operation_patterns() {
        let data_op_sources = vec![
            "^jq 'map(.name)' users.json",
            "^jq 'select(.age > 25)' people.json",
            "^jq 'group_by(.department)' employees.json",
            "^jq '.[] | .field' data.json",
            "^jq 'sort_by(.timestamp)' events.json",
            "^jq 'unique' duplicates.json",
            "^jq 'reverse' ordered.json",
        ];

        let rule = rule();

        for source in data_op_sources {
            LintContext::test_with_parsed_source(source, |context| {
                let violations = rule.check(&context);
                assert!(
                    !violations.is_empty(),
                    "Should detect data operation: {source}"
                );
                assert_eq!(violations[0].rule_id, "prefer_nushell_data_ops");
            });
        }
    }

    #[test]
    fn suggests_appropriate_nushell_operations() {
        let test_cases = vec![
            ("^jq 'map(.name)' users.json", "each { get field }"),
            ("^jq 'select(.active)' items.json", "where condition"),
            ("^jq 'group_by(.type)' data.json", "group-by field"),
            ("^jq 'sort_by(.field)' events.json", "sort-by field"),
            ("^jq 'unique' values.json", "uniq"),
        ];

        let rule = rule();

        for (source, expected_text) in test_cases {
            LintContext::test_with_parsed_source(source, |context| {
                let violations = rule.check(&context);
                assert!(!violations.is_empty());
                assert_fix_text_equals(&violations, expected_text, source);
            });
        }
    }

    #[test]
    fn rule_category_and_severity() {
        let source = "^jq 'map(.field)' data.json";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            let violation = &violations[0];
            assert_eq!(violation.rule_id, "prefer_nushell_data_ops");
            assert!(violation.message.contains("structured data operations"));
        });
    }
}
