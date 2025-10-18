use super::rule;
use crate::LintContext;

#[test]
fn test_detect_addition_assignment() {
    let bad_code = r"
mut count = 0
$count = $count + 1
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect addition assignment pattern"
        );
    });
}

#[test]
fn test_detect_subtraction_assignment() {
    let bad_code = r"
mut count = 0
$count = $count - 5
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect subtraction assignment pattern"
        );
    });
}

#[test]
fn test_detect_multiplication_assignment() {
    let bad_code = r"
mut count = 0
$count = $count * 2
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect multiplication assignment pattern"
        );
    });
}
