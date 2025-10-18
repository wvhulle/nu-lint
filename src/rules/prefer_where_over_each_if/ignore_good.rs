use super::rule;

#[test]
fn test_ignore_where_usage() {
    let good_code = r"
ls | where size > 10kb
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_each_with_side_effects() {
    let good_code = r"
ls | each { |f| if $f.size > 100kb { print $f.name } }
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_each_without_if() {
    let good_code = r"
seq 1 10 | each { |x| $x * 2 }
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_each_if_with_mutation() {
    let good_code = r"
ls | each { |f| if $f.size > 100kb { mut name = $f.name; $name } }
";

    rule().assert_ignores(good_code);
}
