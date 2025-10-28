#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rules::prefer_structured_data_flow::rule};

    #[test]
    fn fix_to_json_then_jq_simple() {
        let source = "$data | to json | ^jq '.field'";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            let violation = &violations[0];
            assert!(violation.message.contains("Converting to JSON string"));
            assert!(
                violation
                    .suggestion
                    .as_ref()
                    .unwrap()
                    .contains("structured format")
            );
        });
    }

    #[test]
    fn fix_to_json_then_jq_complex() {
        let source = "$records | to json | ^jq 'map(.name)'";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            let violation = &violations[0];
            assert!(violation.suggestion.as_ref().unwrap().contains("where"));
            assert!(violation.suggestion.as_ref().unwrap().contains("each"));
        });
    }

    #[test]
    fn fix_spans_correctly() {
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
}
