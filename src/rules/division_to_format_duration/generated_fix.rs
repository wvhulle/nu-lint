use super::RULE;

#[test]
fn fix_division_by_hours() {
    RULE.assert_fixed_contains("$diff / 1hr", "format duration hr");
}

#[test]
fn fix_division_by_minutes() {
    RULE.assert_fixed_contains("$diff / 1min", "format duration min");
}

#[test]
fn fix_division_by_seconds() {
    RULE.assert_fixed_contains("$diff / 1sec", "format duration sec");
}

#[test]
fn fix_preserves_left_operand() {
    RULE.assert_fixed_contains("$diff / 1hr", "$diff | format duration");
}
