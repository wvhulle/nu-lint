use super::rule;

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

    rule().assert_ignores(good_code);
}

#[test]
fn test_immutable_variable_not_flagged() {
    let good_code = r"
def process [] {
    let x = 5
    echo $x
}
";

    rule().assert_ignores(good_code);
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

    rule().assert_ignores(good_code);
}

#[test]
fn test_underscore_prefixed_mut_not_flagged() {
    let good_code = r#"
def process [] {
    mut _temp = 5
    echo "done"
}
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn test_necessary_mut_no_fix() {
    let good_code = r"
def increment [] {
    mut counter = 0
    $counter += 1
    echo $counter
}
";

    rule().assert_ignores(good_code);
}
