use super::RULE;

#[test]
fn test_fix_simple_each_with_print() {
    let bad_code = r"[1 2 3] | each {|x| print $x}";
    let expected = r"for x in [1 2 3] { print $x }";
    RULE.assert_fixed_contains(bad_code, expected);
}

#[test]
fn test_fix_each_with_multiple_prints() {
    let bad_code = r#"[1 2 3] | each {|x|
    print "Value:"
    print $x
}"#;
    let expected = r#"for x in [1 2 3] { print "Value:"
    print $x }"#;
    RULE.assert_fixed_contains(bad_code, expected);
}

// Complex pipelines with multiple stages before `each` don't get auto-fixes.
// Detection is still tested in detect_bad.rs. Users should manually decide
// whether to restructure the code or add `| ignore`.

#[test]
fn fix_replaces_the_list_stage() {
    RULE.assert_fixed_is(
        "[1 2] | each { |number| print $number }",
        "for number in [1 2] { print $number }",
    );
}

#[test]
fn fix_wraps_command_list_in_parentheses() {
    RULE.assert_fixed_is(
        "seq 0 3 | each { |number| print $number }",
        "for number in (seq 0 3) { print $number }",
    );
}

#[test]
fn no_fix_when_pipeline_continues_after_each() {
    RULE.assert_detects_without_fix("[1 2] | each { |number| print $number } | str join");
}

#[test]
fn no_fix_when_body_reads_pipeline_input() {
    RULE.assert_detects_without_fix("[1 2] | each { print $in }");
}
