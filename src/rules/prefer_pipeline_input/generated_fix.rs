use super::rule;
use crate::context::LintContext;

#[test]
fn fix_simple_each_operation() {
    let source = "def process-items [items] { $items | each { |x| $x * 2 } }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains("Change to: def process-items [] { each { |x| $x * 2 } }"));
            assert!(fix.description.contains("Remove the 'items' parameter and use pipeline input"));
        }
    });
}

#[test]
fn fix_where_operation() {
    let source = "def filter-positive [numbers] { $numbers | where $it > 0 }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains("Change to: def filter-positive [] { where $it > 0 }"));
            assert!(fix.description.contains("Remove the 'numbers' parameter and use pipeline input"));
        }
    });
}

#[test]
fn fix_select_operation() {
    let source = "def get-names [records] { $records | select name }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains("Change to: def get-names [] { select name }"));
            assert!(fix.description.contains("Remove the 'records' parameter and use pipeline input"));
        }
    });
}

#[test]
fn fix_sort_by_operation() {
    let source = "def sort-by-name [items] { $items | sort-by name }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains("Change to: def sort-by-name [] { sort-by name }"));
            assert!(fix.description.contains("Remove the 'items' parameter and use pipeline input"));
        }
    });
}

#[test]
fn fix_group_by_operation() {
    let source = "def group-items [data] { $data | group-by category }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains("Change to: def group-items [] { group-by category }"));
            assert!(fix.description.contains("Remove the 'data' parameter and use pipeline input"));
        }
    });
}

#[test]
fn fix_reduce_operation() {
    let source = "def sum-values [numbers] { $numbers | reduce { |acc, val| $acc + $val } }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains("Change to: def sum-values [] { reduce { |acc, val| $acc + $val } }"));
            assert!(fix.description.contains("Remove the 'numbers' parameter and use pipeline input"));
        }
    });
}

#[test]
fn fix_multiple_pipeline_operations() {
    let source = "def process [data] { $data | where active | select name | sort-by name }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains("Change to: def process [] { where active | select name | sort-by name }"));
            assert!(fix.description.contains("Remove the 'data' parameter and use pipeline input"));
        }
    });
}

#[test]
fn fix_math_operations() {
    let source = "def sum-all [numbers] { $numbers | math sum }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains("Change to: def sum-all [] { math sum }"));
            assert!(fix.description.contains("Remove the 'numbers' parameter and use pipeline input"));
        }
    });
}

#[test]
fn fix_length_operation() {
    let source = "def count-items [data] { $data | length }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains("Change to: def count-items [] { length }"));
            assert!(fix.description.contains("Remove the 'data' parameter and use pipeline input"));
        }
    });
}

#[test]
fn fix_typed_list_parameter() {
    let source = "def process-list [items: list] { $items | each { |x| $x + 1 } }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains("Change to: def process-list [] { each { |x| $x + 1 } }"));
            assert!(fix.description.contains("Remove the 'items' parameter and use pipeline input"));
        }
    });
}

#[test]
fn fix_typed_table_parameter() {
    let source = "def process-table [data: table] { $data | select name age }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains("Change to: def process-table [] { select name age }"));
            assert!(fix.description.contains("Remove the 'data' parameter and use pipeline input"));
        }
    });
}

#[test]
fn fix_string_data_processing() {
    let source = "def split-lines [text] { $text | lines }";
    let rule = rule();

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule.check(&context);
        assert!(!violations.is_empty());

        if let Some(fix) = &violations[0].fix {
            assert!(fix.description.contains("Change to: def split-lines [] { lines }"));
            assert!(fix.description.contains("Remove the 'text' parameter and use pipeline input"));
        }
    });
}

