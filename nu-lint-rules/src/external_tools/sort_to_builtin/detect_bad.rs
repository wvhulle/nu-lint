use super::RULE;

#[test]
fn detect_sort_of_lines() {
    for code in [
        "$text | ^sort",
        "$text | ^sort -r",
        "$text | ^sort -fu",
        "$text | ^sort | ^uniq",
        "^sort names.txt",
    ] {
        RULE.assert_detects(code);
    }
}
