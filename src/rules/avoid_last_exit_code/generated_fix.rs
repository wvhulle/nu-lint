use super::RULE;

#[test]
fn fix_removes_external_line_and_inlines() {
    let bad = r#"^git clone repo
if $env.LAST_EXIT_CODE != 0 {
    error make { msg: "failed" }
}
"#;

    RULE.assert_fixed_contains(bad, "(^git clone repo | complete).exit_code");
}

#[test]
fn fix_preserves_conditional_logic() {
    let bad = r#"^curl https://example.com
if $env.LAST_EXIT_CODE == 0 {
    print "success"
}
"#;

    RULE.assert_fixed_contains(bad, "(^curl https://example.com | complete).exit_code == 0");
}

#[test]
fn fix_handles_not_equal_zero() {
    let bad = r#"^git push
if $env.LAST_EXIT_CODE != 0 {
    return
}
"#;

    RULE.assert_fixed_contains(bad, "(^git push | complete).exit_code != 0");
}

#[test]
fn fix_with_sed_command() {
    let bad = r#"^sed -i 's/foo/bar/' file.txt
if $env.LAST_EXIT_CODE != 0 {
    print "sed failed"
}
"#;

    RULE.assert_fixed_contains(bad, "(^sed -i 's/foo/bar/' file.txt | complete).exit_code");
}

#[test]
fn fix_with_comparison_operator() {
    let bad = r#"^git status
if $env.LAST_EXIT_CODE > 0 {
    print "failed"
}
"#;

    RULE.assert_fixed_contains(bad, "(^git status | complete).exit_code");
}
