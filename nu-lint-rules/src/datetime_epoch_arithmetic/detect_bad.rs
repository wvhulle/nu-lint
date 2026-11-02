use super::RULE;

#[test]
fn detect_date_now_divided_to_seconds() {
    RULE.assert_detects("(date now | into int) / 1_000_000_000");
}

#[test]
fn detect_date_now_floor_divided_to_seconds() {
    RULE.assert_detects("(date now | into int) // 1_000_000_000");
}

#[test]
fn detect_parsed_datetime_divided_to_seconds() {
    RULE.assert_detects("('2026-01-01' | into datetime | into int) / 1_000_000_000");
}

#[test]
fn detect_typed_variable_divided_to_seconds() {
    RULE.assert_detects(
        "def elapsed-seconds [started: datetime] { ($started | into int) / 1_000_000_000 }",
    );
}

#[test]
fn detect_subtraction_of_two_epochs() {
    RULE.assert_count(
        "def elapsed-nanos [started: datetime] { (date now | into int) - ($started | into int) }",
        1,
    );
}

#[test]
fn detect_subtraction_from_integer() {
    RULE.assert_detects("let limit = 5\n$limit - (date now | into int)");
}
