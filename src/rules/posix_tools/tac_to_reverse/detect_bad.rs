use super::RULE;

#[test]
fn detect_simple_tac() {
    RULE.assert_detects("^tac file.txt");
}

#[test]
fn detect_tac_log_file() {
    RULE.assert_detects("^tac file.log");
}

#[test]
fn detect_tac_in_pipeline() {
    RULE.assert_detects("^tac file.txt | head -5");
}

#[test]
fn detect_tac_in_function() {
    let bad_code = r"
def reverse-file [path] {
    ^tac $path
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_tac_in_subexpression() {
    let bad_code = r"
let reversed = (^tac log.txt)
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_tac_multiple_files() {
    RULE.assert_detects("^tac file1.txt file2.txt");
}
