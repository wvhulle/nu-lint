use super::rule;

#[test]
fn test_detect_each_if_simple_filter() {
    crate::clean_log::log();

    let bad_code = r"
ls | each { |f| if $f.size > 100kb { $f } }
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_each_if_complex_condition() {
    crate::clean_log::log();

    let bad_code = r"
open data.json | get items | each { |item| if ($item.status == 'active' and $item.count > 0) { $item } }
";

    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_each_if_with_property_access() {
    crate::clean_log::log();

    let bad_code = r"
open users.json | each { |u| if $u.age >= 18 { $u } }
";

    rule().assert_detects(bad_code);
}
