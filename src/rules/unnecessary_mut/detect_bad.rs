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
fn test_unnecessary_mut_fix_nested_function() {
    let bad_code = r"
def outer [] {
    def inner [] {
        mut x = 42
        $x
    }
    inner
}
";
    rule().assert_detects(bad_code);
}
