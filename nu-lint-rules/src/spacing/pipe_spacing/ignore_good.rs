use super::RULE;

#[test]
fn test_proper_pipe_spacing() {
    let good = "ls | get name | str upcase";
    RULE.assert_ignores(good);
}

#[test]
fn test_closure_pipe_not_flagged() {
    // Closures with parameters should not be flagged
    let closure_code = "let completer = {|spans| echo $spans}";
    RULE.assert_ignores(closure_code);

    // Multiple parameter closures
    let multi_param = "{|x, y| $x + $y}";

    RULE.assert_ignores(multi_param);
}
