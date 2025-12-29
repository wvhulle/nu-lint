use super::RULE;

#[test]
fn test_ignore_regex_with_character_classes() {
    let good = r"'foo123' | parse --regex '(?P<word>\\w+)(?P<num>\\d+)'";
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_regex_with_whitespace() {
    let good = r"'foo   bar' | parse --regex '\\s+(?P<word>\\w+)\\s+'";
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_regex_with_quantifiers() {
    let good = r"'aaabbb' | parse --regex '(?P<a>a+)(?P<b>b+)'";
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_regex_with_anchors() {
    let good = r"'test' | parse --regex '^(?P<word>.*)$'";
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_regex_with_alternation() {
    let good = r"'foo' | parse --regex '(?P<word>foo|bar)'";
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_regex_with_optional() {
    let good = r"'test' | parse --regex '(?P<word>test)?'";
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_simple_parse() {
    let good = r#"'key:value' | parse '{key}:{value}'"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_parse_without_regex() {
    let good = r#"'name john' | parse '{label} {value}'"#;
    RULE.assert_ignores(good);
}
