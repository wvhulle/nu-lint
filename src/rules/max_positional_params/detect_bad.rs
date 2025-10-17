#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::max_positional_params::MaxPositionalParams;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_detect_too_many_positional_params_complex() {
        let rule = MaxPositionalParams::new();

        let bad_code = r"
def complex-command [
    param1: string
    param2: int
    param3: bool
    param4: string
] {
    print $param1
}
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect function with too many positional parameters"
        );
    }

    #[test]
    fn test_detect_too_many_positional_params_simple() {
        let rule = MaxPositionalParams::new();

        let bad_code = r"
def too-many [a: int, b: int, c: int, d: int, e: int] {
    print $a
}
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect function with too many simple positional parameters"
        );
    }
}
