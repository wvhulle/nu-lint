use super::rule;

#[test]
fn test_unnecessary_mut_detected() {
    let bad_code = r"
def process [] {
    mut x = 5
    echo $x
}
";

    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_multiple_mut_variables() {
    let bad_code = r"
def process [] {
    mut a = 1
    mut b = 2
    mut c = 3
    $a = 10
    $c = 30
    echo $a $b $c
}
";

    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_unnecessary_mut_fix_provided() {
    let bad_code = r"
def process [] {
    mut x = 5
    echo $x
}
";

    rule().assert_violation_count_exact(bad_code, 1);
}
