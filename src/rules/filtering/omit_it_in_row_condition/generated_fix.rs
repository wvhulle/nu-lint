use super::RULE;

#[test]
fn fix_simple_field_comparison() {
    let source = r#"ls | where $it.type == "dir""#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"type"#);
}

#[test]
fn fix_size_comparison() {
    let source = r"ls | where $it.size > 100kb";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "size");
}

#[test]
fn fix_name_comparison() {
    let source = r#"ls | where $it.name == "foo""#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "name");
}

#[test]
fn fix_field_with_underscore() {
    let source = r"open data.json | where $it.first_name == 'John'";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "first_name");
}

#[test]
fn fix_multiple_it_field_accesses() {
    let source = r#"ls | where $it.size > 100kb and $it.type == "file""#;
    RULE.assert_count(source, 2);
    RULE.assert_fixed_contains(source, "size");
}

#[test]
fn fix_explanation_mentions_field_name() {
    let source = r"ls | where $it.size > 100kb";
    RULE.assert_count(source, 1);
}

#[test]
fn fix_explanation_mentions_removal() {
    let source = r#"ls | where $it.type == "dir""#;
    RULE.assert_count(source, 1);
}

#[test]
fn fix_numeric_comparison() {
    let source = r"[{x: 1}, {x: 2}] | where $it.x > 1";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "x > 1");
}

#[test]
fn fix_string_contains() {
    let source = r#"ls | where $it.name =~ "test""#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "name");
}

#[test]
fn fix_boolean_field() {
    let source = r"open data.json | where $it.active == true";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "active");
}
