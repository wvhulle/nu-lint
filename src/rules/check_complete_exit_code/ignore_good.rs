use super::RULE;
use crate::log::instrument;

#[test]
fn test_ignores_complete_result_with_exit_code_check() {
    instrument();
    let good_code = r"
let result = (^sed -i 's/foo/bar/g' file.txt | complete)
if $result.exit_code != 0 {
    return
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignores_regular_pipeline_without_complete() {
    instrument();
    let good_code = r"
let result = (some | regular | pipeline)
";
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignores_inline_exit_code_check_with_equality() {
    instrument();
    let good_code = r#"
def wait_for_service [] {
  let is_active = (^sed -n '1p' status.txt | complete | get exit_code) == 0

  if not $is_active {
    log "Waiting for service to become active..."
    sleep $SERVICE_WAIT_DELAY
  }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignores_exit_code_check_with_greater_than_comparison() {
    instrument();
    let good_code = r"
let result = (^rm -rf /tmp/test | complete)
if $result.exit_code > 0 {
    error make {msg: 'rm failed'}
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignores_exit_code_in_complex_boolean_expression() {
    instrument();
    let good_code = r"
let result = (^sed -i 's/old/new/g' test.txt | complete)
if $result.exit_code == 0 and ($result.stdout | str contains 'PASS') {
    print 'tests passed'
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignores_exit_code_checked_in_match_expression() {
    instrument();
    let good_code = r"
let result = (^sed -i '' file.txt | complete)
match $result.exit_code {
    0 => { print 'success' }
    _ => { print 'failed' }
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignores_exit_code_accessed_through_pipeline() {
    instrument();
    let good_code = r"
let result = (^rm -rf /tmp/cache | complete)
$result.exit_code | if $in != 0 { return }
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignores_exit_code_extracted_to_separate_variable() {
    instrument();
    let good_code = r"
let result = (^sed -i 's/x/y/g' build.sh | complete)
let code = $result.exit_code
if $code != 0 {
    error make {msg: 'build failed'}
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignores_mixed_inline_and_separate_exit_code_checks() {
    instrument();
    let good_code = r"
let fetch_ok = (^sed -n '1p' status.txt | complete | get exit_code) == 0
let pull_result = (^sed -i 's/a/b/g' config.txt | complete)
if $fetch_ok and $pull_result.exit_code == 0 {
    print 'both succeeded'
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignores_complete_results_all_checked_in_loop() {
    instrument();
    let good_code = r#"
let files = ["file1.txt" "file2.txt"]
for file in $files {
    let result = (^sed -i '' $file | complete)
    if $result.exit_code != 0 {
        print $"Failed to modify ($file)"
    }
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignores_exit_codes_stored_in_record_structure() {
    instrument();
    let good_code = r"
let fetch = (^sed -i '' file1.txt | complete)
let pull = (^sed -i '' file2.txt | complete)
let status = {
    fetch_ok: ($fetch.exit_code == 0),
    pull_ok: ($pull.exit_code == 0)
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_both_exit_codes_checked_regardless_of_semantic_correctness() {
    instrument();
    let good_code = r"
let fetch_result = (^sed -i '' fetch.txt | complete)
let pull_result = (^sed -i '' pull.txt | complete)
if $pull_result.exit_code != 0 {
    print 'fetch failed'
}
if $fetch_result.exit_code != 0 {
    print 'pull failed'
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_ignores_command_without_likely_errors() {
    instrument();
    let good_code = r"
let result = (^echo hello | complete)
";

    RULE.assert_ignores(good_code);
}
