use super::RULE;

#[test]
fn detect_piped_head_and_tail() {
    for code in [
        "^cat log.txt | ^head",
        "^cat log.txt | ^head -n 5",
        "^cat log.txt | ^tail -3",
        "^cat log.txt | ^tail --lines=3",
        "^cat log.txt | ^tail -n +2",
    ] {
        RULE.assert_detects(code);
    }
}

#[test]
fn detect_on_single_file() {
    RULE.assert_detects("^head -n 5 log.txt");
    RULE.assert_detects("^tail log.txt");
}

#[test]
fn detect_inside_closure() {
    RULE.assert_detects("ls | each {|file| open --raw $file.name | ^head -n 1 }");
}
