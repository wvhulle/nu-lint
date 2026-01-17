use super::RULE;

#[test]
fn main_positional_with_documentation() {
    let source = r"
def main [
    input # The input file path
] {
    echo $input
}
";
    RULE.assert_count(source, 0);
}

#[test]
fn main_multiple_positional_with_documentation() {
    let source = r"
def main [
    input # Input file
    output # Output file
] {
    echo $input | save $output
}
";
    RULE.assert_count(source, 0);
}

#[test]
fn main_typed_positional_with_documentation() {
    let source = r"
def main [
    count: int # Number of iterations
] {
    1..$count
}
";
    RULE.assert_count(source, 0);
}

#[test]
fn main_rest_positional_with_documentation() {
    let source = r"
def main [
    ...files # Files to process
] {
    $files | each { |f| open $f }
}
";
    RULE.assert_count(source, 0);
}

#[test]
fn non_main_function_ignores_documentation_requirement() {
    let source = r"
def helper [input] {
    echo $input
}
";
    RULE.assert_count(source, 0);
}

#[test]
fn main_without_parameters_passes() {
    let source = r#"
def main [] {
    echo "Hello"
}
"#;
    RULE.assert_count(source, 0);
}
