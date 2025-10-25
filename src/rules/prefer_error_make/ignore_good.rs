use super::rule;

#[test]
fn test_good_error_make() {
    let good = "error make { msg: 'Something went wrong', label: { text: 'here', span: $span } }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_simple_error_make() {
    let good = "error make 'Invalid input'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_print_without_exit() {
    let good = "print 'This is just a message'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_conditional_error() {
    let good = "if $invalid { error make 'Invalid condition' }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_exit_without_print() {
    let good = "exit 1";
    rule().assert_ignores(good);
}

#[test]
fn test_good_separate_operations() {
    let good = "print 'Processing...'; let result = some_command; exit 0";
    rule().assert_ignores(good);
}
