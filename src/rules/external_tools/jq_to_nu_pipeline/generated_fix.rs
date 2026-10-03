use super::RULE;

#[test]
fn fix_field_access_on_file() {
    RULE.assert_fixed_is(
        "^jq '.name' user.json",
        "open --raw user.json | from json | get name",
    );
}

#[test]
fn fix_nested_field_access() {
    RULE.assert_fixed_is(
        "^jq -r '.user.email' data.json",
        "open --raw data.json | from json | get user.email",
    );
}

#[test]
fn fix_index_access() {
    RULE.assert_fixed_is(
        "^jq '.[0]' items.json",
        "open --raw items.json | from json | get 0",
    );
    RULE.assert_fixed_is(
        "^jq '.[-1]' items.json",
        "open --raw items.json | from json | last",
    );
}

#[test]
fn fix_removes_serialization() {
    RULE.assert_fixed_is("$data | to json | ^jq '.name'", "$data | get name");
    RULE.assert_fixed_is("$list | to json | ^jq 'reverse'", "$list | reverse");
}

#[test]
fn fix_functions_with_field_argument() {
    RULE.assert_fixed_is("$users | to json | ^jq 'map(.name)'", "$users | get name");
    RULE.assert_fixed_is(
        "$events | to json | ^jq 'sort_by(.timestamp)'",
        "$events | sort-by timestamp",
    );
}

#[test]
fn fix_math_functions() {
    RULE.assert_fixed_is("$numbers | to json | ^jq 'min'", "$numbers | math min");
    RULE.assert_fixed_is("$numbers | to json | ^jq 'max'", "$numbers | math max");
}

#[test]
fn fix_parses_external_output() {
    RULE.assert_fixed_is(
        "^curl -s https://example.com/api | ^jq '.name'",
        "^curl -s https://example.com/api | from json | get name",
    );
}

#[test]
fn fix_keeps_quoted_file() {
    RULE.assert_fixed_is(
        r#"^jq 'sort' "my data.json""#,
        r#"open --raw "my data.json" | from json | sort"#,
    );
}

#[test]
fn fix_interpolated_field() {
    RULE.assert_fixed_is(
        r#"let field = "a"; ^jq $".($field)" data.json"#,
        r#"let field = "a"; open --raw data.json | from json | get $field"#,
    );
}

#[test]
fn fix_interpolated_field_then_index() {
    RULE.assert_fixed_is(
        r#"let idx = 1; ^jq $".items[($idx)]" data.json"#,
        r#"let idx = 1; open --raw data.json | from json | get items | get $idx"#,
    );
}

#[test]
fn fix_bracket_notation_with_dots() {
    RULE.assert_fixed_is(
        r#"^jq '.config["db.host"]' settings.json"#,
        r#"open --raw settings.json | from json | get config."db.host""#,
    );
}
