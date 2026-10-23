use super::RULE;

#[test]
fn fixes_interpolation_around_variable() {
    let bad = r#"let x = "hello"; echo $"($x)""#;
    let good = "$x";
    assert_eq!(RULE.first_replacement_text(bad), good);
}

#[test]
fn fixes_interpolation_around_cell_path() {
    let bad = r#"let rec = {name: "test"}; echo $"($rec.name)""#;
    let good = "$rec.name";
    assert_eq!(RULE.first_replacement_text(bad), good);
}

#[test]
fn fixes_single_quote_interpolation() {
    let bad = r#"let x = "hello"; echo $'($x)'"#;
    let good = "$x";
    assert_eq!(RULE.first_replacement_text(bad), good);
}
