use super::rule;

#[test]
fn detect_jq_length() {
    rule().assert_detects("^jq 'length' data.json");
}

#[test]
fn detect_jq_keys() {
    rule().assert_detects("^jq 'keys' object.json");
}

#[test]
fn detect_jq_type() {
    rule().assert_detects("^jq 'type' value.json");
}

#[test]
fn detect_jq_empty() {
    rule().assert_detects("^jq 'empty' file.json");
}

#[test]
fn detect_jq_not() {
    rule().assert_detects("^jq 'not' boolean.json");
}

#[test]
fn detect_jq_flatten() {
    rule().assert_detects("^jq 'flatten' nested.json");
}

#[test]
fn detect_jq_add() {
    rule().assert_detects("^jq 'add' numbers.json");
}

#[test]
fn detect_jq_min() {
    rule().assert_detects("^jq 'min' values.json");
}

#[test]
fn detect_jq_max() {
    rule().assert_detects("^jq 'max' values.json");
}

#[test]
fn detect_jq_array_index() {
    rule().assert_detects("^jq '.[0]' array.json");
}

#[test]
fn detect_jq_array_index_negative() {
    rule().assert_detects("^jq '.[-1]' array.json");
}

#[test]
fn detect_jq_array_index_various() {
    let bad_codes = vec![
        "^jq '.[1]' list.json",
        "^jq '.[42]' data.json",
        "^jq '.[100]' items.json",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_stdin_operations() {
    let bad_codes = vec![
        "^jq 'length'",
        "^jq 'keys'",
        "^jq 'type'",
        "^jq 'add'",
        "^jq 'min'",
        "^jq 'max'",
        "^jq 'flatten'",
        "^jq 'not'",
        "^jq 'empty'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_in_pipelines() {
    rule().assert_detects("cat data.json | ^jq 'length'");
    rule().assert_detects("curl -s api/data | ^jq 'keys'");
}

#[test]
fn detect_jq_in_functions() {
    let bad_code = r"
def count_items [file] {
    ^jq 'length' $file
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn detect_multiple_simple_jq_operations() {
    rule().assert_violation_count("^jq 'keys' data.json; ^jq 'length' data.json", 2);
    rule().assert_violation_count("^jq 'add' nums.json | ^jq 'type'", 2);
}

#[test]
fn detect_jq_in_complex_expressions() {
    let bad_code = r#"
if (^jq 'length' data.json) > 0 {
    print "has data"
}
"#;
    rule().assert_detects(bad_code);
}

#[test]
fn detect_to_json_then_jq_field_access() {
    rule().assert_detects("$data | to json | ^jq '.field'");
}

#[test]
fn detect_to_json_then_jq_complex() {
    rule().assert_detects("$records | to json | ^jq 'map(.name)'");
}

#[test]
fn detect_to_json_then_jq_filter() {
    rule().assert_detects("$items | to json | ^jq 'select(.active)'");
}

#[test]
fn detect_to_json_then_jq_array_ops() {
    let bad_codes = vec![
        "$numbers | to json | ^jq 'add'",
        "$values | to json | ^jq 'length'",
        "$items | to json | ^jq 'sort'",
        "$data | to json | ^jq 'unique'",
        "$nested | to json | ^jq 'flatten'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_to_json_then_jq_iteration() {
    let bad_codes = vec![
        "$data | to json | ^jq '.[]'",
        "$items | to json | ^jq '.users[]'",
        "$config | to json | ^jq '.services[] | .name'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_nested_to_json_jq() {
    rule().assert_detects("$data | select name != null | to json | ^jq '.[]'");
}

#[test]
fn detect_to_json_pipe_jq_with_args() {
    let bad_codes = vec![
        "$config | to json | ^jq -r '.database.host'",
        "$data | to json | ^jq -c '.users'",
        "$items | to json | ^jq -M '.products'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_to_json_multiline_jq() {
    rule().assert_detects(
        r#"$data | to json | ^jq '
        .users[]
        | select(.role == "admin")
        | .email
    '"#,
    );
}

#[test]
fn detect_to_json_jq_grouping() {
    rule().assert_detects("$records | to json | ^jq 'group_by(.category)'");
}

#[test]
fn detect_to_json_jq_sorting() {
    let bad_codes = vec![
        "$events | to json | ^jq 'sort_by(.timestamp)'",
        "$users | to json | ^jq 'sort_by(.name)'",
        "$items | to json | ^jq 'sort_by(.price) | reverse'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_to_json_jq_in_functions() {
    let bad_code = r"
def process_data [data] {
    $data | to json | ^jq '.items[] | .name'
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn detect_to_json_jq_in_nested_blocks() {
    let bad_code = r"
if $condition {
    $data | to json | ^jq '.result'
} else {
    null
}
";
    rule().assert_detects(bad_code);
}
