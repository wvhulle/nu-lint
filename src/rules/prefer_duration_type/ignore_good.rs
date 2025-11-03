use super::rule;

#[test]
fn test_ignore_duration_type_used() {
    let rule = rule();
    rule.assert_ignores(r"def wait [timeout: duration] { sleep $timeout }");
}

#[test]
fn test_ignore_string_without_duration_name() {
    let rule = rule();
    rule.assert_ignores(r"def process [input: string] { echo $input }");
}

#[test]
fn test_ignore_no_type_annotation() {
    let rule = rule();
    rule.assert_ignores(r"def process [input_data] { echo $input_data }");
}

#[test]
fn test_ignore_non_string_with_duration_name() {
    let rule = rule();
    rule.assert_ignores(r"def wait [timeout: int] { sleep ($timeout | into duration) }");
}

#[test]
fn test_ignore_timestamp_string() {
    let rule = rule();
    rule.assert_ignores(r"def log [timestamp: string] { echo $timestamp }");
}

#[test]
fn test_ignore_datetime_string() {
    let rule = rule();
    rule.assert_ignores(r"def process [datetime: string] { echo $datetime }");
}

#[test]
fn test_ignore_timezone_string() {
    let rule = rule();
    rule.assert_ignores(r"def convert [timezone: string] { echo $timezone }");
}

#[test]
fn test_ignore_timer_string() {
    let rule = rule();
    rule.assert_ignores(r"def start [timer_name: string] { echo $timer_name }");
}

#[test]
fn test_ignore_runtime_string() {
    let rule = rule();
    rule.assert_ignores(r"def analyze [runtime_info: string] { echo $runtime_info }");
}

#[test]
fn test_ignore_list_of_durations() {
    let rule = rule();
    rule.assert_ignores(
        r"def wait_many [timeouts: list<duration>] { for t in $timeouts { sleep $t } }",
    );
}

#[test]
fn test_ignore_record_type_with_duration() {
    let rule = rule();
    rule.assert_ignores(r"def config [settings: record] { echo $settings.timeout }");
}

#[test]
fn test_ignore_table_type() {
    let rule = rule();
    rule.assert_ignores(r"def process [data: table] { echo $data.timeout }");
}

#[test]
fn test_ignore_closure_parameter() {
    let rule = rule();
    rule.assert_ignores(
        r"def retry [timeout: duration, action: closure] { sleep $timeout; do $action }",
    );
}
