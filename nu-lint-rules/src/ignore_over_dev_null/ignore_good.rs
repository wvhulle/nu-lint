use super::RULE;

#[test]
fn using_ignore() {
    RULE.assert_ignores(r"^curl https://api.example.com e>| ignore");
}

#[test]
fn using_ignore_both_streams() {
    RULE.assert_ignores(r"^curl https://api.example.com o+e>| ignore");
}

#[test]
fn external_without_redirect() {
    RULE.assert_ignores(r"^git status | lines");
}

#[test]
fn single_external_command() {
    RULE.assert_ignores(r"^curl https://example.com");
}

#[test]
fn builtin_command() {
    RULE.assert_ignores(r"ls | where size > 1kb");
}

#[test]
fn stderr_redirect_to_file() {
    RULE.assert_ignores(r"^curl https://example.com err> /tmp/errors.log | lines");
}

#[test]
fn using_complete_for_inspection() {
    RULE.assert_ignores(r"^curl https://api.example.com | complete | get stdout | from json");
}

#[test]
fn complete_with_error_handling() {
    RULE.assert_ignores(
        r"
let result = (^curl https://api.example.com | complete)
if $result.exit_code != 0 { error make { msg: $result.stderr } }
$result.stdout | from json
",
    );
}
