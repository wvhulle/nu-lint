#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::prefer_compound_assignment::PreferCompoundAssignment;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_detect_addition_assignment() {
        let rule = PreferCompoundAssignment::new();

        let bad_code = r"
mut count = 0
$count = $count + 1
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect addition assignment pattern"
        );
    }

    #[test]
    fn test_detect_subtraction_assignment() {
        let rule = PreferCompoundAssignment::new();

        let bad_code = r"
mut count = 0
$count = $count - 5
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect subtraction assignment pattern"
        );
    }

    #[test]
    fn test_detect_multiplication_assignment() {
        let rule = PreferCompoundAssignment::new();

        let bad_code = r"
mut count = 0
$count = $count * 2
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect multiplication assignment pattern"
        );
    }
}
