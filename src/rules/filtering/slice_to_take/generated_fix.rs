use super::RULE;

#[test]
fn fix_slice_0_to_4() {
    RULE.assert_fixed_is("[1, 2, 3, 4, 5] | slice 0..4", "[1, 2, 3, 4, 5] | take 5");
}

#[test]
fn fix_slice_0_to_2() {
    RULE.assert_fixed_is("[1, 2, 3] | slice 0..2", "[1, 2, 3] | take 3");
}

#[test]
fn fix_slice_0_to_9() {
    RULE.assert_fixed_is("ls | slice 0..9", "ls | take 10");
}

#[test]
fn fix_in_pipeline() {
    RULE.assert_fixed_contains(
        r#"
        def main [] {
            ls | slice 0..5 | where size > 100
        }
        "#,
        "take 6",
    );
}
