use super::rule;

#[test]
fn detect_jq_map() {
    let bad_codes = vec![
        "^jq 'map(.name)' users.json",
        "^jq 'map(.price)' items.json",
        "^jq 'map(.id)' records.json",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_select() {
    let bad_codes = vec![
        "^jq 'select(.age > 30)' people.json",
        "^jq 'select(.active)' users.json",
        "^jq 'select(.status == \"published\")' posts.json",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_group_by() {
    let bad_codes = vec![
        "^jq 'group_by(.category)' items.json",
        "^jq 'group_by(.department)' employees.json",
        "^jq 'group_by(.date)' events.json",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_array_iteration() {
    let bad_codes = vec![
        "^jq '.[]' data.json",
        "^jq '.users[]' config.json",
        "^jq '.items[] | .name' catalog.json",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_sorting_operations() {
    let bad_codes = vec![
        "^jq 'sort_by(.timestamp)' events.json",
        "^jq 'sort_by(.name)' users.json",
        "^jq 'sort_by(.price)' products.json",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_unique() {
    rule().assert_detects("^jq 'unique' values.json");
}

#[test]
fn detect_jq_reverse() {
    rule().assert_detects("^jq 'reverse' list.json");
}

#[test]
fn detect_complex_jq_filtering_chains() {
    let bad_codes = vec![
        "^jq '.[] | select(.status == \"active\")' data.json",
        "^jq 'map(select(.published))' posts.json",
        "^jq '.users[] | select(.role == \"admin\")' config.json",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_complex_jq_mapping_chains() {
    let bad_codes = vec![
        "^jq 'map(.name) | unique' users.json",
        "^jq '.[] | map(.id)' collections.json",
        "^jq 'map(select(.active)) | sort_by(.name)' items.json",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_stdin_operations() {
    let bad_codes = vec![
        "^jq 'map(.field)'",
        "^jq 'select(.active)'",
        "^jq 'group_by(.type)'",
        "^jq '.[]'",
        "^jq 'unique'",
        "^jq 'reverse'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_in_pipelines() {
    let bad_codes = vec![
        "cat data.json | ^jq 'map(.name)'",
        "curl -s api/users | ^jq 'select(.active)'",
    ];

    for code in bad_codes {
        rule().assert_detects(code);
    }
}

#[test]
fn detect_jq_in_functions() {
    let bad_code = r"
def filter_active [file] {
    ^jq 'map(select(.active))' $file
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn detect_multiple_jq_data_ops() {
    rule().assert_violation_count(
        "^jq 'map(.name)' users.json; ^jq 'select(.active)' items.json",
        2,
    );
}
