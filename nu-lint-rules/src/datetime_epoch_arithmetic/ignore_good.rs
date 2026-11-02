use super::RULE;

#[test]
fn ignore_datetime_subtraction() {
    RULE.assert_ignores("def elapsed-time [started: datetime] { (date now) - $started }");
}

#[test]
fn ignore_plain_epoch_conversion() {
    RULE.assert_ignores("date now | into int");
}

#[test]
fn ignore_epoch_passed_to_external() {
    RULE.assert_ignores("^notify-tool --at (date now | into int)");
}

#[test]
fn ignore_division_by_other_number() {
    RULE.assert_ignores("(date now | into int) / 7");
}

#[test]
fn ignore_addition() {
    RULE.assert_ignores("(date now | into int) + 1");
}

#[test]
fn ignore_integer_arithmetic_on_strings() {
    RULE.assert_ignores("('5' | into int) - 1");
}

#[test]
fn ignore_integer_division_to_seconds() {
    RULE.assert_ignores("def to-seconds [nanos: int] { $nanos / 1_000_000_000 }");
}
