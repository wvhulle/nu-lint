#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::AstRule,
        rules::prefer_parse_over_each_split::PreferParseOverEachSplit,
    };

    #[test]
    fn test_detect_each_with_split_row() {
        let rule = PreferParseOverEachSplit;

        let bad_code = r#"
$data | lines | each { |line| $line | split row " " }
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect each with split row");
        });
    }

    #[test]
    fn test_detect_each_with_split() {
        let rule = PreferParseOverEachSplit;

        let bad_code = r#"
$lines | each { |l| $l | split " " }
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect each with split");
        });
    }

    #[test]
    fn test_detect_nested_split_in_each() {
        let rule = PreferParseOverEachSplit;

        let bad_code = r#"
$text | lines | each { |line|
    let parts = ($line | split row ":")
    $parts
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(!violations.is_empty(), "Should detect nested split in each");
        });
    }
}
