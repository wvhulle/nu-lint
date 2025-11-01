use super::rule;
use crate::context::LintContext;

#[test]
fn fix_get_field_operation() {
    let source = "def get-field [field] { $in | get $field }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(
                fix.description
                    .contains("Change to: def get-field [field] { get $field }")
            );
            assert!(fix.description.contains("Remove redundant $in"));
        }
    });
}

#[test]
fn fix_select_operation() {
    let source = "def select-column [column] { $in | select $column }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(
                fix.description
                    .contains("Change to: def select-column [column] { select $column }")
            );
            assert!(fix.description.contains("Remove redundant $in"));
        }
    });
}

#[test]
fn fix_each_operation() {
    let source = "def multiply [factor] { $in | each { |x| $x * $factor } }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(
                fix.description
                    .contains("Change to: def multiply [factor] { each { |x| $x * $factor } }")
            );
            assert!(fix.description.contains("Remove redundant $in"));
        }
    });
}

#[test]
fn fix_no_parameters() {
    let source = "def process [] { $in | where active }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(
                fix.description
                    .contains("Change to: def process [] { where active }")
            );
            assert!(fix.description.contains("Remove redundant $in"));
        }
    });
}

#[test]
fn fix_complex_pipeline() {
    let source = "def process [] { $in | where active | select name | sort-by name }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains(
                "Change to: def process [] { where active | select name | sort-by name }"
            ));
            assert!(fix.description.contains("Remove redundant $in"));
        }
    });
}

#[test]
fn fix_no_space_after_in() {
    let source = "def filter [] { $in| where $it > 0 }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(
                fix.description
                    .contains("Change to: def filter [] { where $it > 0 }")
            );
            assert!(fix.description.contains("Remove redundant $in"));
        }
    });
}

#[test]
fn fix_sort_by_operation() {
    let source = "def sort-by-field [field] { $in | sort-by $field }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(
                fix.description
                    .contains("Change to: def sort-by-field [field] { sort-by $field }")
            );
            assert!(fix.description.contains("Remove redundant $in"));
        }
    });
}

#[test]
fn fix_first_operation() {
    let source = "def take-first [n] { $in | first $n }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(
                fix.description
                    .contains("Change to: def take-first [n] { first $n }")
            );
            assert!(fix.description.contains("Remove redundant $in"));
        }
    });
}
