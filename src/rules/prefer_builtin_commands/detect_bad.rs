#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::prefer_builtin_commands::PreferBuiltinForCommonCommands;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_detect_external_ls() {
        let rule = PreferBuiltinForCommonCommands::new();

        let bad_code = "^ls -la";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external ls command"
        );
    }

    #[test]
    fn test_detect_external_cat() {
        let rule = PreferBuiltinForCommonCommands::new();

        let bad_code = "^cat config.toml";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external cat command"
        );
    }

    #[test]
    fn test_detect_external_grep() {
        let rule = PreferBuiltinForCommonCommands::new();

        let bad_code = "^grep \"error\" logs.txt";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external grep command"
        );
    }

    #[test]
    fn test_detect_external_head() {
        let rule = PreferBuiltinForCommonCommands::new();

        let bad_code = "^head -n 5 file.txt";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external head command"
        );
    }

    #[test]
    fn test_detect_external_tail() {
        let rule = PreferBuiltinForCommonCommands::new();

        let bad_code = "^tail -n 10 file.txt";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external tail command"
        );
    }

    #[test]
    fn test_detect_external_find() {
        let rule = PreferBuiltinForCommonCommands::new();

        let bad_code = "^find . -name \"*.rs\"";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external find command"
        );
    }

    #[test]
    fn test_detect_external_sort() {
        let rule = PreferBuiltinForCommonCommands::new();

        let bad_code = "^sort file.txt";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external sort command"
        );
    }

    #[test]
    fn test_detect_external_uniq() {
        let rule = PreferBuiltinForCommonCommands::new();

        let bad_code = "^uniq file.txt";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external uniq command"
        );
    }
}
