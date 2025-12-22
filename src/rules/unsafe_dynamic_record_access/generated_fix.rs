use super::rule;

#[test]
fn fix_adds_optional_flag() {
    let bad = "$record | get $key";
    rule().assert_replacement_contains(bad, " -o");
}

#[test]
fn fix_adds_optional_flag_in_pipeline() {
    let bad = "$servers | get $name";
    rule().assert_replacement_contains(bad, " -o");
}

#[test]
fn fix_explanation_mentions_optional() {
    let bad = "$data | get $field";
    rule().assert_fix_explanation_contains(bad, "optional");
}
