use super::RULE;

#[test]
fn ignore_open_without_raw() {
    // This is handled by from_after_parsed_open rule
    RULE.assert_ignores("open data.json | from json");
}

#[test]
fn ignore_open_raw_unknown_extension() {
    // Unknown extension - from json might be intentional
    RULE.assert_ignores("open --raw data.dat | from json");
}

#[test]
fn ignore_just_open_raw() {
    RULE.assert_ignores("open --raw data.json");
}

#[test]
fn ignore_just_from() {
    RULE.assert_ignores("$content | from json");
}

#[test]
fn ignore_mismatched_format() {
    // File is .json but from toml - user might know what they're doing
    RULE.assert_ignores("open --raw data.json | from toml");
}

#[test]
fn ignore_http_get_from() {
    RULE.assert_ignores("http get https://api.example.com/data | from json");
}

#[test]
fn ignore_open_raw_txt_from_json() {
    // .txt is not auto-parsed, so --raw | from json is valid
    RULE.assert_ignores("open --raw data.txt | from json");
}
