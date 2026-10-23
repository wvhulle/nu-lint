use super::RULE;

#[test]
fn ignore_fully_annotated_params() {
    let good_code = r#"
def greet [name: string] {
    print $"Hello ($name)"
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_multiple_annotated_params() {
    let good_code = r"
def add [x: int, y: int] {
    $x + $y
}
";

    RULE.assert_ignores(good_code);
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

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_variadic_params() {
    let good_code = r"
def variadic [...args: list] {
    print $args
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_function_without_params() {
    let good_code = r"
def hello [] {
    print 'Hello world'
}
";

    RULE.assert_ignores(good_code);
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

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_untyped_wrapped_rest() {
    let good_code = r"
def --wrapped ezal [...rest] {
    if '-G' in $rest {
        ^eza ...$rest
    } else {
        ^eza -l --icons ...$rest
    }
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_string_wrapped_rest() {
    let good_code = r"
def --wrapped dzal [...rest: string] {
    ^git ...$rest
}
";
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_wrapped_rest_with_explicit_any_left_to_parser_error() {
    let good_code = r"
def --wrapped main [...rest: any] {
    ^ezal ...$rest
    print end
}
";
    RULE.assert_ignores(good_code);
}
