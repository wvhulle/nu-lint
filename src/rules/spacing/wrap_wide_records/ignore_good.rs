use super::RULE;
use crate::log::init_log;

#[test]
fn nested_record() {
    init_log();
    let code = r"let data = { id: 1, config: {nested: true} }";
    RULE.assert_ignores(code);
}
#[test]
fn ignores_short_single_line_record() {
    let code = "let point = {x: 1, y: 2}";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_multiline_record() {
    let code = r#"let config = {
    name: "app"
    version: "1.0.0"
    debug: true
}"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_empty_record() {
    let code = "let empty = {}";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_simple_record() {
    let code = r"let status = {ok: true}";
    RULE.assert_ignores(code);
}
