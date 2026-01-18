use super::RULE;

#[test]
fn fix_bat_to_open_raw() {
    let source = "^bat file.txt";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "open --raw file.txt");
}

#[test]
fn fix_batcat_to_open_raw() {
    let source = "^batcat file.txt";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "open --raw file.txt");
}

#[test]
fn fix_bat_json_to_open() {
    let source = "^bat data.json";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, "open data.json");
}

#[test]
fn fix_bat_toml_to_open() {
    let source = "^bat config.toml";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_is(source, "open config.toml");
}

#[test]
fn fix_bat_yaml_to_open() {
    let source = "^bat config.yaml";
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "open config.yaml");
}

#[test]
fn fix_bat_csv_to_open() {
    let source = "^bat data.csv";
    RULE.assert_fixed_contains(source, "open data.csv");
}

#[test]
fn fix_preserves_filename() {
    let source = "^bat my-complex-file.log";
    RULE.assert_fixed_contains(source, "my-complex-file.log");
}
