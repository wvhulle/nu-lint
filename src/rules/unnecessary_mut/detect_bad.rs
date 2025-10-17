#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rule::Rule, rules::unnecessary_mut::UnnecessaryMut};

    #[test]
    fn test_unnecessary_mut_detected() {
        let rule = UnnecessaryMut::new();

        let bad_code = r"
def process [] {
    mut x = 5
    echo $x
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect unnecessary mut");
            assert!(violations[0].message.contains("never reassigned"));
        });
    }

    #[test]
    fn test_multiple_mut_variables() {
        let rule = UnnecessaryMut::new();

        let bad_code = r"
def process [] {
    mut a = 1
    mut b = 2
    mut c = 3
    $a = 10
    $c = 30
    echo $a $b $c
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            // Only 'b' should be flagged as unnecessary mut
            assert_eq!(
                violations.len(),
                1,
                "Should flag only the one unnecessary mut"
            );
            assert!(
                violations[0].message.contains('b'),
                "Should flag variable 'b'"
            );
        });
    }

    #[test]
    fn test_unnecessary_mut_fix_provided() {
        let rule = UnnecessaryMut::new();

        let bad_code = r"
def process [] {
    mut x = 5
    echo $x
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect unnecessary mut");
            assert!(violations[0].fix.is_some(), "Should provide a fix");

            let fix = violations[0].fix.as_ref().unwrap();
            assert_eq!(fix.replacements.len(), 1, "Should have one replacement");
            // The fix should remove 'mut ' from the declaration
            assert!(
                fix.description.contains("Remove 'mut'"),
                "Fix description should mention removing 'mut'"
            );
        });
    }
}
