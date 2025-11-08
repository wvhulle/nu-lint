use super::rule;

#[test]
fn test_ignore_fully_annotated_params() {
    let good_code = r#"
def greet [name: string] {
    print $"Hello ($name)"
}
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_multiple_annotated_params() {
    let good_code = r"
def add [x: int, y: int] {
    $x + $y
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_flags() {
    let good_code = r"
def process [
    input: string
    --verbose
    --output: string
] {
    print $input
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_spread_params() {
    let good_code = r"
def variadic [...args: list] {
    print $args
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_no_params() {
    let good_code = r"
def hello [] {
    print 'Hello world'
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_complex_types() {
    let good_code = r"
def process [
    data: list<string>
    options: record
] {
    print $data
}
";

    rule().assert_ignores(good_code);
}
