use super::rule;

#[test]
fn ignore_fully_annotated_params() {
    let good_code = r#"
def greet [name: string] {
    print $"Hello ($name)"
}
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn ignore_multiple_annotated_params() {
    let good_code = r"
def add [x: int, y: int] {
    $x + $y
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn ignore_function_with_flags() {
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
fn ignore_variadic_params() {
    let good_code = r"
def variadic [...args: list] {
    print $args
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn ignore_function_without_params() {
    let good_code = r"
def hello [] {
    print 'Hello world'
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn ignore_complex_type_annotations() {
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
