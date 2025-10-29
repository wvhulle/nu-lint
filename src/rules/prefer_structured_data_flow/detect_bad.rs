use super::rule;

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
