use super::rule;

#[test]
fn ignore_builtin_open() {
    rule().assert_ignores("open file.txt");
}

#[test]
fn ignore_open_raw() {
    rule().assert_ignores("open --raw file.txt");
}

#[test]
fn ignore_open_structured() {
    let good_codes = vec![
        "open data.json",
        "open config.toml",
        "open data.csv",
        "open settings.yaml",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_open_with_lines() {
    rule().assert_ignores("open file.txt | lines");
}

#[test]
fn ignore_open_enumerate() {
    rule().assert_ignores("open --raw file.txt | lines | enumerate");
}

#[test]
fn ignore_open_with_processing() {
    let good_codes = vec![
        "open --raw file.txt | lines | where $it != \"\"",
        "open file.json | get data",
        "open --raw log.txt | str trim",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_other_commands() {
    let good_codes = vec![
        "ls | where size > 1kb",
        "http get api/endpoint",
        "from json",
        "to json",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_variable_references() {
    rule().assert_ignores("$content | lines");
}

#[test]
fn ignore_string_operations() {
    rule().assert_ignores(r#""file content" | lines"#);
}
