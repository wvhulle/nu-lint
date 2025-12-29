use super::RULE;

#[test]
fn detects_interpolation_around_variable() {
    let code = r#"let x = "hello"; echo $"($x)""#;
    RULE.assert_detects(code);
}

#[test]
fn detects_interpolation_around_cell_path() {
    let code = r#"let rec = {name: "test"}; echo $"($rec.name)""#;
    RULE.assert_detects(code);
}

#[test]
fn detects_single_quote_interpolation() {
    let code = r#"let x = "hello"; echo $'($x)'"#;
    RULE.assert_detects(code);
}
