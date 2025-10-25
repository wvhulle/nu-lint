use super::rule;

#[test]
fn test_good_compound_add_assignment() {
    let good = "mut x = 5; $x += 3";
    rule().assert_ignores(good);
}

#[test]
fn test_good_compound_subtract_assignment() {
    let good = "mut count = 10; $count -= 2";
    rule().assert_ignores(good);
}

#[test]
fn test_good_compound_multiply_assignment() {
    let good = "mut factor = 2; $factor *= 3";
    rule().assert_ignores(good);
}

#[test]
fn test_good_simple_assignment() {
    let good = "mut x = 5; $x = 10";
    rule().assert_ignores(good);
}

#[test]
fn test_good_different_variables() {
    let good = "mut x = 5; mut y = 3; $x = $y + 2";
    rule().assert_ignores(good);
}

#[test]
fn test_good_append_assignment() {
    let good = "mut items = []; $items ++= [1, 2, 3]";
    rule().assert_ignores(good);
}
