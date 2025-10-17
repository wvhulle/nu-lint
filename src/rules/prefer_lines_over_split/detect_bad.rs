#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::Rule, rules::prefer_lines_over_split::PreferLinesOverSplit,
    };

    #[test]
    fn test_detect_split_row_with_newline_double_quotes() {
        let rule = PreferLinesOverSplit::new();

        let bad_code = r#"
$text | split row "\n"
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect split row with \\n in double quotes"
            );
        });
    }

    #[test]
    fn test_detect_split_row_with_newline_single_quotes() {
        let rule = PreferLinesOverSplit::new();

        let bad_code = r"
$text | split row '\n'
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect split row with \\n in single quotes"
            );
        });
    }

    #[test]
    fn test_detect_split_row_multiline() {
        let rule = PreferLinesOverSplit::new();

        let bad_code = r#"
def process-text [] {
    $input | split row "\n" | each { |line| $line | str trim }
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect split row with newline in function"
            );
        });
    }
}
