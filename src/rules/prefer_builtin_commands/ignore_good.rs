#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::RegexRule,
        rules::prefer_builtin_commands::PreferBuiltinForCommonCommands,
    };

    #[test]
    fn test_ignore_builtin_ls() {
        let rule = PreferBuiltinForCommonCommands::new();

        let good_code = "ls -la";
        LintContext::test_with_parsed_source(good_code, |context| {
            assert!(
                rule.check(&context).is_empty(),
                "Should ignore builtin ls command"
            );
        });
    }

    #[test]
    fn test_ignore_open_command() {
        let rule = PreferBuiltinForCommonCommands::new();

        let good_code = "open --raw config.toml";
        LintContext::test_with_parsed_source(good_code, |context| {
            assert!(
                rule.check(&context).is_empty(),
                "Should ignore proper open command usage"
            );
        });
    }

    #[test]
    fn test_ignore_where_filter() {
        let rule = PreferBuiltinForCommonCommands::new();

        let good_code = r#"
$data | where name =~ "error"
"#;
        LintContext::test_with_parsed_source(good_code, |context| {
            assert!(
                rule.check(&context).is_empty(),
                "Should ignore where filter instead of grep"
            );
        });
    }

    #[test]
    fn test_ignore_first_last_commands() {
        let rule = PreferBuiltinForCommonCommands::new();

        let good_code = r"
$lines | first 5
$lines | last 10
";
        LintContext::test_with_parsed_source(good_code, |context| {
            assert!(
                rule.check(&context).is_empty(),
                "Should ignore proper first/last usage"
            );
        });
    }

    #[test]
    fn test_ignore_builtin_sort_uniq() {
        let rule = PreferBuiltinForCommonCommands::new();

        let good_code = r"
$data | sort-by name | uniq-by id
";
        LintContext::test_with_parsed_source(good_code, |context| {
            assert!(
                rule.check(&context).is_empty(),
                "Should ignore builtin sort and uniq commands"
            );
        });
    }

    #[test]
    fn test_ignore_external_commands_not_in_list() {
        let rule = PreferBuiltinForCommonCommands::new();

        let good_code = "^git status";
        LintContext::test_with_parsed_source(good_code, |context| {
            assert!(
                rule.check(&context).is_empty(),
                "Should ignore external commands not in replacement list"
            );
        });
    }

    #[test]
    fn test_ignore_specialized_external_tools() {
        let rule = PreferBuiltinForCommonCommands::new();

        let good_code = r"
^docker ps -a
^ffmpeg -i input.mp4 output.avi
^curl -X POST https://api.example.com/data
";
        LintContext::test_with_parsed_source(good_code, |context| {
            assert!(
                rule.check(&context).is_empty(),
                "Should ignore specialized external tools"
            );
        });
    }

    #[test]
    fn test_ignore_proper_pipeline_usage() {
        let rule = PreferBuiltinForCommonCommands::new();

        let good_code = r"
ls *.nu | where size > 1KB | sort-by modified | first 10
";
        LintContext::test_with_parsed_source(good_code, |context| {
            assert!(
                rule.check(&context).is_empty(),
                "Should ignore proper nu pipeline usage"
            );
        });
    }
}
