use super::RULE;
use crate::log::instrument;

#[test]
fn test_ignore_where_usage() {
    instrument();

    let good_code = r"
ls | where size > 10kb
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_each_with_side_effects() {
    instrument();

    let good_code = r"
ls | each { |f| if $f.size > 100kb { print $f.name } }
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_each_without_if() {
    instrument();

    let good_code = r"
seq 1 10 | each { |x| $x * 2 }
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_each_if_with_mutation() {
    instrument();

    let good_code = r"
ls | each { |f| if $f.size > 100kb { mut name = $f.name; $name } }
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_each_if_with_else_clause() {
    instrument();

    let good_code = r"
ls | each { |f| if $f.size > 100kb { $f } else { null } }
";

    RULE.assert_ignores(good_code);
}
