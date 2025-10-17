#[cfg(test)]
mod tests {

    use crate::{
        context::LintContext, rule::RegexRule,
        rules::prefer_builtin_system_commands::PreferBuiltinSystemCommands,
    };

    #[test]
    fn test_detect_external_env() {
        let rule = PreferBuiltinSystemCommands::new();
        let bad_code = "^env";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external env command"
            );
        });
    }

    #[test]
    fn test_detect_external_date() {
        let rule = PreferBuiltinSystemCommands::new();
        let bad_code = "^date";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external date command"
            );
        });
    }

    #[test]
    fn test_detect_external_man() {
        let rule = PreferBuiltinSystemCommands::new();
        let bad_code = "^man ls";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external man command"
            );
        });
    }

    #[test]
    fn test_detect_external_read() {
        let rule = PreferBuiltinSystemCommands::new();
        let bad_code = "^read -p \"Enter value: \"";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external read command"
            );
        });
    }

    #[test]
    fn test_detect_external_whoami() {
        let rule = PreferBuiltinSystemCommands::new();
        let bad_code = "^whoami";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external whoami command"
            );
        });
    }

    #[test]
    fn test_detect_external_hostname() {
        let rule = PreferBuiltinSystemCommands::new();
        let bad_code = "^hostname";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external hostname command"
            );
        });
    }

    #[test]
    fn test_detect_external_which() {
        let rule = PreferBuiltinSystemCommands::new();
        let bad_code = "^which ls";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external which command"
            );
        });
    }

    #[test]
    fn test_detect_external_pwd() {
        let rule = PreferBuiltinSystemCommands::new();
        let bad_code = "^pwd";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external pwd command"
            );
        });
    }

    #[test]
    fn test_detect_external_cd() {
        let rule = PreferBuiltinSystemCommands::new();
        let bad_code = "^cd /tmp";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external cd command"
            );
        });
    }
}
