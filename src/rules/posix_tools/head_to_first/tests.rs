use super::RULE;

#[test]
fn converts_head_with_count_to_first() {
    let source = "^head -n 10 file.txt | lines";
    RULE.assert_fixed_is(source, "open file.txt | lines | first 10");
}

#[test]
fn do_not_recommend_invalid_head() {
    let source = "head -20 somefile.txt";
    RULE.assert_ignores(source);
}

#[test]
fn do_not_recommend_without_lines() {
    let source = "head -n 20 somefile.txt";
    RULE.assert_ignores(source);
}

#[test]
fn replace_lines_with_file() {
    let source = "head -n 10 README.md | lines";
    RULE.assert_fixed_is(source, "open README.md | lines | first 10");
}

#[test]
fn replace_stdin_pattern() {
    let source = "cat data.txt | head -n 5 | lines";
    RULE.assert_fixed_is(source, "cat data.txt | lines | first 5");
}
