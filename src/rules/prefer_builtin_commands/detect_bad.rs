#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::Rule,
        rules::prefer_builtin_commands::PreferBuiltinForCommonCommands,
    };

    #[test]
    fn test_detect_external_ls() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = "^ls -la";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external ls command"
            );
        });
    }

    #[test]
    fn test_detect_external_cat() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = "^cat config.toml";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external cat command"
            );
        });
    }

    #[test]
    fn test_detect_external_grep() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = "^grep \"error\" logs.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external grep command"
            );
        });
    }

    #[test]
    fn test_detect_external_grep_with_flags() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = "^grep -i \"warning\" *.log";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external grep with flags"
            );
        });
    }

    #[test]
    fn test_detect_external_head() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = "^head -n 5 file.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external head command"
            );
        });
    }

    #[test]
    fn test_detect_external_tail() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = "^tail -n 10 file.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external tail command"
            );
        });
    }

    #[test]
    fn test_detect_external_find() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = "^find . -name \"*.rs\"";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external find command"
            );
        });
    }

    #[test]
    fn test_detect_external_commands_in_pipelines() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = "^ls -la | ^grep config";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.len() >= 2,
                "Should detect both ls and grep in pipeline"
            );
        });
    }

    #[test]
    fn test_detect_external_commands_in_function() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = r#"
def git_files [] {
    ^find . -name "*.rs" | ^head -10
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.len() >= 2,
                "Should detect find and head in function"
            );
        });
    }

    #[test]
    fn test_detect_external_cat_with_multiple_files() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = "^cat README.md CHANGELOG.md";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external cat with multiple files"
            );
        });
    }

    #[test]
    fn test_detect_external_sort() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = "^sort file.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external sort command"
            );
        });
    }

    #[test]
    fn test_detect_external_uniq() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = "^uniq file.txt";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external uniq command"
            );
        });
    }

    #[test]
    fn test_detect_external_commands_in_completion_function() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = r#"
def "nu-complete git branches" [] {
    ^cat .git/refs/heads/* | ^sort
}
"#;

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                violations.len() >= 2,
                "Should detect cat and sort in completion function"
            );
        });
    }

    #[test]
    fn test_detect_external_head_tail_with_different_syntax() {
        let rule = PreferBuiltinForCommonCommands::new();
        let bad_code = "^head -5 data.csv";

        LintContext::test_with_parsed_source(bad_code, |context| {
            assert!(
                !rule.check(&context).is_empty(),
                "Should detect external head with -5 syntax"
            );
        });
    }
}
