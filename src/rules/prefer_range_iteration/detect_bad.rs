#[cfg(test)]
mod tests {

    use crate::{
        context::LintContext, rule::RegexRule, rules::prefer_range_iteration::PreferRangeIteration,
    };

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

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect while loop with counter pattern"
            );
        });
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

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect while loop with compound assignment pattern"
            );
        });
    }
}
