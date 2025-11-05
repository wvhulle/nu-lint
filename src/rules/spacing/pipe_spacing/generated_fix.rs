use super::rule;

#[test]
fn test_pipe_spacing_fix_no_spaces() {
    let bad_code = "echo 'hello'|str upcase";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, " | ");
}

#[test]
fn test_pipe_spacing_fix_no_space_before() {
    let bad_code = "echo 'hello'| str upcase";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, " | ");
}

#[test]
fn test_pipe_spacing_fix_no_space_after() {
    let bad_code = "echo 'hello' |str upcase";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, " | ");
}

#[test]
fn test_pipe_spacing_fix_multiple_spaces() {
    let bad_code = "echo 'hello'  |  str upcase";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, " | ");
}

#[test]
fn test_pipe_spacing_fix_description() {
    let bad_code = "echo 'hello'|str upcase";
    rule().assert_fix_description_contains(bad_code, "Fix pipe spacing");
}
