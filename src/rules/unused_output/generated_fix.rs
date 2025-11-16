use super::rule;

#[test]
fn test_each_with_output_and_ignore_removed() {
    let bad_code = r#"
def test [] {
    0..5 | each {|i| $"Test number: ($i)" } | ignore
}
"#;
    rule().assert_fix_contains(bad_code, "each");
    rule().assert_fix_contains(bad_code, "$\"Test number: ($i)\"");
}

#[test]
fn test_simple_echo_ignore() {
    let bad_code = "echo 'hello' | ignore";
    rule().assert_fix_contains(bad_code, "echo 'hello'");
}

#[test]
fn test_fix_description_mentions_removal() {
    let bad_code = "ls | ignore";
    rule().assert_fix_explanation_contains(bad_code, "Remove");
    rule().assert_fix_explanation_contains(bad_code, "ignore");
}

#[test]
fn test_each_with_string_output() {
    let bad_code = r#"
[1 2 3] | each {|x| $"Item ($x)" } | ignore
"#;
    rule().assert_fix_contains(bad_code, "each");
    rule().assert_fix_contains(bad_code, "$\"Item ($x)\"");
}

#[test]
fn test_complex_pipeline_with_where() {
    let bad_code = r"
0..10 | where $it mod 2 == 0 | each {|x| $x * 2 } | ignore
";
    rule().assert_fix_contains(bad_code, "where");
    rule().assert_fix_contains(bad_code, "each");
}

#[test]
fn test_fix_preserves_pipeline() {
    let bad_code = r"
def main [] {
    if true {
        ls | ignore
    }
}
";
    rule().assert_fix_contains(bad_code, "ls");
}

#[test]
fn test_multiple_violations_have_fixes() {
    let bad_code = r#"
def test [] {
    ls | ignore
    echo "test" | ignore
}
"#;
    rule().assert_violation_count_exact(bad_code, 2);
}

#[test]
fn test_http_get_with_fix() {
    let bad_code = "http get https://example.com | ignore";
    rule().assert_fix_contains(bad_code, "http get");
    rule().assert_fix_contains(bad_code, "https://example.com");
}

#[test]
fn test_external_curl_with_fix() {
    let bad_code = "curl -X GET https://api.example.com | ignore";
    rule().assert_fix_contains(bad_code, "curl");
    rule().assert_fix_contains(bad_code, "https://api.example.com");
}
