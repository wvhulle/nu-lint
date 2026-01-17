use super::RULE;

#[test]
fn detect_tail_with_count() {
    RULE.assert_detects("tail -10 file.txt");
}

#[test]
fn detect_tail_follow() {
    RULE.assert_detects("tail -f log.txt");
}

#[test]
fn detect_tail_follow_long() {
    RULE.assert_detects("tail --follow log.txt");
}

#[test]
fn detect_tail_follow_uppercase() {
    RULE.assert_detects("tail -F log.txt");
}

#[test]
fn detect_tail_in_pipeline() {
    RULE.assert_detects("tail -20 file.txt | head -5");
}

#[test]
fn detect_tail_in_function() {
    let bad_code = r"
def show-end [path] {
    tail -10 $path
}
";
    RULE.assert_detects(bad_code);
}
