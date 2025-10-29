use super::rule;

#[test]
fn ignore_nushell_where() {
    let good_codes = vec![
        "$data | where age > 30",
        "$items | where active",
        "$users | where role == 'admin'",
        "$posts | where published",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_nushell_each() {
    let good_codes = vec![
        "$items | each { get name }",
        "$users | each { |u| $u.email }",
        "$files | each { |f| $f.size }",
        "$data | each { |item| $item.price * 1.1 }",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_nushell_group_by() {
    let good_codes = vec![
        "$data | group-by category",
        "$employees | group-by department",
        "$events | group-by date",
        "$products | group-by brand",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_nushell_sort_by() {
    let good_codes = vec![
        "$events | sort-by timestamp",
        "$users | sort-by name",
        "$items | sort-by price",
        "$products | sort-by category",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_nushell_data_operations() {
    let good_codes = vec![
        "$record | values",
        "$data | uniq",
        "$list | reverse",
        "$items | flatten",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_simple_jq_operations() {
    let good_codes = vec![
        "^jq '.' data.json",
        "^jq '.name' user.json",
        "^jq '.config.database' settings.json",
        "^jq '.[0]' array.json",
        "^jq 'length' data.json",
        "^jq 'keys' object.json",
        "^jq 'type' value.json",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_complex_jq_without_data_ops() {
    let good_codes = vec![
        "^jq '.users[0].profile.name' config.json",
        "^jq 'if .status == \"ok\" then .data else empty end' response.json",
        "^jq 'def f: .; . | f' input.json",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_other_external_commands() {
    let good_codes = vec![
        "^grep pattern file.txt",
        "^curl -s api.example.com",
        "^git status",
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
        "open file.json | from json | where active | each { get name }",
        "$data | where price > 100 | sort-by name | each { |item| $item.title }",
        "ls *.json | each { |f| open $f.name | from json } | flatten",
        "$users | group-by department | each { |dept| $dept | where active | length }",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_mixed_valid_operations() {
    let good_codes = vec![
        "open data.json | from json | where active",
        "$data | select name email | sort-by name",
        "http get api/users | where role == 'admin'",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_nushell_utility_commands() {
    let good_codes = vec![
        "$data | length",
        "$items | first 10",
        "$list | last 5",
        "$records | skip 2",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}
