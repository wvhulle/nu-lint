use super::RULE;

#[test]
fn test_ignore_lines_without_each() {
    let good = r#"$text | lines"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_lines_each_with_other_processing() {
    // Closure does more than just parse
    let good = r#"$text | lines | each {|l| $l | str trim | parse "{k}:{v}" }"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_lines_each_without_parse() {
    let good = r#"$text | lines | each {|l| $l | str upcase }"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_lines_parse_direct() {
    // Already using the optimal pattern
    let good = r#"$text | lines | parse "{key}:{value}""#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_each_without_lines() {
    let good = r#"$list | each {|x| $x | parse "{a}:{b}" }"#;
    RULE.assert_ignores(good);
}
