#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rule::AstRule, rules::unnecessary_mut::UnnecessaryMut};

    #[test]
    fn test_unnecessary_mut_fix_simple() {
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

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(fix.replacements.len(), 1, "Should have one replacement");
            assert_eq!(
                fix.replacements[0].new_text, "",
                "Should remove 'mut ' keyword"
            );
            assert!(
                fix.description.contains("Remove 'mut'"),
                "Fix description should mention removing mut"
            );
        });
    }

    #[test]
    fn test_unnecessary_mut_fix_multiple_variables() {
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

            let violation = &violations[0];
            assert!(violation.fix.is_some(), "Should provide a fix");
            assert!(violation.message.contains('b'), "Should flag variable 'b'");

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(
                fix.replacements[0].new_text, "",
                "Should remove 'mut ' keyword"
            );
        });
    }

    #[test]
    fn test_unnecessary_mut_fix_nested_function() {
        let rule = UnnecessaryMut::new();
        let bad_code = r"
def outer [] {
    def inner [] {
        mut x = 42
        $x
    }
    inner
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect unnecessary mut in nested function"
            );

            let violation = &violations[0];
            assert!(
                violation.fix.is_some(),
                "Should provide a fix for nested function"
            );

            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(
                fix.replacements[0].new_text, "",
                "Should remove 'mut ' keyword"
            );
        });
    }

    #[test]
    fn test_unnecessary_mut_fix_description() {
        let rule = UnnecessaryMut::new();
        let bad_code = r"
def test [] {
    mut unused = 123
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect unnecessary mut");

            let violation = &violations[0];
            let fix = violation.fix.as_ref().unwrap();

            assert!(
                fix.description.contains("Remove 'mut' keyword"),
                "Fix description should mention removing mut keyword"
            );
            assert!(
                fix.description.contains("unused"),
                "Fix description should mention the variable name"
            );
        });
    }

    #[test]
    fn test_unnecessary_mut_no_fix_for_reassigned() {
        let rule = UnnecessaryMut::new();
        let bad_code = r"
def process [] {
    mut x = 5
    $x = 10
    echo $x
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag reassigned mutable variable"
            );
        });
    }
}
