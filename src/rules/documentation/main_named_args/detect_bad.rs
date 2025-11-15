use super::rule;

#[test]
fn main_named_flag_missing_documentation() {
    let source = r#"
def main [--verbose] {
    if $verbose { print "Verbose mode" }
}
"#;
    rule().assert_violation_count_exact(source, 1);
}

#[test]
fn main_short_named_flag_missing_documentation() {
    let source = r#"
def main [--verbose (-v)] {
    if $verbose { print "Verbose mode" }
}
"#;
    rule().assert_violation_count_exact(source, 1);
}

#[test]
fn main_multiple_named_flags_missing_documentation() {
    let source = r#"
def main [--verbose --debug --output: string] {
    if $verbose { print "Verbose" }
    if $debug { print "Debug" }
    print $output
}
"#;
    rule().assert_violation_count_exact(source, 3);
}

#[test]
fn main_typed_named_flag_missing_documentation() {
    let source = r"
def main [--count: int] {
    1..$count
}
";
    rule().assert_violation_count_exact(source, 1);
}

#[test]
fn main_named_flag_with_default_missing_documentation() {
    let source = r#"
def main [--output: string = "output.txt"] {
    save $output
}
"#;
    rule().assert_violation_count_exact(source, 1);
}
