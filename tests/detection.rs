use assert_cmd::Command;
use std::fs;

/// Helper function to run linter on a test file and check for specific rule
fn check_rule_detected(rule_id: &str) {
    let test_file = format!("tests/nu/{}.nu", rule_id);

    // Ensure test file exists
    assert!(
        fs::metadata(&test_file).is_ok(),
        "Test file {} not found",
        test_file
    );

    let mut cmd = Command::cargo_bin("nu-lint").unwrap();

    // Don't assert success/failure, just get output
    // The linter returns non-zero when violations are found
    let output = cmd.arg(&test_file).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Check that the specific rule ID appears in the output
    assert!(
        combined.contains(rule_id),
        "Rule {} not detected in file {}. Stdout:\n{}\nStderr:\n{}",
        rule_id,
        test_file,
        stdout,
        stderr
    );
}

// Style rules tests
#[test]
fn test_s001_snake_case_variables_detected() {
    check_rule_detected("S001");
}

#[test]
fn test_s002_kebab_case_commands_detected() {
    check_rule_detected("S002");
}

#[test]
fn test_s003_screaming_snake_constants_detected() {
    check_rule_detected("S003");
}

#[test]
fn test_s005_pipe_spacing_detected() {
    check_rule_detected("S005");
}

#[test]
fn test_s007_brace_spacing_detected() {
    check_rule_detected("S007");
}

#[test]
fn test_s008_prefer_compound_assignment_detected() {
    check_rule_detected("S008");
}

#[test]
fn test_s009_unnecessary_variable_return_detected() {
    check_rule_detected("S009");
}

#[test]
fn test_s010_prefer_is_not_empty_detected() {
    check_rule_detected("S010");
}

#[test]
fn test_s011_discourage_bare_ignore_detected() {
    check_rule_detected("S011");
}

#[test]
fn test_s012_discourage_underscore_commands_detected() {
    check_rule_detected("S012");
}

#[test]
fn test_s014_completion_function_naming_detected() {
    check_rule_detected("S014");
}

// Best practices tests
#[test]
fn test_bp001_prefer_error_make_detected() {
    check_rule_detected("BP001");
}

#[test]
fn test_bp002_avoid_mutable_accumulation_detected() {
    check_rule_detected("BP002");
}

#[test]
fn test_bp003_prefer_range_iteration_detected() {
    check_rule_detected("BP003");
}

#[test]
fn test_bp004_prefer_parse_command_detected() {
    check_rule_detected("BP004");
}

#[test]
fn test_bp005_consistent_error_handling_detected() {
    check_rule_detected("BP005");
}

#[test]
fn test_bp007_prefer_match_over_if_chain_detected() {
    check_rule_detected("BP007");
}

#[test]
fn test_bp008_prefer_each_over_for_detected() {
    check_rule_detected("BP008");
}

#[test]
fn test_bp009_max_positional_params_detected() {
    check_rule_detected("BP009");
}

#[test]
fn test_bp011_descriptive_error_messages_detected() {
    check_rule_detected("BP011");
}

// Documentation tests
#[test]
fn test_d001_missing_command_docs_detected() {
    check_rule_detected("D001");
}

#[test]
fn test_d002_exported_function_docs_detected() {
    check_rule_detected("D002");
}

// Type safety tests
#[test]
fn test_t001_missing_type_annotation_detected() {
    check_rule_detected("T001");
}

// Performance tests
#[test]
fn test_p001_prefer_where_over_each_if_detected() {
    check_rule_detected("P001");
}
