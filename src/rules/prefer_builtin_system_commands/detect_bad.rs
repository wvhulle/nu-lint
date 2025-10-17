#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::prefer_builtin_system_commands::PreferBuiltinSystemCommands;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_detect_external_env() {
        let rule = PreferBuiltinSystemCommands::new();

        let bad_code = "^env";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external env command"
        );
    }

    #[test]
    fn test_detect_external_date() {
        let rule = PreferBuiltinSystemCommands::new();

        let bad_code = "^date";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external date command"
        );
    }

    #[test]
    fn test_detect_external_man() {
        let rule = PreferBuiltinSystemCommands::new();

        let bad_code = "^man ls";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external man command"
        );
    }

    #[test]
    fn test_detect_external_read() {
        let rule = PreferBuiltinSystemCommands::new();

        let bad_code = "^read -p \"Enter value: \"";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external read command"
        );
    }

    #[test]
    fn test_detect_external_whoami() {
        let rule = PreferBuiltinSystemCommands::new();

        let bad_code = "^whoami";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external whoami command"
        );
    }

    #[test]
    fn test_detect_external_hostname() {
        let rule = PreferBuiltinSystemCommands::new();

        let bad_code = "^hostname";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external hostname command"
        );
    }

    #[test]
    fn test_detect_external_which() {
        let rule = PreferBuiltinSystemCommands::new();

        let bad_code = "^which ls";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external which command"
        );
    }

    #[test]
    fn test_detect_external_pwd() {
        let rule = PreferBuiltinSystemCommands::new();

        let bad_code = "^pwd";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external pwd command"
        );
    }

    #[test]
    fn test_detect_external_cd() {
        let rule = PreferBuiltinSystemCommands::new();

        let bad_code = "^cd /tmp";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect external cd command"
        );
    }
}
