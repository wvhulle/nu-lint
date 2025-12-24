use super::RULE;

#[test]
fn test_ignore_native_subcommands() {
    let good = r#"
def main [] {
    print "Use --help to see subcommands"
}

def "main info" [] {
    print "Info subcommand"
}

def "main adjust" [] {
    print "Adjust subcommand"
}
"#;

    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_main_without_match() {
    let good = r#"
def main [
    file: path
] {
    open $file | process
}
"#;

    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_match_on_non_command_variable() {
    let good = r#"
def main [
    level?: int
] {
    match $level {
        1 => { "low" }
        2 => { "medium" }
        3 => { "high" }
        _ => { "unknown" }
    }
}
"#;

    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_match_with_only_wildcard() {
    let good = r#"
def main [
    mode?: string
] {
    match $mode {
        _ => { run-normal }
    }
}
"#;

    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_non_main_function_with_dispatch() {
    let good = r#"
def process-command [
    command?: string
] {
    match $command {
        "a" => { 1 }
        "b" => { 2 }
        "c" => { 3 }
        _ => { 0 }
    }
}

def main [] {
    process-command "a"
}
"#;

    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_main_with_match_on_different_variable() {
    let good = r#"
def main [
    command?: string
    mode?: string
] {
    let status = "ready"
    match $status {
        "ready" => { do-ready }
        "pending" => { do-pending }
        "done" => { do-done }
        _ => { do-unknown }
    }
}
"#;

    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_exported_main_with_subcommands() {
    let good = r#"
export def main [] {
    print "Module with subcommands"
}

export def "main sub1" [] {
    print "sub1"
}

export def "main sub2" [] {
    print "sub2"
}
"#;

    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_main_using_if_chain() {
    let good = r#"
def main [
    command?: string
] {
    if $command == "info" {
        print "info"
    } else if $command == "debug" {
        print "debug"
    } else {
        print "usage"
    }
}
"#;

    RULE.assert_ignores(good);
}
