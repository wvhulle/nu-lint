use super::rule;

#[test]
fn test_detects_unchecked_complete_result() {
    let bad_code = r"
let result = (^bluetoothctl info $mac | complete)
";

    rule().assert_detects(bad_code);
}


#[test]
fn test_detects_when_only_stderr_checked_not_exit_code() {
    let bad_code = r#"
let result = (^command arg | complete)
if ($result.stderr | is-empty) {
    print "ok"
}
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detects_stored_complete_result_never_accessed() {
    let bad_code = r#"
let result = (^make build | complete)
print "Build finished"
"#;

    rule().assert_detects(bad_code);
}



#[test]
fn test_detects_mixed_complete_calls_with_unchecked_result() {
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
fn test_detects_outer_complete_when_inner_exit_code_checked() {
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
