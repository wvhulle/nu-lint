use super::RULE;

#[test]
fn fix_open_raw_json_to_open() {
    let source = "open --raw data.json | from json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_is(source, "open data.json");
}

#[test]
fn fix_open_raw_toml_to_open() {
    let source = "open --raw config.toml | from toml";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_is(source, "open config.toml");
}

#[test]
fn fix_open_short_raw_to_open() {
    let source = "open -r data.yaml | from yaml";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_is(source, "open data.yaml");
}

#[test]
fn fix_explanation_mentions_auto_parses() {
    let source = "open --raw data.json | from json";
    RULE.assert_fix_explanation_contains(source, "auto-parses");
}

#[test]
fn fix_preserves_double_quotes() {
    let source = r#"open --raw "my file.json" | from json"#;
    RULE.assert_count(source, 1);
    RULE.assert_replacement_is(source, r#"open "my file.json""#);
}

#[test]
fn fix_preserves_single_quotes() {
    let source = "open --raw 'my file.json' | from json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_is(source, "open 'my file.json'");
}
