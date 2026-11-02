use super::RULE;

#[test]
fn detect_hours_and_minutes_from_seconds() {
    RULE.assert_count(
        "def split-time [total_seconds: int] {
  let whole_hours = $total_seconds // 3600
  let whole_minutes = ($total_seconds mod 3600) // 60
  $\"($whole_hours)h($whole_minutes)m\"
}",
        1,
    );
}

#[test]
fn detect_minutes_and_seconds() {
    RULE.assert_detects(
        "def split-time [total_seconds: int] { [($total_seconds // 60) ($total_seconds mod 60)] }",
    );
}

#[test]
fn detect_days_and_hours() {
    RULE.assert_detects(
        "def split-time [total_seconds: int] { [($total_seconds // 86400) ($total_seconds mod \
         86400)] }",
    );
}

#[test]
fn detect_each_variable_once() {
    RULE.assert_count(
        "def split-times [first_total: int, second_total: int] {
  [($first_total // 60) ($first_total mod 60) ($second_total // 60) ($second_total mod 60)]
}",
        2,
    );
}
