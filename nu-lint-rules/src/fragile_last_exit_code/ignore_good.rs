use super::RULE;

#[test]
fn ignores_complete_with_inline_check() {
    RULE.assert_ignores(
        r#"
if (^git clone repo | complete).exit_code != 0 {
    error make { msg: "failed" }
}
"#,
    );
}

#[test]
fn ignores_complete_with_variable() {
    RULE.assert_ignores(
        r#"
let result = ^git clone repo | complete
if $result.exit_code != 0 {
    error make { msg: "failed" }
}
"#,
    );
}

#[test]
fn ignores_when_not_adjacent() {
    RULE.assert_ignores(
        r#"
^git clone repo
print "cloning..."
if $env.LAST_EXIT_CODE != 0 {
    error make { msg: "failed" }
}
"#,
    );
}

#[test]
fn ignores_last_exit_code_without_preceding_external() {
    RULE.assert_ignores(
        r#"
if $env.LAST_EXIT_CODE != 0 {
    print "something failed earlier"
}
"#,
    );
}

#[test]
fn ignores_external_without_check() {
    RULE.assert_ignores(
        r#"
^git clone repo
print "done"
"#,
    );
}

#[test]
fn ignores_builtin_command() {
    RULE.assert_ignores(
        r#"
ls
if $env.LAST_EXIT_CODE != 0 {
    print "ls failed"
}
"#,
    );
}

#[test]
fn ignores_complete_stored_in_variable_checked_later() {
    RULE.assert_ignores(
        r#"
let result = ^curl https://example.com | complete
if $result.exit_code == 0 {
    print $result.stdout
} else {
    error make { msg: "curl failed" }
}
"#,
    );
}
