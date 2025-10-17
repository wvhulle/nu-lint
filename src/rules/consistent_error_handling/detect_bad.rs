#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::consistent_error_handling::ConsistentErrorHandling;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_missing_exit_code_check() {
        let rule = ConsistentErrorHandling::new();

        let bad_code = r"
let result = (^bluetoothctl info $mac | complete)
let output = $result.stdout
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect missing exit_code check"
        );
    }

    #[test]
    fn test_risky_external_function() {
        let rule = ConsistentErrorHandling::new();

        let bad_code = r#"
def risky-external [] {
    let result = (^bluetoothctl info "AA:BB" | complete)
    print $result.stdout
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect missing exit_code check in risky-external function"
        );
    }

    #[test]
    fn test_another_risky_function() {
        let rule = ConsistentErrorHandling::new();

        let bad_code = r#"
def another-risky [] {
    let output = (^git status | complete)
    print $output.stdout
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect missing exit_code check in another-risky function"
        );
    }
}