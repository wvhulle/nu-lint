use super::RULE;

#[test]
fn fix_slice_1() {
    RULE.assert_fixed_is("[1, 2, 3, 4, 5] | slice 1..", "[1, 2, 3, 4, 5] | skip 1");
}

#[test]
fn fix_slice_2() {
    RULE.assert_fixed_is("[1, 2, 3] | slice 2..", "[1, 2, 3] | skip 2");
}

#[test]
fn fix_slice_10() {
    RULE.assert_fixed_is("ls | slice 10..", "ls | skip 10");
}

#[test]
fn fix_in_pipeline() {
    RULE.assert_fixed_contains(
        r#"
        def main [] {
            ls | slice 5.. | where size > 100
        }
        "#,
        "skip 5",
    );
}
