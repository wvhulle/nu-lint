use super::RULE;

#[test]
fn fix_append_uniq_to_union() {
    RULE.assert_fixed_is("[3 1 2] | append [1 4] | uniq", "[3 1 2] | union [1 4]");
}

#[test]
fn fix_where_in_uniq_to_intersect() {
    let bad_code = r#"
let allowed = [1 4]
[3 1 2 1 4] | where $it in $allowed | uniq
"#;
    let expected = r#"
let allowed = [1 4]
[3 1 2 1 4] | intersect $allowed
"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn fix_where_not_in_uniq_to_difference() {
    let bad_code = r#"
let excluded = [1 4]
[3 1 2 1 4] | where $it not-in $excluded | uniq
"#;
    let expected = r#"
let excluded = [1 4]
[3 1 2 1 4] | difference $excluded
"#;
    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn fix_keeps_following_pipeline_elements() {
    let bad_code = r#"
let left = [3 1 2]
$left | append [1 4] | uniq | sort
"#;
    let expected = r#"
let left = [3 1 2]
$left | union [1 4] | sort
"#;
    RULE.assert_fixed_is(bad_code, expected);
}
