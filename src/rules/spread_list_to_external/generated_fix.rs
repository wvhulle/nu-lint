use super::RULE;

#[test]
fn fix_list_var() {
    RULE.assert_fixed_contains(
        r#"
let items = ["a" "b"]
^cmd $items
"#,
        "^cmd ...$items",
    );
}

#[test]
fn fix_typed_list_param() {
    RULE.assert_fixed_contains(
        r#"
def foo [items: list<string>] {
    ^cmd $items
}
"#,
        "^cmd ...$items",
    );
}

#[test]
fn fix_list_literal() {
    RULE.assert_fixed_contains(r#"^cmd ["a" "b"]"#, r#"^cmd ...["a" "b"]"#);
}

#[test]
fn fix_preserves_other_args() {
    RULE.assert_fixed_contains(
        r#"
let items = ["a" "b"]
^cmd --flag $items arg
"#,
        "...$items",
    );
}
