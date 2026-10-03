use super::RULE;

#[test]
fn fix_cat_to_open_raw() {
    RULE.assert_fixed_is("^cat file.txt", "open --raw file.txt");
}

#[test]
fn fix_keeps_quoted_path() {
    RULE.assert_fixed_is(r#"^cat "my file.txt""#, r#"open --raw "my file.txt""#);
}

#[test]
fn fix_keeps_variable_path() {
    RULE.assert_fixed_is(
        "def show [path] { ^cat $path }",
        "def show [path] { open --raw $path }",
    );
}
