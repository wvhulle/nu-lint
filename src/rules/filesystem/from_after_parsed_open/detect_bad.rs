use super::RULE;

#[test]
fn detect_open_json_from_json() {
    RULE.assert_detects("open data.json | from json");
}

#[test]
fn detect_open_toml_from_toml() {
    RULE.assert_detects("open config.toml | from toml");
}

#[test]
fn detect_open_yaml_from_yaml() {
    RULE.assert_detects("open settings.yaml | from yaml");
}

#[test]
fn detect_open_yml_from_yaml() {
    RULE.assert_detects("open settings.yml | from yaml");
}

#[test]
fn detect_open_csv_from_csv() {
    RULE.assert_detects("open data.csv | from csv");
}

#[test]
fn detect_open_xml_from_xml() {
    RULE.assert_detects("open data.xml | from xml");
}

#[test]
fn detect_open_nuon_from_nuon() {
    RULE.assert_detects("open data.nuon | from nuon");
}

#[test]
fn detect_in_function() {
    let bad_code = r"
def load-config [] {
    open config.json | from json
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_with_further_pipeline() {
    RULE.assert_detects("open data.json | from json | get field");
}

#[test]
fn detect_double_quoted_filename() {
    RULE.assert_detects(r#"open "my file.json" | from json"#);
}

#[test]
fn detect_single_quoted_filename() {
    RULE.assert_detects("open 'my file.json' | from json");
}
