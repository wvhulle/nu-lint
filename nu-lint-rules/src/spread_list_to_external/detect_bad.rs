use super::RULE;

#[test]
fn detect_list_var_to_external() {
    RULE.assert_detects(
        r#"
let items = ["a" "b"]
^cmd $items
"#,
    );
}

#[test]
fn detect_typed_list_param() {
    RULE.assert_detects(
        r#"
def foo [items: list<string>] {
    ^cmd $items
}
"#,
    );
}

#[test]
fn detect_list_literal_to_external() {
    RULE.assert_detects(r#"^cmd ["a" "b"]"#);
}

#[test]
fn detect_list_in_subexpression() {
    RULE.assert_detects(
        r#"
let items = ["a" "b"]
(^cmd $items)
"#,
    );
}

#[test]
fn detect_list_in_pipeline() {
    RULE.assert_detects(
        r#"
let items = ["a" "b"]
^cmd $items | lines
"#,
    );
}
