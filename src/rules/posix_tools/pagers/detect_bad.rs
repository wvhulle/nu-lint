use super::RULE;

#[test]
fn detect_less() {
    RULE.assert_detects("^less file.txt");
}

#[test]
fn detect_more() {
    RULE.assert_detects("^more file.txt");
}

#[test]
fn detect_less_log() {
    RULE.assert_detects("^less output.log");
}

#[test]
fn detect_more_documentation() {
    RULE.assert_detects("^more documentation.txt");
}

#[test]
fn detect_less_follow_mode() {
    RULE.assert_detects("^less -f log.txt");
}

#[test]
fn detect_less_follow_long() {
    RULE.assert_detects("^less --follow log.txt");
}

#[test]
fn detect_less_in_pipeline() {
    RULE.assert_detects("^less file.txt | head");
}

#[test]
fn detect_less_in_function() {
    let bad_code = r"
def view-file [path] {
    ^less $path
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_multiple_pagers() {
    let bad_code = r"
^less file1.txt
^more file2.txt
";
    RULE.assert_count(bad_code, 2);
}
