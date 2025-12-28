use super::RULE;
use crate::log::instrument;

#[test]
fn test_detects_unchecked_complete_result() {
    instrument();
    let bad_code = r"
let result = (^sed -i 's/foo/bar/g' file.txt | complete)
";

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detects_when_only_stderr_checked_not_exit_code() {
    instrument();
    let bad_code = r#"
let result = (^sed -i 's/old/new/g' config.txt | complete)
if ($result.stderr | is-empty) {
    print "ok"
}
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detects_stored_complete_result_never_accessed() {
    instrument();
    let bad_code = r#"
let result = (^rm -rf /tmp/build | complete)
print "Build finished"
"#;

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detects_mixed_complete_calls_with_unchecked_result() {
    instrument();
    let bad_code = r"
let success1 = (^sed -i '' file1.txt | complete | get exit_code) == 0
let result2 = (^sed -i '' file2.txt | complete)
if $success1 {
    print 'command1 succeeded'
}
";

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detects_outer_complete_when_inner_exit_code_checked() {
    instrument();
    let bad_code = r"
let inner = (^sed -i '' inner.txt | complete | get exit_code)
let outer = (^sed -i '' outer.txt | complete)
if $inner != 0 {
    print 'inner failed'
}
";

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detects_cat_command_without_exit_code_check() {
    instrument();
    let bad_code = r"
let result = (^cat file.txt | complete)
";

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detects_grep_command_without_exit_code_check() {
    instrument();
    let bad_code = r"
let result = (^grep pattern file.txt | complete)
";

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detects_find_command_without_exit_code_check() {
    instrument();
    let bad_code = r"
let result = (^find . -name '*.rs' | complete)
";

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detects_git_pull_without_exit_code_check() {
    instrument();
    let bad_code = r"
let result = (^git pull | complete)
";

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detects_curl_without_exit_code_check() {
    instrument();
    let bad_code = r"
let result = (^curl https://example.com | complete)
";

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detects_git_clone_without_exit_code_check() {
    instrument();
    let bad_code = r"
let result = (^git clone https://github.com/user/repo | complete)
";

    RULE.assert_detects(bad_code);
}
