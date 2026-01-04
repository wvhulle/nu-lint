use super::RULE;

#[test]
fn detect_slice_minus_2_to_end() {
    RULE.assert_detects("[1, 2, 3, 4, 5] | slice (-2)..");
}

#[test]
fn detect_slice_minus_1_to_end() {
    RULE.assert_detects("[1, 2, 3] | slice (-1)..");
}

#[test]
fn detect_slice_minus_3_to_end() {
    RULE.assert_detects("ls | slice (-3)..");
}

#[test]
fn detect_in_pipeline() {
    RULE.assert_detects(
        r#"
        def main [] {
            ls | slice (-5).. | where size > 100
        }
        "#,
    );
}

#[test]
fn detect_in_variable_assignment() {
    RULE.assert_detects(
        r#"
        let items = [1, 2, 3, 4, 5] | slice (-3)..
        "#,
    );
}
