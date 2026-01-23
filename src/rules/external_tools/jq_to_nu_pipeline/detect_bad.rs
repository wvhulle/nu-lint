use super::RULE;

#[test]
fn simple_functions_with_file() {
    // Note: length and keys are NOT converted due to semantic differences
    // jq length/keys work on objects AND arrays, Nu equivalents don't
    let cases = [
        "^jq 'type' value.json",
        "^jq 'not' boolean.json",
        "^jq 'flatten' nested.json",
        "^jq 'add' numbers.json",
        "^jq 'min' values.json",
        "^jq 'max' values.json",
        "^jq 'sort' items.json",
        "^jq 'unique' list.json",
        "^jq 'reverse' array.json",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn simple_functions_from_stdin() {
    let cases = [
        "^jq 'type'",
        "^jq 'add'",
        "^jq 'min'",
        "^jq 'max'",
        "^jq 'flatten'",
        "^jq 'sort'",
        "^jq 'unique'",
        "^jq 'reverse'",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn simple_functions_from_pipeline() {
    let cases = [
        "$numbers | to json | ^jq 'add'",
        "$items | to json | ^jq 'sort'",
        "$data | to json | ^jq 'unique'",
        "$nested | to json | ^jq 'flatten'",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn array_index_access() {
    let cases = [
        "^jq '.[0]' array.json",
        "^jq '.[1]' list.json",
        "^jq '.[42]' data.json",
        "^jq '.[-1]' array.json",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn single_field_access() {
    let cases = [
        "^jq '.name' user.json",
        "^jq '.email' contact.json",
        "^jq '.id' record.json",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn nested_field_access() {
    let cases = [
        "^jq '.data.items' response.json",
        "^jq '.config.version' settings.json",
        "^jq '.user.profile.name' data.json",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn field_access_from_pipeline() {
    RULE.assert_detects("$data | to json | ^jq '.field'");
    RULE.assert_detects("$config | to json | ^jq -r '.database.host'");
}

#[test]
fn array_iteration() {
    let cases = ["^jq '.[]' array.json", "$data | to json | ^jq '.[]'"];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn field_then_iteration() {
    let cases = [
        "^jq '.users[]' data.json",
        "^jq '.items[]' catalog.json",
        "$data | to json | ^jq '.products[]'",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn map_function() {
    let cases = [
        "^jq 'map(.name)' users.json",
        "^jq 'map(.id)' items.json",
        "$data | to json | ^jq 'map(.email)'",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn select_function() {
    let cases = [
        "^jq 'select(.active)' users.json",
        "^jq 'select(.enabled)' features.json",
        "$data | to json | ^jq 'select(.published)'",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn group_by_function() {
    let cases = [
        "^jq 'group_by(.category)' items.json",
        "^jq 'group_by(.status)' tasks.json",
        "$events | to json | ^jq 'group_by(.type)'",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn sort_by_function() {
    let cases = [
        "^jq 'sort_by(.timestamp)' events.json",
        "^jq 'sort_by(.name)' users.json",
        "$items | to json | ^jq 'sort_by(.price)'",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn jq_piped_with_simple_chains() {
    let cases = [
        "$data | to json | ^jq '.users[] | .name'",
        "$data | to json | ^jq '.items[] | .id'",
        "$config | to json | ^jq '.services[] | .name'",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn jq_with_flags() {
    let cases = [
        "$config | to json | ^jq -r '.database.host'",
        "$data | to json | ^jq -c '.users'",
        "$items | to json | ^jq -M '.products'",
    ];
    for code in cases {
        RULE.assert_detects(code);
    }
}

#[test]
fn jq_in_pipeline() {
    RULE.assert_detects("cat data.json | ^jq 'sort'");
    RULE.assert_detects("curl -s api/data | ^jq '.name'");
}

#[test]
fn jq_in_function() {
    let code = r"
def get_type [file] {
    ^jq 'type' $file
}
";
    RULE.assert_detects(code);
}

#[test]
fn jq_in_conditional() {
    let code = r#"
if (^jq '.active' data.json) {
    print "is active"
}
"#;
    RULE.assert_detects(code);
}

#[test]
fn jq_in_nested_block() {
    let code = r"
if $condition {
    $data | to json | ^jq '.result'
} else {
    null
}
";
    RULE.assert_detects(code);
}

#[test]
fn multiple_jq_calls() {
    RULE.assert_count("^jq 'sort' data.json; ^jq 'reverse' data.json", 2);
    RULE.assert_count("^jq 'add' nums.json | ^jq 'type'", 2);
}

#[test]
fn interpolated_simple_field() {
    // $".($var)" pattern
    RULE.assert_detects(r#"^jq $".($field)" data.json"#);
    RULE.assert_detects(r#"$data | to json | ^jq $".($key)""#);
}

#[test]
fn interpolated_with_static_prefix() {
    // $".prefix.($var)" pattern
    RULE.assert_detects(r#"^jq $".user.($field)" data.json"#);
    RULE.assert_detects(r#"^jq $".config.database.($key)" settings.json"#);
}

#[test]
fn interpolated_index() {
    // $".[($idx)]" pattern
    RULE.assert_detects(r#"^jq $".[($idx)]" array.json"#);
    RULE.assert_detects(r#"$data | to json | ^jq $".[($i)]""#);
}

#[test]
fn interpolated_field_then_index() {
    // $".field[($idx)]" pattern
    RULE.assert_detects(r#"^jq $".items[($idx)]" data.json"#);
    RULE.assert_detects(r#"^jq $".users[($i)]" people.json"#);
}

#[test]
fn bracket_notation_with_special_chars() {
    // Keys containing dots require bracket notation in jq
    RULE.assert_detects(r#"^jq '.["test.write"]' settings.json"#);
    RULE.assert_detects(r#"^jq '.["my.nested.key"]' config.json"#);
    // Keys containing spaces
    RULE.assert_detects(r#"^jq '.["key with space"]' data.json"#);
    // Mixed: normal field then bracketed field with dot
    RULE.assert_detects(r#"^jq '.config["db.host"]' settings.json"#);
}
