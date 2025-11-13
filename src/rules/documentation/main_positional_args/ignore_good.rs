use super::rule;

#[test]
fn test_main_positional_with_docs() {
    let source = r"
def main [
    input # The input file path
] {
    echo $input
}
";
    rule().assert_violation_count_exact(source, 0);
}

#[test]
fn test_main_multiple_positional_with_docs() {
    let source = r"
def main [
    input # Input file
    output # Output file
] {
    echo $input | save $output
}
";
    rule().assert_violation_count_exact(source, 0);
}

#[test]
fn test_main_typed_positional_with_docs() {
    let source = r"
def main [
    count: int # Number of iterations
] {
    1..$count
}
";
    rule().assert_violation_count_exact(source, 0);
}

#[test]
fn test_main_rest_positional_with_docs() {
    let source = r"
def main [
    ...files # Files to process
] {
    $files | each { |f| open $f }
}
";
    rule().assert_violation_count_exact(source, 0);
}

#[test]
fn test_non_main_function_not_checked() {
    let source = r"
def helper [input] {
    echo $input
}
";
    rule().assert_violation_count_exact(source, 0);
}

#[test]
fn test_main_no_params() {
    let source = r#"
def main [] {
    echo "Hello"
}
"#;
    rule().assert_violation_count_exact(source, 0);
}
