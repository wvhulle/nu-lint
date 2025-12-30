use super::RULE;

#[test]
fn detect_bat() {
    RULE.assert_detects("^bat file.txt");
}

#[test]
fn detect_batcat() {
    RULE.assert_detects("^batcat file.txt");
}

#[test]
fn detect_bat_json() {
    RULE.assert_detects("^bat data.json");
}

#[test]
fn detect_bat_in_pipeline() {
    RULE.assert_detects("^bat file.txt | head -5");
}

#[test]
fn detect_bat_in_function() {
    let bad_code = r"
def view-file [path] {
    ^bat $path
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_multiple_bat_calls() {
    let bad_code = r"
^bat file1.txt
^batcat file2.txt
";
    RULE.assert_count(bad_code, 2);
}
