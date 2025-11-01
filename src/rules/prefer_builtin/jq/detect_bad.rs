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

// Test cases for JSON filtering that Nushell can handle

#[test]
fn detect_jq_simple_field_access() {
    // .name -> get name
    let bad_codes = vec![
        "^jq '.name' user.json",
        "^jq '.email' contact.json",
        "^jq '.id' record.json",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_nested_field_access() {
    // .database.host -> get database.host
    let bad_codes = vec![
        "^jq '.data.items' response.json",
        "^jq '.config.version' settings.json",
        "^jq '.user.profile.name' data.json",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_array_iteration_all() {
    // .[] -> each (when used with open)
    let bad_codes = vec!["^jq '.[]' array.json", "$data | to json | ^jq '.[]'"];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_field_array_iteration() {
    // .users[] -> get users (with further processing)
    let bad_codes = vec![
        "^jq '.users[]' data.json",
        "^jq '.items[]' catalog.json",
        "$data | to json | ^jq '.products[]'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_map_simple() {
    // map(.name) -> get name (for tables)
    let bad_codes = vec![
        "^jq 'map(.name)' users.json",
        "^jq 'map(.id)' items.json",
        "$data | to json | ^jq 'map(.email)'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_group_by_simple() {
    // group_by(.category) -> group-by category
    let bad_codes = vec![
        "^jq 'group_by(.category)' items.json",
        "^jq 'group_by(.status)' tasks.json",
        "$events | to json | ^jq 'group_by(.type)'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_sort_by_simple() {
    // sort_by(.timestamp) -> sort-by timestamp
    let bad_codes = vec![
        "^jq 'sort_by(.timestamp)' events.json",
        "^jq 'sort_by(.name)' users.json",
        "^jq 'sort_by(.priority)' tasks.json",
        "$items | to json | ^jq 'sort_by(.price)'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_select_simple_field() {
    // select(.active) -> where active (simple field check)
    let bad_codes = vec![
        "^jq 'select(.active)' users.json",
        "^jq 'select(.enabled)' features.json",
        "$data | to json | ^jq 'select(.published)'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_with_file_operations() {
    // Common patterns where jq reads from file but Nushell can do it directly
    let bad_codes = vec![
        "^jq '.data' response.json",
        "^jq '.config.settings' app.json",
        "^jq 'length' items.json",
        "^jq 'keys' object.json",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_chained_simple_operations() {
    // Chains that can be converted: .users[] | .name
    let bad_codes = vec![
        "$data | to json | ^jq '.users[] | .name'",
        "$data | to json | ^jq '.items[] | .id'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_with_flags() {
    // jq with flags like -r, -c, -M should still be detected if the filter is
    // simple
    let bad_codes = vec![
        "$config | to json | ^jq -r '.database.host'",
        "$data | to json | ^jq -c '.users'",
        "$items | to json | ^jq -M '.products'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}
