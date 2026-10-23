use super::RULE;

#[test]
fn ignore_open_raw_from_json() {
    // This is handled by a different rule (open_raw_from_to_open)
    RULE.assert_ignores("open --raw data.json | from json");
}

#[test]
fn ignore_open_different_format() {
    // Unknown extension, open returns raw, from json might be intentional
    RULE.assert_ignores("open data.txt | from json");
}

#[test]
fn ignore_just_open() {
    RULE.assert_ignores("open data.json");
}

#[test]
fn ignore_just_from() {
    RULE.assert_ignores("$content | from json");
}

#[test]
fn ignore_open_unknown_extension() {
    RULE.assert_ignores("open data.dat | from json");
}

#[test]
fn ignore_mismatched_format() {
    // open data.json parses as JSON, then from toml would error differently
    // but that's a different kind of error - we only catch matching formats
    RULE.assert_ignores("open data.json | from toml");
}

#[test]
fn ignore_http_get_from() {
    RULE.assert_ignores("http get https://api.example.com/data | from json");
}
