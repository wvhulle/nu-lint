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
