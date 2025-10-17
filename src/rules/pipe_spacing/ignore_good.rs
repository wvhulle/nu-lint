use super::*;

#[test]
fn test_pipe_spacing() {
    let rule = PipeSpacing::default();

    let good = "ls | get name | str upcase";
    let context = LintContext::test_from_source(good);
    assert_eq!(rule.check(&context).len(), 0);
}

#[test]
fn test_closure_pipe_not_flagged() {
    let rule = PipeSpacing::default();

    // Closures with parameters should not be flagged
    let closure_code = "let completer = {|spans| echo $spans}";
    let context = LintContext::test_from_source(closure_code);
    assert_eq!(
        rule.check(&context).len(),
        0,
        "Closure parameter pipes should not be flagged"
    );

    // Multiple parameter closures
    let multi_param = "{|x, y| $x + $y}";
    let context = LintContext::test_from_source(multi_param);
    assert_eq!(
        rule.check(&context).len(),
        0,
        "Multi-param closure pipes should not be flagged"
    );
}