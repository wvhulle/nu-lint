use super::rule;

#[test]
fn test_unnecessary_mut_fix_simple() {
    let bad_code = r"
def process [] {
    mut x = 5
    echo $x
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "");
    rule().assert_fix_description_contains(bad_code, "Remove 'mut'");
}

#[test]
fn test_unnecessary_mut_fix_multiple_variables() {
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
    rule().assert_fix_contains(bad_code, "");
}

#[test]
fn test_unnecessary_mut_fix_description() {
    let bad_code = r"
def test [] {
    mut unused = 123
}
";
    rule().assert_fix_description_contains(bad_code, "Remove 'mut' keyword");
    rule().assert_fix_description_contains(bad_code, "unused");
}

#[test]
fn test_unnecessary_mut_no_fix_for_reassigned() {
    let bad_code = r"
def process [] {
    mut x = 5
    $x = 10
    echo $x
}
";
    rule().assert_ignores(bad_code);
}
