use super::RULE;

#[test]
fn detect_slice_to_minus_2() {
    RULE.assert_detects("[1, 2, 3, 4, 5] | slice ..-2");
}

#[test]
fn detect_slice_0_to_minus_2() {
    RULE.assert_detects("[1, 2, 3, 4, 5] | slice 0..-2");
}

#[test]
fn detect_slice_to_minus_3() {
    RULE.assert_detects("[1, 2, 3] | slice ..-3");
}

#[test]
fn detect_slice_0_to_minus_4() {
    RULE.assert_detects("ls | slice 0..-4");
}

#[test]
fn detect_in_pipeline() {
    RULE.assert_detects(
        r#"
        def main [] {
            ls | slice ..-2 | where size > 100
        }
        "#,
    );
}

#[test]
fn detect_in_variable_assignment() {
    RULE.assert_detects(
        r#"
        let items = [1, 2, 3, 4, 5] | slice 0..-3
        "#,
    );
}
