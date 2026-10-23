use super::RULE;

#[test]
fn ignore_spread_list() {
    RULE.assert_ignores(
        r#"
let items = ["a" "b"]
^cmd ...$items
"#,
    );
}

#[test]
fn ignore_string_var() {
    RULE.assert_ignores(
        r#"
let name = "test"
^cmd $name
"#,
    );
}

#[test]
fn ignore_int_var() {
    RULE.assert_ignores(
        r#"
let count = 5
^cmd $count
"#,
    );
}

#[test]
fn ignore_builtin_command_with_list() {
    // Builtin commands can handle lists directly
    RULE.assert_ignores(
        r#"
let items = ["a" "b"]
echo $items
"#,
    );
}

#[test]
fn ignore_spread_list_literal() {
    RULE.assert_ignores(r#"^cmd ...["a" "b"]"#);
}

#[test]
fn ignore_typed_string_param() {
    RULE.assert_ignores(
        r#"
def foo [name: string] {
    ^cmd $name
}
"#,
    );
}
