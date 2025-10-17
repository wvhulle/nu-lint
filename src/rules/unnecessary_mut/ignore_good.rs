#[cfg(test)]
mod tests {
    use crate::{context::LintContext, rule::Rule, rules::unnecessary_mut::UnnecessaryMut};

    #[test]
    fn test_necessary_mut_not_flagged() {
        let rule = UnnecessaryMut::new();

        let good_code = r"
def fibonacci [n: int] {
    mut a = 0
    mut b = 1
    for _ in 2..=$n {
        let c = $a + $b
        $a = $b
        $b = $c
    }
    $b
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag mut variables that are reassigned"
            );
        });
    }

    #[test]
    fn test_immutable_variable_not_flagged() {
        let rule = UnnecessaryMut::new();

        let good_code = r"
def process [] {
    let x = 5
    echo $x
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(violations.is_empty(), "Should not flag immutable variables");
        });
    }

    #[test]
    fn test_mut_with_compound_assignment() {
        let rule = UnnecessaryMut::new();

        let good_code = r"
def increment [] {
    mut counter = 0
    $counter += 1
    echo $counter
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag mut with compound assignment"
            );
        });
    }

    #[test]
    fn test_underscore_prefixed_mut_not_flagged() {
        let rule = UnnecessaryMut::new();

        let good_code = r#"
def process [] {
    mut _temp = 5
    echo "done"
}
"#;

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag underscore-prefixed mut variables"
            );
        });
    }

    #[test]
    fn test_necessary_mut_no_fix() {
        let rule = UnnecessaryMut::new();

        let good_code = r"
def increment [] {
    mut counter = 0
    $counter += 1
    echo $counter
}
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag necessary mut variables"
            );
        });
    }
}
