use super::RULE;

#[test]
fn division_by_hours() {
    RULE.assert_detects("$diff / 1hr");
}

#[test]
fn division_by_minutes() {
    RULE.assert_detects("$diff / 1min");
}

#[test]
fn division_by_seconds() {
    RULE.assert_detects("$diff / 1sec");
}

#[test]
fn division_by_days() {
    RULE.assert_detects("$diff / 1day");
}

#[test]
fn nested_in_subexpression() {
    RULE.assert_detects("($diff / 1hr) | math floor");
}

#[test]
fn date_subtraction_divided() {
    RULE.assert_detects("($reset - (date now)) / 1hr");
}
