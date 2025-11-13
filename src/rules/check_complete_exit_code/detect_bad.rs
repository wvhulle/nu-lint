use super::rule;

#[test]
fn test_missing_exit_code_check() {
    let bad_code = r"
let result = (^bluetoothctl info $mac | complete)
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_mut_missing_exit_code_check() {
    let bad_code = r"
mut result = (^git status | complete)
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_complete_with_only_stderr_check() {
    let bad_code = r#"
let result = (^command arg | complete)
if ($result.stderr | is-empty) {
    print "ok"
}
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_complete_stored_but_not_used() {
    let bad_code = r#"
let result = (^make build | complete)
print "Build finished"
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_wrong_variable_exit_code_checked() {
    // Checking result1.exit_code when result2 is the one with complete
    let bad_code = r"
let result1 = (^git fetch | complete)
let result2 = (^git pull | complete)
if $result1.exit_code != 0 {
    return
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_swapped_exit_code_checks() {
    // Both have complete, but exit codes are checked for wrong variables
    let bad_code = r"
let fetch_result = (^git fetch | complete)
let pull_result = (^git pull | complete)
if $pull_result.exit_code != 0 {
    print 'fetch failed'
}
if $fetch_result.exit_code != 0 {
    print 'pull failed'
}
";

    // This should NOT detect - both are checked, even if messages are wrong
    // The rule only cares that exit_code is checked, not that it's semantically
    // correct
    rule().assert_ignores(bad_code);
}

#[test]
fn test_multiple_complete_inline_only_one_checked() {
    let bad_code = r"
let success1 = (^command1 | complete | get exit_code) == 0
let result2 = (^command2 | complete)
if $success1 {
    print 'command1 succeeded'
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_nested_complete_outer_unchecked() {
    // Inner complete has exit_code checked inline, outer doesn't
    let bad_code = r"
let inner = (^inner-cmd | complete | get exit_code)
let outer = (^outer-cmd | complete)
if $inner != 0 {
    print 'inner failed'
}
";

    rule().assert_detects(bad_code);
}
