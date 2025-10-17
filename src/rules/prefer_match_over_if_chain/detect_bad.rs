#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::prefer_match_over_if_chain::PreferMatchOverIfChain;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_detect_if_chain_in_function() {
        let rule = PreferMatchOverIfChain::new();

        let bad_code = r#"
def get-color [scope: string] {
    if $scope == "wan" {
        "red"
    } else if $scope == "lan" {
        "yellow"
    } else if $scope == "local" {
        "blue"
    } else {
        "green"
    }
}
"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect if-else chain in function"
        );
    }

    #[test]
    fn test_detect_inline_if_chain() {
        let rule = PreferMatchOverIfChain::new();

        let bad_code = r#"let priority = if $level == "high" { 1 } else if $level == "medium" { 2 } else if $level == "low" { 3 } else { 0 }"#;
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect inline if-else chain"
        );
    }
}
