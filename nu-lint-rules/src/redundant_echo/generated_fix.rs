use super::RULE;

#[test]
fn fix_string_literal() {
    RULE.assert_fixed_is(r#"echo "hello world""#, r#""hello world""#);
}

#[test]
fn fix_variable() {
    RULE.assert_fixed_is("echo $value", "$value");
}

#[test]
fn fix_pipeline() {
    RULE.assert_fixed_is("echo $var | str upcase", "$var | str upcase");
}

#[test]
fn fix_bare_word_is_quoted() {
    RULE.assert_fixed_is("echo hello | save out.txt", "'hello' | save out.txt");
}

#[test]
fn fix_multiple_arguments_become_list() {
    RULE.assert_fixed_is("echo hello world test", "[hello world test]");
}

#[test]
fn fix_inside_subexpression_reported_once() {
    RULE.assert_count("let y = (echo $x)", 1);
    RULE.assert_fixed_is("let y = (echo $x)", "let y = ($x)");
}

#[test]
fn no_fix_without_arguments() {
    RULE.assert_detects_without_fix("echo");
}
