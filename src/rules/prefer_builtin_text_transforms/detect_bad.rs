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
    fn test_detect_external_sed_in_place() {
        let rule = AvoidExternalTextTools::new();
        let bad_code = "^sed -i 's/old/new/g' *.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external sed with in-place editing"
            );
        });
    }

    #[test]
    fn test_detect_external_sed_pipeline() {
        let rule = AvoidExternalTextTools::new();
        let bad_code = "cat file.txt | ^sed 's/pattern/replacement/'";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external sed in pipeline"
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
    fn test_detect_external_awk_field_separator() {
        let rule = AvoidExternalTextTools::new();
        let bad_code = "^awk -F',' '{print $2}' data.csv";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external awk with field separator"
            );
        });
    }

    #[test]
    fn test_detect_external_awk_filtering() {
        let rule = AvoidExternalTextTools::new();
        let bad_code = "^awk '$3 > 100 {print $0}' data.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external awk with filtering"
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

    #[test]
    fn test_detect_external_tr_delete() {
        let rule = AvoidExternalTextTools::new();
        let bad_code = "^tr -d '\n' < file.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external tr with delete option"
            );
        });
    }

    #[test]
    fn test_detect_external_wc_lines_words() {
        let rule = AvoidExternalTextTools::new();
        let bad_code = "^wc -lw *.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external wc with multiple flags"
            );
        });
    }
}
