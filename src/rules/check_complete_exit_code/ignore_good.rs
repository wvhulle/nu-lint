use super::rule;
use crate::log::instrument;

#[test]
fn test_exit_code_checked_with_not_equal() {
    let good_code = r"
let result = (^bluetoothctl info $mac | complete)
if $result.exit_code != 0 {
    return
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_no_complete_not_flagged() {
    let good_code = r"
let result = (some | regular | pipeline)
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_exit_code_inline_with_equal() {
    instrument();
    let good_code = r#"
def wait_for_bluetooth_service [] {
  let is_active = (^systemctl is-active bluetooth.service | complete | get exit_code) == 0

  if not $is_active {
    log "Waiting for bluetooth service to become active..."
    sleep $SERVICE_WAIT_DELAY
  }
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_exit_code_checked_with_greater_than() {
    let good_code = r"
let result = (^curl https://example.com | complete)
if $result.exit_code > 0 {
    error make {msg: 'curl failed'}
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_exit_code_in_comparison_chain() {
    let good_code = r"
let result = (^make test | complete)
if $result.exit_code == 0 and ($result.stdout | str contains 'PASS') {
    print 'tests passed'
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_exit_code_with_match() {
    let good_code = r"
let result = (^command arg | complete)
match $result.exit_code {
    0 => { print 'success' }
    _ => { print 'failed' }
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_exit_code_field_access_in_pipeline() {
    let good_code = r"
let result = (^git pull | complete)
$result.exit_code | if $in != 0 { return }
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_exit_code_stored_in_variable() {
    let good_code = r"
let result = (^build.sh | complete)
let code = $result.exit_code
if $code != 0 {
    error make {msg: 'build failed'}
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_multiple_complete_mixed_checking_styles() {
    let good_code = r"
let fetch_ok = (^git fetch | complete | get exit_code) == 0
let pull_result = (^git pull | complete)
if $fetch_ok and $pull_result.exit_code == 0 {
    print 'both succeeded'
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_complete_in_loop_all_checked() {
    let good_code = r#"
let files = ["file1.txt" "file2.txt"]
for file in $files {
    let result = (^cat $file | complete)
    if $result.exit_code != 0 {
        print $"Failed to read ($file)"
    }
}
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn test_complete_exit_codes_in_record() {
    let good_code = r"
let fetch = (^git fetch | complete)
let pull = (^git pull | complete)
let status = {
    fetch_ok: ($fetch.exit_code == 0),
    pull_ok: ($pull.exit_code == 0)
}
";

    rule().assert_ignores(good_code);
}
