#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::Rule, rules::prefer_lines_over_split::PreferLinesOverSplit,
    };

    #[test]
    fn test_ignore_lines_usage() {
        let rule = PreferLinesOverSplit::new();

        let good_code = r"
open file.txt | lines
";

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(violations.is_empty(), "Should not flag proper lines usage");
        });
    }

    #[test]
    fn test_ignore_split_row_with_other_delimiter() {
        let rule = PreferLinesOverSplit::new();

        let good_code = r#"
"a,b,c" | split row ","
"#;

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag split row with non-newline delimiter"
            );
        });
    }

    #[test]
    fn test_ignore_split_row_with_colon() {
        let rule = PreferLinesOverSplit::new();

        let good_code = r#"
"PATH=/usr/bin:/bin" | split row ":"
"#;

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag split row with colon delimiter"
            );
        });
    }

    #[test]
    fn test_ignore_split_row_with_space() {
        let rule = PreferLinesOverSplit::new();

        let good_code = r#"
"one two three" | split row " "
"#;

        LintContext::test_with_parsed_source(good_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.is_empty(),
                "Should not flag split row with space delimiter"
            );
        });
    }
}
