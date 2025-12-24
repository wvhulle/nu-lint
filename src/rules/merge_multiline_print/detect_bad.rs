use super::RULE;

#[test]
fn test_detect_three_consecutive_prints() {
    let bad_code = r#"
print "line 1"
print "line 2"
print "line 3"
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_four_consecutive_prints() {
    let bad_code = r#"
print "Usage: script.nu <subcommand>"
print ""
print "Subcommands:"
print "  help - Show help"
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_prints_in_function() {
    let bad_code = r#"
def show_help [] {
    print "Usage: tool <command>"
    print ""
    print "Commands:"
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_stderr_prints() {
    let bad_code = r#"
print -e "Error occurred"
print -e "Please check input"
print -e "Exiting..."
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_in_match_branch() {
    let bad_code = r#"
def main [cmd: string] {
    match $cmd {
        "help" => {
            print "Help:"
            print "  --version"
            print "  --help"
        }
        _ => {}
    }
}
"#;
    RULE.assert_detects(bad_code);
}
