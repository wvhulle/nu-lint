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
fn ignore_complex_jq_operations() {
    let good_codes = vec![
        "^jq '.[] | select(.age > 30)' people.json",
        "^jq 'map(.name)' users.json",
        "^jq 'group_by(.category)' items.json",
        "^jq '.users[] | .name' data.json",
        "^jq 'sort_by(.timestamp)' events.json",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_jq_field_access() {
    let good_codes = vec![
        "^jq '.name' user.json",
        "^jq '.data.items' response.json",
        "^jq '.config.version' settings.json",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_other_external_commands() {
    let good_codes = vec![
        "^curl -s api.example.com",
        "^git status",
        "^docker ps",
        "^sed 's/old/new/g' file.txt",
        "^awk '{print $1}' data.txt",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_proper_nushell_pipelines() {
    let good_codes = vec![
        "open file.json | from json | length",
        "open data.json | from json | get name",
        "ls *.json | where size > 1KB",
        "$data | where active | length",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_jq_with_complex_filters() {
    let good_codes = vec![
        "^jq '.items[] | select(.price > 100) | .name' catalog.json",
        "^jq 'def f: .; . | f' input.json",
        "^jq 'if .status == \"ok\" then .data else empty end' response.json",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}
