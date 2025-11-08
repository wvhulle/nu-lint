use super::rule;

#[test]
fn ignores_short_single_line_record() {
    let code = "let point = {x: 1, y: 2}";
    rule().assert_ignores(code);
}

#[test]
fn ignores_multiline_record() {
    let code = r#"let config = {
    name: "app"
    version: "1.0.0"
    debug: true
}"#;
    rule().assert_ignores(code);
}

#[test]
fn ignores_empty_record() {
    let code = "let empty = {}";
    rule().assert_ignores(code);
}

#[test]
fn ignores_simple_record() {
    let code = r"let status = {ok: true}";
    rule().assert_ignores(code);
}
