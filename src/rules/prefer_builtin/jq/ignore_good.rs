use super::rule;

#[test]
fn ignore_nushell_length() {
    rule().assert_ignores("$data | length");
}

#[test]
fn ignore_nushell_columns() {
    rule().assert_ignores("$record | columns");
}

#[test]
fn ignore_nushell_describe() {
    rule().assert_ignores("$value | describe");
}

#[test]
fn ignore_nushell_flatten() {
    rule().assert_ignores("$nested | flatten");
}

#[test]
fn ignore_nushell_math_operations() {
    let good_codes = vec![
        "$numbers | math sum",
        "$values | math min",
        "$scores | math max",
        "$items | math avg",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_nushell_get_operations() {
    let good_codes = vec![
        "$array | get 0",
        "$list | get 1",
        "$data | get field",
        "$record | get name",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_jq_with_function_definitions() {
    // jq allows defining custom functions - no direct Nushell equivalent in jq
    // syntax
    rule().assert_ignores("^jq 'def f: .; . | f' input.json");
}

#[test]
fn ignore_jq_with_complex_conditionals() {
    // Complex conditional logic with multiple branches
    let good_codes = vec![
        "^jq 'if .status == \"ok\" then .data else empty end' response.json",
        "^jq 'if .x > 0 then .y elif .x < 0 then .z else .w end' data.json",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_jq_with_arithmetic_in_filters() {
    // Complex arithmetic operations in filter expressions
    let good_codes = vec![
        "^jq '.items[] | select(.price * .quantity > 100)' catalog.json",
        "^jq '.[] | select((.a + .b) / 2 > 10)' data.json",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_jq_with_string_operations_in_select() {
    // String operations inside select predicates
    let good_codes = vec![
        "^jq '.[] | select(.name | startswith(\"A\"))' people.json",
        "^jq '.[] | select(.email | contains(\"@example.com\"))' users.json",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_jq_multiline_with_complex_select() {
    // Multiline jq with complex select condition (comparison)
    rule().assert_ignores(
        r#"$data | to json | ^jq '
        .users[]
        | select(.role == "admin")
        | .email
    '"#,
    );
}

#[test]
fn ignore_proper_structured_pipelines() {
    let good_codes = vec![
        "$data | where active | get name | sort",
        "$users | each { |u| $u.name } | uniq",
        "$items | where price > 100 | sort-by name",
        "$events | group-by date | each { |group| $group | length }",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_mixed_valid_operations() {
    let good_codes = vec![
        "open data.json | from json | where active",
        "$data | select name age | to json",
        "ls *.json | each { |f| open $f.name | from json }",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_from_json() {
    let good_codes = vec![
        "$json_string | from json",
        "open data.json | from json",
        "http get api/data | from json",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_to_json_for_external_tools() {
    let good_codes = vec![
        "$data | to json | ^curl -d @- api.example.com",
        "$config | to json | ^http POST api/settings",
        "$payload | to json | ^wget --post-data=@- url",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_to_json_standalone() {
    rule().assert_ignores("$data | to json");
}

#[test]
fn ignore_to_json_for_output() {
    let good_codes = vec![
        "$data | to json | save output.json",
        "$config | to json | save settings.json",
        "$results | to json | save report.json",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_structured_data_operations() {
    let good_codes = vec![
        "$data | where field == 'value'",
        "$items | where active",
        "$users | where age > 30",
        "$records | where category == 'important'",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_structured_data_get() {
    let good_codes = vec![
        "$config | get database.host",
        "$data | get name",
        "$response | get data.items",
        "$user | get profile.email",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_structured_data_each() {
    let good_codes = vec![
        "$users | each { get name }",
        "$items | each { |item| $item.price * 1.1 }",
        "$files | each { |f| $f.name }",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_structured_data_grouping() {
    let good_codes = vec![
        "$data | group-by category",
        "$users | group-by department",
        "$events | group-by date",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_structured_data_sorting() {
    let good_codes = vec![
        "$items | sort-by price",
        "$users | sort-by name",
        "$events | sort-by timestamp",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}
