use super::RULE;

#[test]
fn fixes_quoted_glob_to_ls() {
    let code = r#"ls "*.txt""#;
    let expected = "ls *.txt";
    RULE.assert_fixed_is(code, expected);
}

#[test]
fn fixes_quoted_glob_with_single_quotes() {
    let code = r#"ls '*.rs'"#;
    let expected = "ls *.rs";
    RULE.assert_fixed_is(code, expected);
}

#[test]
fn fixes_glob_with_question_mark() {
    let code = r#"ls "file?.txt""#;
    let expected = "ls file?.txt";
    RULE.assert_fixed_is(code, expected);
}

#[test]
fn fixes_glob_in_rm_command() {
    let code = r#"rm "*.log""#;
    let expected = "rm *.log";
    RULE.assert_fixed_is(code, expected);
}

#[test]
fn fixes_glob_with_spaces() {
    let code = r#"ls "my files *""#;
    let expected = r#"ls ("my files *" | into glob)"#;
    RULE.assert_fixed_is(code, expected);
}

#[test]
fn fixes_glob_with_spaces_and_wildcards() {
    let code = r#"ls "test files/*.txt""#;
    let expected = r#"ls ("test files/*.txt" | into glob)"#;
    RULE.assert_fixed_is(code, expected);
}

#[test]
fn fixes_glob_with_special_chars() {
    let code = r#"ls "path|with|pipes*""#;
    let expected = r#"ls ("path|with|pipes*" | into glob)"#;
    RULE.assert_fixed_is(code, expected);
}
