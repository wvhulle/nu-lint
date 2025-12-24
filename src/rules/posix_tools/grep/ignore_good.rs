use super::RULE;

#[test]
fn ignore_nushell_find() {
    RULE.assert_ignores(r#"find "pattern""#);
}

#[test]
fn ignore_nushell_where() {
    RULE.assert_ignores(r#"lines | where $it =~ "pattern""#);
}

#[test]
fn ignore_nushell_where_with_field() {
    RULE.assert_ignores(r#"ls | where name =~ "pattern""#);
}

#[test]
fn ignore_lines_where_pipeline() {
    let good_codes = vec![
        r#"open file.txt | lines | where $it =~ "error""#,
        r#"cat log.txt | lines | where $it =~ "warning""#,
        r#"$content | lines | where $it =~ "TODO""#,
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_structured_data_where() {
    let good_codes = vec![
        r#"ls | where name =~ "test""#,
        r#"$users | where email =~ "@example.com""#,
        r#"open data.json | where status == "active""#,
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_regex_operators() {
    let good_codes = vec![
        r#"$items | where $it =~ "pattern""#,
        r#"$items | where $it !~ "pattern""#,
        r#"lines | where $it =~ "[0-9]+""#,
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_case_insensitive_regex() {
    RULE.assert_ignores(r"ls | where name =~ '(?i)readme'");
}

#[test]
fn ignore_find_with_closure() {
    RULE.assert_ignores(r#"ls | find {|f| $f.name | str contains "test" }"#);
}

#[test]
fn ignore_enumerate_for_line_numbers() {
    RULE.assert_ignores(r#"lines | enumerate | where item =~ "pattern""#);
}

#[test]
fn ignore_length_for_counting() {
    RULE.assert_ignores(r#"lines | where $it =~ "pattern" | length"#);
}

#[test]
fn ignore_other_commands() {
    let good_codes = vec![
        r"ls | where size > 1kb",
        r"open file.txt | from json",
        r"http get api/endpoint",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_complex_where_conditions() {
    let good_codes = vec![
        r#"ls | where type == file and name =~ "test""#,
        r#"$data | where ($it.name | str contains "foo") and $it.value > 10"#,
        r"lines | where {|line| ($line | str length) > 80 }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_str_contains() {
    RULE.assert_ignores(r#"lines | where ($it | str contains "pattern")"#);
}
