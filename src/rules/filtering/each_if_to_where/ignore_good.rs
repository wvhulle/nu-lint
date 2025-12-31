use super::RULE;
use crate::log::init_env_log;

#[test]
fn test_ignore_where_usage() {
    init_env_log();

    let good_code = r"
ls | where size > 10kb
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_each_with_side_effects() {
    init_env_log();

    let good_code = r"
ls | each { |f| if $f.size > 100kb { print $f.name } }
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_each_without_if() {
    init_env_log();

    let good_code = r"
seq 1 10 | each { |x| $x * 2 }
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_each_if_with_mutation() {
    init_env_log();

    let good_code = r"
ls | each { |f| if $f.size > 100kb { mut name = $f.name; $name } }
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignore_each_if_with_else_clause() {
    init_env_log();

    let good_code = r"
ls | each { |f| if $f.size > 100kb { $f } else { null } }
";

    RULE.assert_ignores(good_code);
}
