use super::rule;
use crate::LintContext;

#[test]
fn test_ignore_parse_usage() {
    let good_code = r#"
open data.txt | parse "{name} {value}"
"#;

    LintContext::test_with_parsed_source(good_code, |context| {
        let violations = (rule().check)(&context);
        assert!(violations.is_empty(), "Should not flag proper parse usage");
    });
}

#[test]
fn test_ignore_each_without_split() {
    let good_code = r"
seq 1 10 | each { |x| $x * 2 }
";

    LintContext::test_with_parsed_source(good_code, |context| {
        let violations = (rule().check)(&context);
        assert!(violations.is_empty(), "Should not flag each without split");
    });
}

#[test]
fn test_ignore_split_without_each() {
    let good_code = r#"
"one,two,three" | split row ","
"#;

    LintContext::test_with_parsed_source(good_code, |context| {
        let violations = (rule().check)(&context);
        assert!(violations.is_empty(), "Should not flag split without each");
    });
}
