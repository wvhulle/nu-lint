use super::RULE;
use crate::log::init_test_log;

#[test]
fn test_detect_loop_with_counter_and_break() {
    init_test_log();
    let bad_code = r"
mut x = 0
loop {
    if $x > 10 { break }
    $x = $x + 1
}
";

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_loop_with_compound_increment() {
    init_test_log();
    let bad_code = r"
mut i = 0
loop {
    if $i >= 5 { break }
    print $i
    $i += 1
}
";

    RULE.assert_count(bad_code, 1);
}
