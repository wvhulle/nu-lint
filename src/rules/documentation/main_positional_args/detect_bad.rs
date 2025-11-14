use super::rule;

#[test]
fn main_required_positional_missing_documentation() {
    let source = r"
def main [input] {
    echo $input
}
";
    rule().assert_violation_count_exact(source, 1);
}

#[test]
fn main_multiple_positional_missing_documentation() {
    let source = r"
def main [input output] {
    echo $input | save $output
}
";
    rule().assert_violation_count_exact(source, 2);
}

#[test]
fn main_optional_positional_missing_documentation() {
    let source = r#"
def main [input? = "default"] {
    echo $input
}
"#;
    rule().assert_violation_count_exact(source, 1);
}

#[test]
fn main_rest_positional_missing_documentation() {
    let source = r"
def main [...files] {
    $files | each { |f| open $f }
}
";
    rule().assert_violation_count_exact(source, 1);
}

#[test]
fn main_typed_positional_missing_documentation() {
    let source = r"
def main [count: int] {
    1..$count
}
";
    rule().assert_violation_count_exact(source, 1);
}
