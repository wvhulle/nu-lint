use super::RULE;

#[test]
fn detect_first_without_count() {
    RULE.assert_detects("$items | first");
}

#[test]
fn detect_last_without_count() {
    RULE.assert_detects("$items | last");
}

#[test]
fn detect_in_pipeline() {
    RULE.assert_detects("ls | get name | first");
}

#[test]
fn detect_after_filter() {
    RULE.assert_detects("$items | where active | first");
}

#[test]
fn detect_last_in_closure() {
    RULE.assert_detects(
        r#"
        $groups | each {|group|
            $group.items | last
        }
    "#,
    );
}

#[test]
fn detect_in_function() {
    RULE.assert_detects(
        r#"
        def get-first-item [] {
            $in | first
        }
    "#,
    );
}
