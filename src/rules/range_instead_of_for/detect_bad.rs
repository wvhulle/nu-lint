use super::rule;
use crate::log::instrument;

#[test]
fn test_detect_while_loop_with_counter() {
    instrument();
    let bad_code = r"
mut attempts = 0
while $attempts < 10 {
    print $'Attempt ($attempts)'
    $attempts = $attempts + 1
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_while_loop_with_compound_assignment() {
    instrument();
    let bad_code = r"
mut count = 0
while $count < 5 {
    do_something
    $count += 1
}
";

    rule().assert_count(bad_code, 1);
}
