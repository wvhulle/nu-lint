use super::rule;

#[test]
fn test_detect_addition_assignment() {
    let bad_code = r"
mut count = 0
$count = $count + 1
";

    rule().assert_count(bad_code, 1);
}

#[test]
fn test_detect_subtraction_assignment() {
    let bad_code = r"
mut count = 0
$count = $count - 5
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_multiplication_assignment() {
    let bad_code = r"
mut count = 0
$count = $count * 2
";

    rule().assert_count(bad_code, 1);
}
