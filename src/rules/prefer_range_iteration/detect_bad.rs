#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::prefer_range_iteration::PreferRangeIteration;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_detect_while_loop_with_counter() {
        let rule = PreferRangeIteration::new();

        let bad_code = r"
mut attempts = 0
while $attempts < 10 {
    print $'Attempt ($attempts)'
    $attempts = $attempts + 1
}
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect while loop with counter pattern"
        );
    }

    #[test]
    fn test_detect_while_loop_with_compound_assignment() {
        let rule = PreferRangeIteration::new();

        let bad_code = r"
mut count = 0
while $count < 5 {
    do_something
    $count += 1
}
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect while loop with compound assignment pattern"
        );
    }
}
