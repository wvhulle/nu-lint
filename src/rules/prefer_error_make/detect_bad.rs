use super::rule;
use crate::LintContext;

#[test]
fn test_detect_print_exit_pattern() {
    let bad_code = r#"
def bad-error [] {
    print "Error occurred"
    exit 1
}
"#;

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(!(rule().check)(&context).is_empty());
    });
}
