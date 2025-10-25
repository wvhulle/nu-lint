use super::rule;

#[test]
fn test_functional_each_pipeline() {
    let good = "[1, 2, 3] | each { |x| $x * 2 }";
    rule().assert_ignores(good);
}

#[test]
fn test_functional_where_filter() {
    let good = "[1, 2, 3, 4] | where { |x| $x > 2 }";
    rule().assert_ignores(good);
}

#[test]
fn test_immutable_list() {
    let good = "let items = [1, 2, 3]";
    rule().assert_ignores(good);
}

#[test]
fn test_functional_reduce() {
    let good = "[1, 2, 3] | reduce { |it, acc| $acc + $it }";
    rule().assert_ignores(good);
}

#[test]
fn test_mutable_without_append() {
    let good = "mut counter = 0";
    rule().assert_ignores(good);
}
