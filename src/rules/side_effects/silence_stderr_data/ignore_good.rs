use super::RULE;

#[test]
fn test_ignores_command_without_stderr_data_side_effect() {
    RULE.assert_ignores("ls e>| ignore");
}

#[test]
fn test_ignores_command_without_stderr_redirection() {
    RULE.assert_ignores("ffmpeg -i input.mp4 output.mp4");
}

#[test]
fn test_ignores_stdout_only_redirection() {
    RULE.assert_ignores("ffmpeg -i input.mp4 output.mp4 o>| ignore");
}

#[test]
fn test_ignores_stderr_redirected_to_file() {
    RULE.assert_ignores("ffmpeg -i input.mp4 output.mp4 e> errors.txt");
}

#[test]
fn test_ignores_builtin_command() {
    RULE.assert_ignores("print 'test' e>| ignore");
}

#[test]
fn test_ignores_stderr_piped_but_not_to_ignore() {
    RULE.assert_ignores("ffmpeg -i input.mp4 output.mp4 e>| str trim");
}

#[test]
fn test_ignores_command_with_no_redirection() {
    RULE.assert_ignores("ffmpeg -version");
}
