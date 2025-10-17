#[cfg(test)]
mod tests {

    use crate::{
        context::LintContext, rule::Rule, rules::max_positional_params::MaxPositionalParams,
    };

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

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect function with too many positional parameters"
            );
        });
    }

    #[test]
    fn test_detect_too_many_positional_params_simple() {
        let rule = MaxPositionalParams::new();
        let bad_code = r"
def too-many [a: int, b: int, c: int, d: int, e: int] {
    print $a
}
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect function with too many simple positional parameters"
            );
        });
    }
}
