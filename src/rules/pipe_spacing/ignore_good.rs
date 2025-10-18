use crate::LintContext;

use super::rule;

#[test]
fn test_pipe_spacing() {
    let good = "ls | get name | str upcase";
    LintContext::test_with_parsed_source(good, |context| {
        assert_eq!((rule().check)(&context).len(), 0);
    });
}

#[test]
fn test_closure_pipe_not_flagged() {
    // Closures with parameters should not be flagged
    let closure_code = "let completer = {|spans| echo $spans}";
    LintContext::test_with_parsed_source(closure_code, |context| {
        assert_eq!(
            (rule().check)(&context).len(),
            0,
            "Closure parameter pipes should not be flagged"
        );
    });

    // Multiple parameter closures
    let multi_param = "{|x, y| $x + $y}";
    LintContext::test_with_parsed_source(multi_param, |context| {
        assert_eq!(
            (rule().check)(&context).len(),
            0,
            "Multi-param closure pipes should not be flagged"
        );
    });
}
