use super::RULE;

#[test]
fn test_fix_lines_each_parse() {
    let bad_code = r#"$text | lines | each {|l| $l | parse "{key}:{value}" }"#;
    RULE.assert_fixed_is(bad_code, r#"$text | lines | parse "{key}:{value}""#);
}

#[test]
fn test_fix_lines_each_parse_regex() {
    let bad_code = r#"$text | lines | each {|line| $line | parse --regex "(?P<k>.*):(?P<v>.*)" }"#;
    RULE.assert_fixed_is(
        bad_code,
        r#"$text | lines | parse --regex "(?P<k>.*):(?P<v>.*)""#,
    );
}

#[test]
fn test_fix_lines_each_parse_space() {
    let bad_code = r#"$input | lines | each {|x| $x | parse "{a} {b}" }"#;
    RULE.assert_fixed_is(bad_code, r#"$input | lines | parse "{a} {b}""#);
}
