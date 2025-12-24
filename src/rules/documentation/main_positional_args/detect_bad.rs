use super::RULE;

#[test]
fn main_required_positional_missing_documentation() {
    let source = r"
def main [input] {
    echo $input
}
";
    RULE.assert_count(source, 1);
}

#[test]
fn main_multiple_positional_missing_documentation() {
    let source = r"
def main [input output] {
    echo $input | save $output
}
";
    RULE.assert_count(source, 2);
}

#[test]
fn main_optional_positional_missing_documentation() {
    let source = r#"
def main [input? = "default"] {
    echo $input
}
"#;
    RULE.assert_count(source, 1);
}

#[test]
fn main_rest_positional_missing_documentation() {
    let source = r"
def main [...files] {
    $files | each { |f| open $f }
}
";
    RULE.assert_count(source, 1);
}

#[test]
fn main_typed_positional_missing_documentation() {
    let source = r"
def main [count: int] {
    1..$count
}
";
    RULE.assert_count(source, 1);
}
