use super::rule;
use crate::LintContext;

#[test]
fn test_hyphenated_command_not_flagged() {
    let good_code = r"def my-command [param: string] {
    echo $param
}";

    LintContext::test_with_parsed_source(good_code, |context| {
        assert_eq!(
            (rule().check)(&context).len(),
            0,
            "Should not flag hyphenated names"
        );
    });
}

#[test]
fn test_single_word_command_not_flagged() {
    let good_code = r"def command [param: string] {
    echo $param
}";

    LintContext::test_with_parsed_source(good_code, |context| {
        assert_eq!(
            (rule().check)(&context).len(),
            0,
            "Should not flag single-word names"
        );
    });
}
