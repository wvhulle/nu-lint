#[cfg(test)]
mod tests {

    use crate::{
        context::LintContext, rule::RegexRule,
        rules::prefer_builtin_text_transforms::AvoidExternalTextTools,
    };

    #[test]
    fn test_detect_external_sed() {
        let rule = AvoidExternalTextTools::new();
        let bad_code = "^sed 's/foo/bar/' file.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external sed command"
            );
        });
    }

    #[test]
    fn test_detect_external_awk() {
        let rule = AvoidExternalTextTools::new();
        let bad_code = "^awk '{print $1}' file.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external awk command"
            );
        });
    }

    #[test]
    fn test_detect_external_cut() {
        let rule = AvoidExternalTextTools::new();
        let bad_code = "^cut -d ',' -f 1 file.csv";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external cut command"
            );
        });
    }

    #[test]
    fn test_detect_external_wc() {
        let rule = AvoidExternalTextTools::new();
        let bad_code = "^wc -l file.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external wc command"
            );
        });
    }

    #[test]
    fn test_detect_external_tr() {
        let rule = AvoidExternalTextTools::new();
        let bad_code = "^tr 'a-z' 'A-Z' file.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external tr command"
            );
        });
    }
}
