use super::rule;

#[test]
fn test_fix_simple_split_get_space_delimiter() {
    let bad_code = r#""hello world" | split row " " | get 0"#;
    rule().assert_replacement_is(bad_code, r#"parse "{field0} {field1}""#);
}

#[test]
fn test_fix_simple_split_get_colon_delimiter() {
    let bad_code = r#""key:value" | split row ":" | get 0"#;
    rule().assert_replacement_is(bad_code, r#"parse "{field0}:{field1}""#);
}

#[test]
fn test_fix_split_get_higher_index() {
    let bad_code = r#""a:b:c:d" | split row ":" | get 2"#;
    rule().assert_replacement_is(bad_code, r#"parse "{field0}:{field1}:{field2}:{field3}""#);
}

#[test]
fn test_fix_split_skip_pattern() {
    let bad_code = r#""a b c" | split row " " | skip 1"#;
    rule().assert_replacement_is(bad_code, r#"parse "{field0} {field1} {field2}""#);
}

#[test]
fn test_fix_requires_regex_for_dot_delimiter() {
    let bad_code = r#""file.txt" | split row "." | get 0"#;
    rule().assert_replacement_is(bad_code, r"parse --regex '(?P<field0>.*)\.(?P<field1>.*)'");
}

#[test]
fn test_fix_requires_regex_for_pipe_delimiter() {
    let bad_code = r#""a|b" | split row "|" | get 0"#;
    rule().assert_replacement_is(bad_code, r"parse --regex '(?P<field0>.*)\|(?P<field1>.*)'");
}

#[test]
fn test_fix_requires_regex_for_plus_delimiter() {
    let bad_code = r#""1+2" | split row "+" | get 0"#;
    rule().assert_replacement_is(bad_code, r"parse --regex '(?P<field0>.*)\+(?P<field1>.*)'");
}

#[test]
fn test_fix_requires_regex_for_star_delimiter() {
    let bad_code = r#""a*b" | split row "*" | get 0"#;
    rule().assert_replacement_is(bad_code, r"parse --regex '(?P<field0>.*)\*(?P<field1>.*)'");
}

#[test]
fn test_fix_requires_regex_for_parentheses_delimiter() {
    let bad_code = r#""func(arg)" | split row "(" | get 0"#;
    rule().assert_replacement_is(bad_code, r"parse --regex '(?P<field0>.*)\((?P<field1>.*)'");
}

#[test]
fn test_fix_requires_regex_for_brackets_delimiter() {
    let bad_code = r#""arr[0]" | split row "[" | get 0"#;
    rule().assert_replacement_is(bad_code, r"parse --regex '(?P<field0>.*)\[(?P<field1>.*)'");
}

#[test]
fn test_fix_simple_delimiter_no_regex() {
    let bad_code = r#""a-b" | split row "-" | get 0"#;
    rule().assert_replacement_is(bad_code, r#"parse "{field0}-{field1}""#);
}

#[test]
fn test_fix_tab_delimiter() {
    let bad_code = "\"a\\tb\" | split row \"\\t\" | get 0";
    rule().assert_replacement_contains(bad_code, "parse");
    rule().assert_replacement_contains(bad_code, "{field0}");
}

#[test]
fn test_fix_explanation_mentions_parse() {
    let bad_code = r#""hello world" | split row " " | get 0"#;
    rule().assert_fix_explanation_contains(bad_code, "parse");
}

#[test]
fn test_fix_explanation_mentions_replace() {
    let bad_code = r#""hello world" | split row " " | get 0"#;
    rule().assert_fix_explanation_contains(bad_code, "Replace");
}
