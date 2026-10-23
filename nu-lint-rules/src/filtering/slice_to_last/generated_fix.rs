use super::RULE;

#[test]
fn fix_slice_minus_2() {
    RULE.assert_fixed_is("[1, 2, 3, 4, 5] | slice (-2)..", "[1, 2, 3, 4, 5] | last 2");
}

#[test]
fn fix_slice_minus_1() {
    RULE.assert_fixed_is("[1, 2, 3] | slice (-1)..", "[1, 2, 3] | last 1");
}

#[test]
fn fix_slice_minus_10() {
    RULE.assert_fixed_is("ls | slice (-10)..", "ls | last 10");
}

#[test]
fn fix_in_pipeline() {
    RULE.assert_fixed_contains(
        r#"
        def main [] {
            ls | slice (-5).. | where size > 100
        }
        "#,
        "last 5",
    );
}
