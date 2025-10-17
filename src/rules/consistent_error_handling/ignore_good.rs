#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::consistent_error_handling::ConsistentErrorHandling;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_exit_code_checked() {
        let rule = ConsistentErrorHandling::new();

        let good_code = r"
let result = (^bluetoothctl info $mac | complete)
if $result.exit_code != 0 {
    return
}
";
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag when exit_code is checked"
        );
    }

    #[test]
    fn test_no_complete_not_flagged() {
        let rule = ConsistentErrorHandling::new();

        let good_code = r"
let result = (some | regular | pipeline)
";
        let context = LintContext::test_from_source(good_code);
        assert_eq!(
            rule.check(&context).len(),
            0,
            "Should not flag non-external commands"
        );
    }
}