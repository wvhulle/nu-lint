use super::rule;

#[test]
fn test_already_using_complete() {
    let good_code = r"let result = (^curl https://api.example.com | complete)
if $result.exit_code != 0 { error make { msg: 'Failed' } }
$result.stdout | from json";
    rule().assert_ignores(good_code);
}

#[test]
fn test_in_try_block() {
    let good_code = r"try { ^curl https://api.example.com | from json }";
    rule().assert_ignores(good_code);
}

#[test]
fn test_single_external_command() {
    let good_code = r"^git status";
    rule().assert_ignores(good_code);
}

#[test]
fn test_safe_command() {
    let good_code = r"^echo 'test' | from json";
    rule().assert_ignores(good_code);
}

#[test]
fn test_bare_external_command_no_pipeline() {
    // Bare external commands without pipelines are not detected by this rule
    let good_code = r"^curl https://example.com";
    rule().assert_ignores(good_code);
}

#[test]
fn test_complete_in_subexpression() {
    let good_code =
        r"let data = (^curl https://api.example.com | complete | get stdout | from json)";
    rule().assert_ignores(good_code);
}

#[test]
fn test_safe_ls_with_from() {
    let good_code = r"^ls -la | lines | from ssv";
    rule().assert_ignores(good_code);
}

#[test]
fn test_try_with_multiline() {
    let good_code = r"
try {
    ^curl https://api.example.com 
    | from json 
    | select name age
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_safe_command_git() {
    let good_code = r"^git log --format=%H | lines | each { |commit| ^git show $commit }";
    rule().assert_ignores(good_code);
}
