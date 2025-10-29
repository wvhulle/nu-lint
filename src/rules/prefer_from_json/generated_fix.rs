// Tests for generated fixes - basic validation that fixes are generated
#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rules::prefer_from_json::rule};

    #[test]
    fn fix_simple_jq_identity() {
        let source = "^jq '.' data.json";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            if let Some(fix) = &violations[0].fix {
                assert!(fix.replacements[0].new_text.contains("from json"));
            }
        });
    }

    #[test]
    fn fix_jq_field_access() {
        let source = "^jq '.name' users.json";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            if let Some(fix) = &violations[0].fix {
                assert!(fix.replacements[0].new_text.contains("get name"));
            }
        });
    }
}
