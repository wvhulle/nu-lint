use super::rule;
use crate::LintContext;

#[test]
fn test_good_brace_spacing_with_spaces() {
    // According to style guide: { x: 1, y: 2 } is correct (consistent spaces)
    let good = "{ key: value }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(violations.len(), 0);
    });
}

#[test]
fn test_good_brace_spacing_without_spaces() {
    // According to style guide: {x: 1, y: 2} is also correct (no spaces)
    let good = "{x: 1, y: 2}";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(
            violations.len(),
            0,
            "No spaces inside braces is valid according to style guide"
        );
    });
}

#[test]
fn test_good_closure_without_space() {
    // According to style guide: {|el| ...} is correct (no space before |)
    let good = "[[status]; [UP]] | all {|el| $el.status == UP }";
    LintContext::test_with_parsed_source(good, |context| {
        let violations = (rule().check)(&context);
        assert_eq!(
            violations.len(),
            0,
            "No space before closure parameters is correct"
        );
    });
}
