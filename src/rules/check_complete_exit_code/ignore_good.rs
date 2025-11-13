use crate::log::instrument;

use super::rule;

#[test]
fn test_exit_code_checked() {
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
fn test_exit_code_pipeline_equal_check() {
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
