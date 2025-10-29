#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rules::prefer_structured_data_flow::rule};

    #[test]
    fn detects_to_json_jq_field_access() {
        let source = "$data | to json | ^jq '.field'";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            let violation = &violations[0];
            assert!(violation.message.contains("Converting to JSON string"));
            assert!(
                violation
                    .message
                    .contains("consider keeping data structured")
            );
            assert!(
                violation
                    .suggestion
                    .as_ref()
                    .unwrap()
                    .contains("structured format")
            );
            assert!(violation.suggestion.as_ref().unwrap().contains("get"));
        });
    }

    #[test]
    fn detects_to_json_jq_mapping() {
        let source = "$records | to json | ^jq 'map(.name)'";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            let violation = &violations[0];
            assert!(violation.suggestion.as_ref().unwrap().contains("each"));
            assert!(violation.suggestion.as_ref().unwrap().contains("where"));
        });
    }

    #[test]
    fn detects_to_json_jq_filtering() {
        let source = "$users | to json | ^jq '.[] | select(.age > 30)'";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            let violation = &violations[0];
            assert!(violation.message.contains("Converting to JSON string"));
        });
    }

    #[test]
    fn detects_to_json_jq_array_operations() {
        let test_cases = vec![
            "$numbers | to json | ^jq 'add'",
            "$values | to json | ^jq 'length'",
            "$items | to json | ^jq 'sort'",
            "$data | to json | ^jq 'unique'",
        ];

        let rule = rule();

        for source in test_cases {
            LintContext::test_with_parsed_source(source, |context| {
                let violations = rule.check(&context);
                assert!(
                    !violations.is_empty(),
                    "Should detect inefficient pattern: {source}"
                );

                let violation = &violations[0];
                assert!(violation.message.contains("Converting to JSON string"));
            });
        }
    }

    #[test]
    fn detects_complex_json_jq_chains() {
        let test_cases = vec![
            "$data | to json | ^jq '.users[] | .name'",
            "$records | to json | ^jq 'group_by(.category)'",
            "$items | to json | ^jq 'map(select(.active))'",
        ];

        let rule = rule();

        for source in test_cases {
            LintContext::test_with_parsed_source(source, |context| {
                let violations = rule.check(&context);
                assert!(
                    !violations.is_empty(),
                    "Should detect complex pattern: {source}"
                );
            });
        }
    }

    #[test]
    fn span_covers_both_commands() {
        let source = "$data | to json | ^jq '.field'";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            let violation = &violations[0];
            // The span should cover both 'to json' and '^jq ...' parts
            let span_text = &source[violation.span.start..violation.span.end];
            assert!(span_text.contains("to json"));
            assert!(span_text.contains("jq"));
        });
    }

    #[test]
    fn no_violation_for_direct_json_usage() {
        let good_sources = vec![
            "$data | to json | save output.json",
            "$data | get field",
            "$records | each { |r| $r.name }",
            "$users | where age > 30",
            "^jq '.field' input.json",
        ];

        let rule = rule();

        for source in good_sources {
            LintContext::test_with_parsed_source(source, |context| {
                let violations = rule.check(&context);
                assert!(violations.is_empty(), "Should not trigger for: {source}");
            });
        }
    }

    #[test]
    fn suggests_idiomatic_alternatives() {
        let source = "$data | to json | ^jq '.field'";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            let violation = &violations[0];
            let suggestion = violation.suggestion.as_ref().unwrap();

            // Should suggest structured alternatives
            assert!(suggestion.contains("structured format"));
            assert!(suggestion.contains("get"));
            assert!(suggestion.contains("where"));
            assert!(suggestion.contains("each"));
        });
    }

    #[test]
    fn detects_in_nested_blocks() {
        let source = "if true { $data | to json | ^jq '.field' }";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect in nested blocks");
        });
    }
}
