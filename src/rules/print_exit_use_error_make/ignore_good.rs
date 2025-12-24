use super::RULE;

#[test]
fn test_good_error_make() {
    let good = "error make { msg: 'Something went wrong', label: { text: 'here', span: $span } }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_simple_error_make() {
    let good = "error make 'Invalid input'";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_print_without_exit() {
    let good = "print 'This is just a message'";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_conditional_error() {
    let good = "if $invalid { error make 'Invalid condition' }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_exit_without_print() {
    let good = "exit 1";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_separate_operations() {
    let good = "print 'Processing...'; let result = some_command; exit 0";
    RULE.assert_ignores(good);
}
