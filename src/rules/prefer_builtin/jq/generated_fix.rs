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

#[test]
fn fix_to_json_then_jq_add() {
    let source = "$numbers | to json | ^jq 'add'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.replacements[0].new_text.contains("math sum"));
        }
    });
}

#[test]
fn fix_to_json_then_jq_length() {
    let source = "$values | to json | ^jq 'length'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.replacements[0].new_text.contains("length"));
        }
    });
}

#[test]
fn fix_to_json_then_jq_sort() {
    let source = "$items | to json | ^jq 'sort'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.replacements[0].new_text.contains("sort"));
        }
    });
}

#[test]
fn fix_to_json_then_jq_unique() {
    let source = "$data | to json | ^jq 'unique'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.replacements[0].new_text.contains("uniq"));
        }
    });
}

#[test]
fn fix_to_json_then_jq_flatten() {
    let source = "$nested | to json | ^jq 'flatten'";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.replacements[0].new_text.contains("flatten"));
        }
    });
}
