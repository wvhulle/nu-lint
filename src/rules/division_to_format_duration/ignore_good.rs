use super::RULE;

#[test]
fn duration_divided_by_number() {
    RULE.assert_ignores("3hr / 2");
}

#[test]
fn regular_number_division() {
    RULE.assert_ignores("$x / $y");
}

#[test]
fn date_humanize_already_used() {
    RULE.assert_ignores("$iso | into datetime | date humanize");
}

#[test]
fn format_duration_already_used() {
    RULE.assert_ignores("$dur | format duration hr");
}
