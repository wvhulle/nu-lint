use super::RULE;

#[test]
fn fix_first_without_count() {
    RULE.assert_fixed_is("$items | first", "$items | first 1");
}

#[test]
fn fix_last_without_count() {
    RULE.assert_fixed_is("$items | last", "$items | last 1");
}

#[test]
fn fix_in_pipeline() {
    RULE.assert_fixed_is("ls | get name | first", "ls | get name | first 1");
}

#[test]
fn fix_after_filter() {
    RULE.assert_fixed_is(
        "$items | where active | last",
        "$items | where active | last 1",
    );
}

#[test]
fn fix_in_closure() {
    RULE.assert_fixed_is(
        "$groups | each {|group| $group.items | first }",
        "$groups | each {|group| $group.items | first 1 }",
    );
}
