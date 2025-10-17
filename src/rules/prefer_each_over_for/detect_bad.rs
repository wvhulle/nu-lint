#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::prefer_each_over_for::PreferEachOverFor;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_detect_for_loop_with_echo() {
        let rule = PreferEachOverFor::new();

        let bad_code = r"
for item in [1 2 3 4 5] {
    echo ($item * 2)
}
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect for loop with echo"
        );
    }

    #[test]
    fn test_detect_for_loop_with_ls() {
        let rule = PreferEachOverFor::new();

        let bad_code = r"
for file in (ls | get name) {
    echo $file
}
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect for loop with ls output"
        );
    }
}
