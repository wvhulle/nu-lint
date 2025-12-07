use super::rule;

#[test]
fn test_single_get_not_flagged() {
    let good_code = r#"
[[name value]; [foo 1] [bar 2]] | get name
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_combined_cell_path_not_flagged() {
    let good_code = r#"
[[name value]; [foo 1] [bar 2]] | get name.0
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_non_consecutive_gets_not_flagged() {
    let good_code = r#"
{a: {b: 1}} | get a | to json | get b
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_already_combined_path_not_flagged() {
    let good_code = r#"
{foo: {bar: {baz: 42}}} | get foo.bar.baz
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_get_with_no_args_not_flagged() {
    let good_code = r#"
{a: 1} | get
"#;
    rule().assert_ignores(good_code);
}
