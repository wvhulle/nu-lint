use super::rule;

fn init_logger() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
    });
}

#[test]
fn test_ignore_with_complete_between() {
    init_logger();
    let good_code = r"
let result = (^command1 | complete)
if $result.exit_code == 0 {
    ^command2
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_with_conditional() {
    init_logger();
    let good_code = r"
^command1 && ^command2
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_single_external() {
    init_logger();
    let good_code = r"
^command1
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_alias_definitions() {
    init_logger();
    let good_code = r"
alias b = bat
alias bn = bat --number
alias bnl = bat --number --line-range
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_export_alias_definitions() {
    init_logger();
    let good_code = r"
export alias b = bat
export alias bn = bat --number
export alias bnl = bat --number --line-range
export alias bp = bat --plain
export alias bpl = bat --plain --line-range
export alias bl = bat --line-range
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_separate_top_level_commands() {
    init_logger();
    let good_code = r"
def setup [] {
    ^make
}

def build [] {
    ^cargo build
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_builtin_print_commands() {
    init_logger();
    let good_code = r#"
def test_func [] {
    "test output"
}

def main [] {
    print -n (test_func)
    print "after"
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_builtin_commands_with_help() {
    init_logger();
    let good_code = r#"
export def main [] {
    print -n (help bm)
    print (["info"] | str join "\n")
}
"#;
    rule().assert_ignores(good_code);
}
