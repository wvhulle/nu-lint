use super::RULE;

#[test]
fn ignores_nushell_where() {
    RULE.assert_ignores(r#"lines | where $it =~ "pattern""#);
}

#[test]
fn ignores_nushell_split_column() {
    RULE.assert_ignores(r#"lines | split column "," | get column1"#);
}

#[test]
fn ignores_nushell_select() {
    RULE.assert_ignores(r#"open data.csv | select name age"#);
}

#[test]
fn ignores_open_lines_pipeline() {
    let good_codes = vec![
        r#"open file.txt | lines | where $it =~ "error""#,
        r#"open log.txt | lines | split column " " | get column1"#,
        r#"$content | lines | each {|line| $line}"#,
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignores_structured_data_operations() {
    let good_codes = vec![
        r#"open data.csv | where name =~ "test""#,
        r#"$records | select column1 column3"#,
        r#"open file.json | get items | each {|item| $item.name}"#,
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignores_enumerate_for_line_numbers() {
    RULE.assert_ignores(r#"lines | enumerate | where item =~ "pattern""#);
}

#[test]
fn ignores_split_column_with_separator() {
    let good_codes = vec![
        r#"lines | split column ":""#,
        r#"lines | split column "," | get column2"#,
        r#"lines | split column " " | select column1 column3"#,
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignores_other_commands() {
    let good_codes = vec![
        r#"ls | where size > 1kb"#,
        r#"open file.txt | from json"#,
        r#"http get api/endpoint"#,
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignores_get_command() {
    RULE.assert_ignores(r#"$data | get column1"#);
}

#[test]
fn ignores_each_transformation() {
    RULE.assert_ignores(r#"lines | each {|line| $line | str trim}"#);
}
