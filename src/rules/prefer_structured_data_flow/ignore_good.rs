use super::rule;

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

#[test]
fn ignore_direct_jq_on_file() {
    let good_codes = vec![
        "^jq '.field' data.json",
        "^jq '.users[] | .name' people.json",
        "^jq 'map(.price)' items.json",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
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
fn ignore_separate_operations() {
    rule().assert_ignores(
        r"
        $data | to json | save temp.json
        ^jq '.field' temp.json
    ",
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
