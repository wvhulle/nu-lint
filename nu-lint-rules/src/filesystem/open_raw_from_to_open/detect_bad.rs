use super::RULE;

#[test]
fn detect_open_raw_json_from_json() {
    RULE.assert_detects("open --raw data.json | from json");
}

#[test]
fn detect_open_raw_short_json_from_json() {
    RULE.assert_detects("open -r data.json | from json");
}

#[test]
fn detect_open_raw_toml_from_toml() {
    RULE.assert_detects("open --raw config.toml | from toml");
}

#[test]
fn detect_open_raw_yaml_from_yaml() {
    RULE.assert_detects("open --raw settings.yaml | from yaml");
}

#[test]
fn detect_open_raw_csv_from_csv() {
    RULE.assert_detects("open --raw data.csv | from csv");
}

#[test]
fn detect_in_function() {
    let bad_code = r"
def load-config [] {
    open --raw config.json | from json
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_with_further_pipeline() {
    RULE.assert_detects("open --raw data.json | from json | get field");
}

#[test]
fn detect_double_quoted_filename() {
    RULE.assert_detects(r#"open --raw "my file.json" | from json"#);
}

#[test]
fn detect_single_quoted_filename() {
    RULE.assert_detects("open --raw 'my file.json' | from json");
}
