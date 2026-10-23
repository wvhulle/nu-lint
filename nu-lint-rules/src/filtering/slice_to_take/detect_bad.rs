use super::RULE;

#[test]
fn detect_slice_0_to_4() {
    RULE.assert_detects("[1, 2, 3, 4, 5] | slice 0..4");
}

#[test]
fn detect_slice_0_to_2() {
    RULE.assert_detects("[1, 2, 3] | slice 0..2");
}

#[test]
fn detect_slice_0_to_9() {
    RULE.assert_detects("ls | slice 0..9");
}

#[test]
fn detect_in_pipeline() {
    RULE.assert_detects(
        r#"
        def main [] {
            ls | slice 0..5 | where size > 100
        }
        "#,
    );
}

#[test]
fn detect_in_variable_assignment() {
    RULE.assert_detects(
        r#"
        let items = [1, 2, 3, 4, 5] | slice 0..3
        "#,
    );
}
