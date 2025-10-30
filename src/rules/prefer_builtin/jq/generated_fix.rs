use super::rule;
use crate::context::LintContext;
#[test]
fn fix_jq_length() {
    let source = "^jq 'length' data.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.replacements[0].new_text.contains("length"));
            assert!(fix.replacements[0].new_text.contains("from json"));
        }
    });
}

#[test]
fn fix_jq_keys() {
    let source = "^jq 'keys' object.json";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.replacements[0].new_text.contains("columns"));
        }
    });
}
