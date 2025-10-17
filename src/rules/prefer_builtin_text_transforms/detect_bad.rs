#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::prefer_builtin_text_transforms::PreferBuiltinTextTransforms;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_detect_external_sed() {
        let rule = PreferBuiltinTextTransforms::new();

        let bad_code = "^sed 's/foo/bar/' file.txt";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external sed command"
        );
    }

    #[test]
    fn test_detect_external_awk() {
        let rule = PreferBuiltinTextTransforms::new();

        let bad_code = "^awk '{print $1}' file.txt";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external awk command"
        );
    }

    #[test]
    fn test_detect_external_cut() {
        let rule = PreferBuiltinTextTransforms::new();

        let bad_code = "^cut -d ',' -f 1 file.csv";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external cut command"
        );
    }

    #[test]
    fn test_detect_external_wc() {
        let rule = PreferBuiltinTextTransforms::new();

        let bad_code = "^wc -l file.txt";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external wc command"
        );
    }

    #[test]
    fn test_detect_external_tr() {
        let rule = PreferBuiltinTextTransforms::new();

        let bad_code = "^tr 'a-z' 'A-Z' file.txt";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external tr command"
        );
    }
}
