use super::rule;

#[test]
fn test_detect_timeout_parameter() {
    let rule = rule();
    rule.assert_detects(r"def wait [timeout: string] { sleep $timeout }");
}

#[test]
fn test_detect_duration_parameter() {
    let rule = rule();
    rule.assert_detects(r"def sleep_for [duration: string] { sleep $duration }");
}

#[test]
fn test_detect_delay_parameter() {
    let rule = rule();
    rule.assert_detects(r"def wait_before [delay: string] { sleep $delay }");
}

#[test]
fn test_detect_interval_parameter() {
    let rule = rule();
    rule.assert_detects(r"def repeat [interval: string] { sleep $interval }");
}

#[test]
fn test_detect_period_parameter() {
    let rule = rule();
    rule.assert_detects(r"def schedule [period: string] { sleep $period }");
}

#[test]
fn test_detect_wait_parameter() {
    let rule = rule();
    rule.assert_detects(r"def pause [wait: string] { sleep $wait }");
}

#[test]
fn test_detect_sleep_parameter() {
    let rule = rule();
    rule.assert_detects(r"def do_sleep [sleep_time: string] { sleep $sleep_time }");
}

#[test]
fn test_detect_elapsed_parameter() {
    let rule = rule();
    rule.assert_detects(r"def track [elapsed: string] { $elapsed }");
}

#[test]
fn test_detect_time_parameter() {
    let rule = rule();
    rule.assert_detects(r"def wait_time [wait_time: string] { sleep $wait_time }");
}

#[test]
fn test_detect_timeout_no_type_annotation() {
    let rule = rule();
    rule.assert_detects(r"def wait [timeout] { sleep $timeout }");
}

#[test]
fn test_detect_duration_no_type_annotation() {
    let rule = rule();
    rule.assert_detects(r"def sleep_for [duration] { sleep $duration }");
}

#[test]
fn test_detect_delay_no_type_annotation() {
    let rule = rule();
    rule.assert_detects(r"def wait_before [delay] { sleep $delay }");
}

#[test]
fn test_detect_interval_no_type_annotation() {
    let rule = rule();
    rule.assert_detects(r"def repeat [interval] { sleep $interval }");
}

#[test]
fn test_detect_optional_timeout_parameter() {
    let rule = rule();
    rule.assert_detects(r"def wait [timeout?: string] { sleep ($timeout | default 5sec) }");
}

#[test]
fn test_detect_exported_function() {
    let rule = rule();
    rule.assert_detects(r"export def wait [timeout: string] { sleep $timeout }");
}

#[test]
fn test_detect_multiple_duration_parameters() {
    let rule = rule();
    rule.assert_violation_count_exact(
        r"def wait_with_retry [timeout: string, retry_delay: string] { sleep $timeout }",
        2,
    );
}

#[test]
fn test_detect_mixed_case_duration() {
    let rule = rule();
    rule.assert_detects(r"def wait [waitTimeout: string] { sleep $waitTimeout }");
}

#[test]
fn test_detect_uppercase_timeout() {
    let rule = rule();
    rule.assert_detects(r"def wait [TIMEOUT: string] { sleep $TIMEOUT }");
}
