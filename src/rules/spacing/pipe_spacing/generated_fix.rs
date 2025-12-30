use super::RULE;

#[test]
fn test_pipe_spacing_fix_no_spaces() {
    let bad_code = "echo 'hello'|str upcase";
    RULE.assert_fixed_contains(bad_code, " | ");
}

#[test]
fn test_pipe_spacing_fix_no_space_before() {
    let bad_code = "echo 'hello'| str upcase";
    RULE.assert_fixed_contains(bad_code, " | ");
}

#[test]
fn test_pipe_spacing_fix_no_space_after() {
    let bad_code = "echo 'hello' |str upcase";
    RULE.assert_fixed_contains(bad_code, " | ");
}

#[test]
fn test_pipe_spacing_fix_multiple_spaces() {
    let bad_code = "echo 'hello'  |  str upcase";
    RULE.assert_fixed_contains(bad_code, " | ");
}

#[test]
fn test_pipe_spacing_fix_description() {
    let bad_code = "echo 'hello'|str upcase";
    RULE.assert_fix_explanation_contains(bad_code, "Fix pipe spacing");
}
