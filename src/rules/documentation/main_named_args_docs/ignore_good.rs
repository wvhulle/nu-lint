use super::RULE;

#[test]
fn main_named_flag_with_documentation() {
    let source = r#"
def main [
    --verbose # Enable verbose output
] {
    if $verbose { print "Verbose mode" }
}
"#;
    RULE.assert_count(source, 0);
}

#[test]
fn main_short_named_flag_with_documentation() {
    let source = r#"
def main [
    --verbose (-v) # Enable verbose output
] {
    if $verbose { print "Verbose mode" }
}
"#;
    RULE.assert_count(source, 0);
}

#[test]
fn main_multiple_named_flags_with_documentation() {
    let source = r#"
def main [
    --verbose # Enable verbose output
    --debug # Enable debug mode
    --output: string # Output file path
] {
    if $verbose { print "Verbose" }
    if $debug { print "Debug" }
    print $output
}
"#;
    RULE.assert_count(source, 0);
}

#[test]
fn main_typed_named_flag_with_documentation() {
    let source = r"
def main [
    --count: int # Number of iterations
] {
    1..$count
}
";
    RULE.assert_count(source, 0);
}

#[test]
fn non_main_function_ignores_documentation_requirement() {
    let source = r#"
def helper [--verbose] {
    if $verbose { print "Verbose" }
}
"#;
    RULE.assert_count(source, 0);
}

#[test]
fn main_without_named_parameters_passes() {
    let source = r#"
def main [] {
    echo "Hello"
}
"#;
    RULE.assert_count(source, 0);
}

#[test]
fn main_mixed_parameters_all_documented_passes() {
    let source = r#"
def main [
    input # Input file
    --verbose # Enable verbose output
] {
    if $verbose { print "Processing" $input }
}
"#;
    RULE.assert_count(source, 0);
}
