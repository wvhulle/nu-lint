use super::RULE;

#[test]
fn ignore_properly_typed_output() {
    let good_code = r"
def create-list []: nothing -> list<int> {
    [1, 2, 3]
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_function_returning_nothing() {
    let good_code = r#"
def setup [] {
    mkdir ~/.config/myapp
    print "Setup complete"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_print_only_command() {
    let good_code = r#"
def main [] {
    print "Solar-based brightness manager"
    print "Use --help to see available subcommands"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_specific_output_type_with_any_input() {
    let good_code = r"
def create []: any -> list<int> {
    [1, 2, 3]
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_fully_typed_pipeline() {
    let good_code = r"
def transform []: list<int> -> list<int> {
    $in | each { |x| $x + 1 }
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_function_with_specific_types() {
    let good_code = r"
def add [a: int, b: int]: nothing -> int {
    $a + $b
}
";
    RULE.assert_ignores(good_code);
}
