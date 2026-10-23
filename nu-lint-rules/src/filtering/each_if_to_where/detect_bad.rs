use super::RULE;
use crate::log::init_test_log;

#[test]
fn test_detect_each_if_simple_filter() {
    init_test_log();

    let bad_code = r"
ls | each { |f| if $f.size > 100kb { $f } }
";

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_each_if_complex_condition() {
    init_test_log();

    let bad_code = r"
open data.json | get items | each { |item| if ($item.status == 'active' and $item.count > 0) { $item } }
";

    RULE.assert_count(bad_code, 1);
}

#[test]
fn test_detect_each_if_with_property_access() {
    init_test_log();

    let bad_code = r"
open users.json | each { |u| if $u.age >= 18 { $u } }
";

    RULE.assert_detects(bad_code);
}
