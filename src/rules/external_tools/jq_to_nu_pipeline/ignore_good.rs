use super::RULE;

#[test]
fn complex_jq_function_definitions() {
    RULE.assert_ignores("^jq 'def f: .; . | f' input.json");
}

#[test]
fn complex_jq_conditionals() {
    let cases = [
        "^jq 'if .status == \"ok\" then .data else empty end' response.json",
        "^jq 'if .x > 0 then .y elif .x < 0 then .z else .w end' data.json",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn complex_jq_arithmetic() {
    let cases = [
        "^jq '.items[] | select(.price * .quantity > 100)' catalog.json",
        "^jq '.[] | select((.a + .b) / 2 > 10)' data.json",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn complex_jq_string_operations() {
    let cases = [
        "^jq '.[] | select(.name | startswith(\"A\"))' people.json",
        "^jq '.[] | select(.email | contains(\"@example.com\"))' users.json",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn complex_jq_multiline_select() {
    RULE.assert_ignores(
        r#"$data | to json | ^jq '
        .users[]
        | select(.role == "admin")
        | .email
    '"#,
    );
}

#[test]
fn native_nu_simple_functions() {
    let cases = [
        "$data | length",
        "$record | columns",
        "$value | describe",
        "$nested | flatten",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn native_nu_math_operations() {
    let cases = [
        "$numbers | math sum",
        "$values | math min",
        "$scores | math max",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn native_nu_get_operations() {
    let cases = [
        "$array | get 0",
        "$data | get field",
        "$config | get database.host",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn native_nu_where_operations() {
    let cases = [
        "$data | where field == 'value'",
        "$items | where active",
        "$users | where age > 30",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn native_nu_grouping_and_sorting() {
    let cases = [
        "$data | group-by category",
        "$items | sort-by price",
        "$events | sort-by timestamp",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn native_nu_each_operations() {
    let cases = [
        "$users | each { get name }",
        "$items | each { |item| $item.price * 1.1 }",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn native_nu_complex_pipelines() {
    let cases = [
        "$data | where active | get name | sort",
        "$users | each { |u| $u.name } | uniq",
        "$items | where price > 100 | sort-by name",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn from_json_operations() {
    let cases = [
        "$json_string | from json",
        "open data.json | from json",
        "http get api/data | from json",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn to_json_for_output() {
    let cases = [
        "$data | to json",
        "$data | to json | save output.json",
        "$config | to json | save settings.json",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn to_json_for_other_external_tools() {
    let cases = [
        "$data | to json | ^curl -d @- api.example.com",
        "$config | to json | ^http POST api/settings",
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}

#[test]
fn jq_with_complex_interpolated_filter() {
    // Complex interpolations we can't convert
    let cases = [
        // Function call with interpolation
        r#"^jq $'select(.($predicate))' items.json"#,
        // Full dynamic filter (no static prefix)
        r#"let filter = ".name"; ^jq $"($filter)" data.json"#,
        // Multiple dynamic parts
        r#"^jq $".($a).($b)" data.json"#,
        // Nested field access with dynamic
        r#"^jq $".($field).nested" data.json"#,
    ];
    for code in cases {
        RULE.assert_ignores(code);
    }
}
