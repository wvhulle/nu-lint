use super::RULE;

#[test]
fn detect_tac_on_file() {
    RULE.assert_detects("^tac file.txt");
}

#[test]
fn detect_piped_tac() {
    RULE.assert_detects("$text | ^tac");
}

#[test]
fn detect_tac_in_function() {
    RULE.assert_detects("def reverse-file [path] { ^tac $path }");
}
