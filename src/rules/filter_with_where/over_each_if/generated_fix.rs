use super::RULE;
use crate::log::instrument;

#[test]
fn test_fix_simple_each_if_to_where() {
    instrument();

    let bad_code = r"
ls | each {|f| if $f.size > 100kb { $f } }
";

    RULE.assert_replacement_contains(bad_code, "where");
    RULE.assert_replacement_contains(bad_code, "$f.size > 100kb");
}

#[test]
fn test_fix_preserves_condition() {
    instrument();

    let bad_code = r"
open users.json | each {|u| if $u.age >= 18 { $u } }
";

    RULE.assert_replacement_contains(bad_code, "where $u.age >= 18");
}

#[test]
fn test_fix_complex_condition() {
    instrument();

    let bad_code = r"
open data.json | get items | each {|item| if ($item.status == 'active' and $item.count > 0) { $item } }
";

    RULE.assert_replacement_contains(bad_code, "where");
    RULE.assert_replacement_contains(bad_code, "$item.status == 'active' and $item.count > 0");
}

#[test]
fn test_fix_explanation_mentions_where() {
    instrument();

    let bad_code = r"
ls | each {|f| if $f.size > 100kb { $f } }
";

    RULE.assert_fix_explanation_contains(bad_code, "where");
}

#[test]
fn test_fix_removes_each_and_if() {
    instrument();

    let bad_code = r"
[1 2 3 4 5] | each {|x| if $x > 2 { $x } }
";

    RULE.assert_replacement_contains(bad_code, "where $x > 2");
}
