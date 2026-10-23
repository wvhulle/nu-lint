use super::RULE;

// Note: Variable tests removed - Nu parser produces Garbage for variables in
// ranges like ..-$n

#[test]
fn fix_slice_to_minus_2() {
    RULE.assert_fixed_is("[1, 2, 3, 4, 5] | slice ..-2", "[1, 2, 3, 4, 5] | drop 1");
}

#[test]
fn fix_slice_0_to_minus_3() {
    RULE.assert_fixed_is("[1, 2, 3, 4, 5] | slice 0..-3", "[1, 2, 3, 4, 5] | drop 2");
}

#[test]
fn fix_slice_to_minus_4() {
    RULE.assert_fixed_is("ls | slice ..-4", "ls | drop 3");
}

#[test]
fn fix_in_pipeline() {
    RULE.assert_fixed_contains(
        r#"
        def main [] {
            ls | slice ..-2 | where size > 100
        }
        "#,
        "drop 1",
    );
}
