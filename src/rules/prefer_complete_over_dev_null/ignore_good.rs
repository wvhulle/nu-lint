use super::rule;

#[test]
fn test_ignore_using_complete() {
    let good_code = r"^curl https://api.example.com | complete | get stdout | from json";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_external_without_redirect() {
    let good_code = r"^git status | lines";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_single_external_command() {
    let good_code = r"^curl https://example.com";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_builtin_command() {
    let good_code = r"ls | where size > 1kb";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_external_without_pipeline() {
    let good_code = r"^evtest $keyboard err> /dev/null";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_stdout_redirect() {
    let good_code = r"^curl https://example.com out> /tmp/output.txt | lines";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_complete_with_error_handling() {
    let good_code = r"
let result = (^curl https://api.example.com | complete)
if $result.exit_code != 0 { error make { msg: $result.stderr } }
$result.stdout | from json
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_stderr_redirect_to_file() {
    let good_code = r"^curl https://example.com err> /tmp/errors.log | lines";
    rule().assert_ignores(good_code);
}
