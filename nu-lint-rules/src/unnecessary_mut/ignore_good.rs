use super::RULE;

#[test]
fn test_necessary_mut_not_flagged() {
    let good_code = r"
def fibonacci [n: int] {
    mut a = 0
    mut b = 1
    for _ in 2..=$n {
        let c = $a + $b
        $a = $b
        $b = $c
    }
    $b
}
";

    RULE.assert_ignores(good_code);
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
    RULE.assert_ignores(bad_code);
}

#[test]
fn test_immutable_variable_not_flagged() {
    let good_code = r"
def process [] {
    let x = 5
    echo $x
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_mut_with_compound_assignment() {
    let good_code = r"
def increment [] {
    mut counter = 0
    $counter += 1
    echo $counter
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_underscore_prefixed_mut_not_flagged() {
    let good_code = r#"
def process [] {
    mut _temp = 5
    echo "done"
}
"#;

    RULE.assert_ignores(good_code);
}
