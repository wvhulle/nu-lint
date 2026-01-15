use super::RULE;

#[test]
fn ignore_variadic_param() {
    RULE.assert_ignores(
        r#"
def process [...items: string] {
    $items | each { print $in }
}
"#,
    );
}

#[test]
fn ignore_list_not_last_positional() {
    RULE.assert_ignores(
        r#"
def cmd [items: list<string>, count: int] {
    $items | take $count
}
"#,
    );
}

#[test]
fn ignore_list_with_rest_param() {
    RULE.assert_ignores(
        r#"
def cmd [items: list<string>, ...rest: any] {
    $items | append $rest
}
"#,
    );
}

#[test]
fn ignore_no_list_param() {
    RULE.assert_ignores(
        r#"
def greet [name: string] {
    print $"Hello ($name)"
}
"#,
    );
}

#[test]
fn ignore_record_param() {
    RULE.assert_ignores(
        r#"
def process [config: record] {
    $config.name
}
"#,
    );
}

#[test]
fn ignore_multiple_non_list_params() {
    RULE.assert_ignores(
        r#"
def cmd [a: string, b: int, c: bool] {
    if $c { print $"($a): ($b)" }
}
"#,
    );
}

#[test]
fn ignore_wrapped_command() {
    // --wrapped commands require rest parameters by design
    RULE.assert_ignores(
        r#"
def --wrapped my_ls [...rest] {
    ^ls ...$rest
}
"#,
    );
}

#[test]
fn ignore_nested_list_type() {
    // list<list<T>> indicates intentional nested structure (matrix, grouped data)
    // Converting would change semantics
    RULE.assert_ignores(
        r#"
def process [rows: list<list<string>>] {
    $rows | each { |row| $row | str join ", " }
}
"#,
    );
}
