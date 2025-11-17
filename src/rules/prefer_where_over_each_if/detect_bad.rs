use super::rule;
use crate::log::instrument;

#[test]
fn test_detect_each_if_simple_filter() {
    instrument();

    let bad_code = r"
ls | each { |f| if $f.size > 100kb { $f } }
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_each_if_complex_condition() {
    instrument();

    let bad_code = r"
open data.json | get items | each { |item| if ($item.status == 'active' and $item.count > 0) { $item } }
";

    rule().assert_count(bad_code, 1);
}

#[test]
fn test_detect_each_if_with_property_access() {
    instrument();

    let bad_code = r"
open users.json | each { |u| if $u.age >= 18 { $u } }
";

    rule().assert_detects(bad_code);
}
