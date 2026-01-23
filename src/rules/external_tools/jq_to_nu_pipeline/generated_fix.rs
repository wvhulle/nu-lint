use super::RULE;

#[test]
fn fix_simple_functions() {
    RULE.assert_fixed_contains("^jq 'type' value.json", "describe");
    RULE.assert_fixed_contains("$list | to json | ^jq 'reverse'", "reverse");
}

#[test]
fn fix_math_functions() {
    RULE.assert_fixed_contains("$numbers | to json | ^jq 'add'", "math sum");
    RULE.assert_fixed_contains("$numbers | to json | ^jq 'min'", "math min");
    RULE.assert_fixed_contains("$numbers | to json | ^jq 'max'", "math max");
}

#[test]
fn fix_array_index() {
    RULE.assert_fixed_contains("^jq '.[0]' data.json", "get 0");
    RULE.assert_fixed_contains("^jq '.[-1]' items.json", "last");
}

#[test]
fn fix_field_access() {
    RULE.assert_fixed_contains("^jq '.name' user.json", "get name");
    RULE.assert_fixed_contains("^jq '.user.email' data.json", "get user.email");
}

#[test]
fn fix_array_iteration() {
    RULE.assert_fixed_contains("^jq '.[]' array.json", "each");
    RULE.assert_fixed_contains("^jq '.users[]' data.json", "get users | each");
}

#[test]
fn fix_functions_with_args() {
    RULE.assert_fixed_contains("$users | to json | ^jq 'map(.name)'", "get name");
    RULE.assert_fixed_contains(
        "$records | to json | ^jq 'group_by(.category)'",
        "group-by category",
    );
    RULE.assert_fixed_contains(
        "$events | to json | ^jq 'sort_by(.timestamp)'",
        "sort-by timestamp",
    );
}

#[test]
fn fix_pipeline_functions() {
    RULE.assert_fixed_contains("$items | to json | ^jq 'sort'", "sort");
    RULE.assert_fixed_contains("$data | to json | ^jq 'unique'", "uniq");
    RULE.assert_fixed_contains("$nested | to json | ^jq 'flatten'", "flatten");
}

#[test]
fn fix_preserves_file_context() {
    let source = "^jq '.name' /path/to/user.json";
    RULE.assert_fixed_contains(source, "open /path/to/user.json");
    RULE.assert_fixed_contains(source, "get name");
}

#[test]
fn fix_handles_piped_data() {
    let source = "$data | to json | ^jq '.name'";
    RULE.assert_fixed_contains(source, "get name");
}

#[test]
fn fix_interpolated_simple_field() {
    RULE.assert_fixed_contains(r#"^jq $".($field)" data.json"#, "get $field");
    RULE.assert_fixed_contains(r#"$data | to json | ^jq $".($key)""#, "get $key");
}

#[test]
fn fix_interpolated_with_prefix() {
    RULE.assert_fixed_contains(r#"^jq $".user.($field)" data.json"#, "get user.$field");
}

#[test]
fn fix_interpolated_index() {
    RULE.assert_fixed_contains(r#"^jq $".[($idx)]" array.json"#, "get $idx");
}

#[test]
fn fix_interpolated_field_then_index() {
    RULE.assert_fixed_contains(r#"^jq $".items[($idx)]" data.json"#, "get items | get $idx");
}

#[test]
fn fix_bracket_notation_with_dots() {
    // Keys containing dots must be quoted in Nushell
    RULE.assert_fixed_contains(
        r#"^jq '.["test.write"]' settings.json"#,
        r#"get "test.write""#,
    );
    RULE.assert_fixed_contains(
        r#"^jq '.["my.nested.key"]' config.json"#,
        r#"get "my.nested.key""#,
    );
}

#[test]
fn fix_bracket_notation_with_spaces() {
    RULE.assert_fixed_contains(
        r#"^jq '.["key with space"]' data.json"#,
        r#"get "key with space""#,
    );
}

#[test]
fn fix_mixed_field_and_bracket() {
    // Normal field followed by bracketed field with dot
    RULE.assert_fixed_contains(
        r#"^jq '.config["db.host"]' settings.json"#,
        r#"get config."db.host""#,
    );
}
