use super::RULE;

#[test]
fn fix_current_date() {
    RULE.assert_fixed_is("^date", "date now");
}

#[test]
fn fix_utc() {
    RULE.assert_fixed_is("^date -u", "date now | date to-timezone UTC");
}

#[test]
fn fix_format() {
    RULE.assert_fixed_is("^date +%Y-%m-%d", "date now | format date '%Y-%m-%d'");
}

#[test]
fn fix_quoted_format_with_padding_modifier() {
    RULE.assert_fixed_is(
        r#"^date "+%-d %B, %H:%M""#,
        "date now | format date '%-d %B, %H:%M'",
    );
}

#[test]
fn fix_utc_epoch_seconds() {
    RULE.assert_fixed_is(
        "^date --utc +%s",
        "date now | date to-timezone UTC | format date '%s'",
    );
}
