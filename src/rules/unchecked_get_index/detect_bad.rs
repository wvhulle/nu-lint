use super::RULE;

#[test]
fn detect_get_with_numeric_index() {
    RULE.assert_detects("$list | get 0");
}

#[test]
fn detect_get_with_large_index() {
    RULE.assert_detects("$list | get 42");
}

#[test]
fn detect_in_pipeline() {
    RULE.assert_detects("ls | get name | get 0");
}

#[test]
fn detect_after_filter() {
    RULE.assert_detects("$items | where active | get 0");
}

#[test]
fn detect_in_closure() {
    RULE.assert_detects(
        r#"
        $data | each {|row|
            $row | get 0
        }
    "#,
    );
}

#[test]
fn detect_in_function() {
    RULE.assert_detects(
        r#"
        def get-first-item [] {
            $in | get 0
        }
    "#,
    );
}
