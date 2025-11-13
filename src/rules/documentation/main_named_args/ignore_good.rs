use super::rule;

#[test]
fn test_main_named_flag_with_docs() {
    let source = r#"
def main [
    --verbose # Enable verbose output
] {
    if $verbose { print "Verbose mode" }
}
"#;
    rule().assert_violation_count_exact(source, 0);
}

#[test]
fn test_main_named_flag_with_short_with_docs() {
    let source = r#"
def main [
    --verbose (-v) # Enable verbose output
] {
    if $verbose { print "Verbose mode" }
}
"#;
    rule().assert_violation_count_exact(source, 0);
}

#[test]
fn test_main_multiple_named_flags_with_docs() {
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
    rule().assert_violation_count_exact(source, 0);
}

#[test]
fn test_main_typed_named_flag_with_docs() {
    let source = r"
def main [
    --count: int # Number of iterations
] {
    1..$count
}
";
    rule().assert_violation_count_exact(source, 0);
}

#[test]
fn test_non_main_function_not_checked() {
    let source = r#"
def helper [--verbose] {
    if $verbose { print "Verbose" }
}
"#;
    rule().assert_violation_count_exact(source, 0);
}

#[test]
fn test_main_no_named_params() {
    let source = r#"
def main [] {
    echo "Hello"
}
"#;
    rule().assert_violation_count_exact(source, 0);
}

#[test]
fn test_main_mixed_params_all_documented() {
    let source = r#"
def main [
    input # Input file
    --verbose # Enable verbose output
] {
    if $verbose { print "Processing" $input }
}
"#;
    rule().assert_violation_count_exact(source, 0);
}
