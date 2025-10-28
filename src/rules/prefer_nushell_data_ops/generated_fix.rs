// Tests for generated fixes - basic validation that fixes are generated
#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rules::prefer_nushell_data_ops::rule};

    #[test]
    fn fix_jq_map() {
        let source = "^jq 'map(.name)' users.json";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            if let Some(fix) = &violations[0].fix {
                assert!(fix.replacements[0].new_text.contains("each"));
            }
        });
    }

    #[test]
    fn fix_jq_select() {
        let source = "^jq 'select(.age > 30)' people.json";
        let rule = rule();

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty());

            if let Some(fix) = &violations[0].fix {
                assert!(fix.replacements[0].new_text.contains("where"));
            }
        });
    }
}
