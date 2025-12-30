use super::RULE;

#[test]
fn fix_removes_from_json() {
    let source = "open data.json | from json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_is(source, "open data.json");
}

#[test]
fn fix_removes_from_toml() {
    let source = "open config.toml | from toml";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_is(source, "open config.toml");
}

#[test]
fn fix_explanation_mentions_already_parses() {
    let source = "open data.json | from json";
    RULE.assert_fix_explanation_contains(source, "already parses");
}

#[test]
fn fix_preserves_double_quotes() {
    let source = r#"open "my file.json" | from json"#;
    RULE.assert_count(source, 1);
    RULE.assert_replacement_is(source, r#"open "my file.json""#);
}

#[test]
fn fix_preserves_single_quotes() {
    let source = "open 'my file.json' | from json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_is(source, "open 'my file.json'");
}
