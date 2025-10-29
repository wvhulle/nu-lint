// Tests for generated fixes for prefer_builtin_commands rule
//
// This module validates that fixes explain Nu's better defaults and flag redundancies.

#[cfg(test)]
mod ls_command {
    use crate::{context::LintContext, rules::prefer_builtin_commands::rule};

    #[test]
    fn replaces_simple_ls() {
        let source = "^ls";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            assert_eq!(violations.len(), 1);
            let fix = violations[0].fix.as_ref().expect("Fix should be generated");
            assert_eq!(fix.replacements[0].new_text.as_ref(), "ls");
            assert!(
                fix.description.contains("structured table"),
                "Fix should mention structured data advantage"
            );
        });
    }

    #[test]
    fn preserves_directory_argument() {
        let source = "^ls /tmp";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            let fix = violations[0].fix.as_ref().unwrap();
            assert_eq!(fix.replacements[0].new_text.as_ref(), "ls /tmp");
        });
    }

    #[test]
    fn preserves_flags() {
        let source = "^ls -la";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            let fix = violations[0].fix.as_ref().unwrap();
            assert_eq!(fix.replacements[0].new_text.as_ref(), "ls -la");
        });
    }

    #[test]
    fn preserves_glob_pattern() {
        let source = "^ls *.rs";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            let fix = violations[0].fix.as_ref().unwrap();
            assert_eq!(fix.replacements[0].new_text.as_ref(), "ls *.rs");
        });
    }
}

#[cfg(test)]
mod cat_command {
    use crate::{context::LintContext, rules::prefer_builtin_commands::rule};

    #[test]
    fn replaces_simple_cat_with_open_raw() {
        let source = "^cat file.txt";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            assert_eq!(violations.len(), 1);
            let fix = violations[0].fix.as_ref().expect("Fix should be generated");
            assert_eq!(fix.replacements[0].new_text.as_ref(), "open --raw file.txt");
            assert!(
                fix.description.contains("auto-parse") || fix.description.contains("structured"),
                "Fix should mention auto-parsing advantage: {}",
                fix.description
            );
        });
    }

    #[test]
    fn suggests_open_for_first_file_when_multiple() {
        let source = "^cat file1.txt file2.txt";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            let fix = violations[0].fix.as_ref().unwrap();
            assert_eq!(
                fix.replacements[0].new_text.as_ref(),
                "open --raw file1.txt"
            );
        });
    }

    #[test]
    fn handles_structured_files() {
        let source = "^cat config.json";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            let fix = violations[0].fix.as_ref().unwrap();
            assert_eq!(
                fix.replacements[0].new_text.as_ref(),
                "open --raw config.json"
            );
        });
    }
}

#[cfg(test)]
mod grep_command {
    use crate::{context::LintContext, rules::prefer_builtin_commands::rule};

    #[test]
    fn replaces_simple_grep_with_find() {
        let source = r#"^grep "pattern""#;

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            assert_eq!(violations.len(), 1);
            let fix = violations[0].fix.as_ref().expect("Fix should be generated");
            assert_eq!(fix.replacements[0].new_text.as_ref(), r#"find ""pattern"""#);
            assert!(
                fix.description.contains("case-insensitive"),
                "Fix should mention case-insensitive default: {}",
                fix.description
            );
        });
    }

    #[test]
    fn mentions_redundant_i_flag() {
        let source = r#"^grep -i "warning" logs.txt"#;

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            let fix = violations[0].fix.as_ref().unwrap();
            assert!(
                fix.description.contains("redundant") || fix.description.contains("-i"),
                "Fix should mention that -i flag is redundant in Nu: {}",
                fix.description
            );
        });
    }

    #[test]
    fn suggests_where_for_complex_grep() {
        let source = r#"^grep -r "TODO" ."#;

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            let fix = violations[0].fix.as_ref().unwrap();
            assert_eq!(
                fix.replacements[0].new_text.as_ref(),
                r#"where $it =~ "pattern""#
            );
        });
    }
}

#[cfg(test)]
mod head_tail_commands {
    use crate::{context::LintContext, rules::prefer_builtin_commands::rule};

    #[test]
    fn replaces_head_with_first() {
        let source = "^head -5 file.txt";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            assert_eq!(violations.len(), 1);
            let fix = violations[0].fix.as_ref().expect("Fix should be generated");
            assert_eq!(fix.replacements[0].new_text.as_ref(), "first 5");
            assert!(
                fix.description.contains("cleaner syntax") || fix.description.contains("first"),
                "Fix should mention cleaner syntax: {}",
                fix.description
            );
        });
    }

    #[test]
    fn replaces_tail_with_last() {
        let source = "^tail -10 file.txt";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            let fix = violations[0].fix.as_ref().unwrap();
            assert_eq!(fix.replacements[0].new_text.as_ref(), "last 10");
            assert!(
                fix.description.contains("cleaner syntax") || fix.description.contains("last"),
                "Fix should mention cleaner syntax: {}",
                fix.description
            );
        });
    }

    #[test]
    fn handles_head_without_count() {
        let source = "^head file.txt";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            let fix = violations[0].fix.as_ref().unwrap();
            assert_eq!(fix.replacements[0].new_text.as_ref(), "first 10");
        });
    }
}

#[cfg(test)]
mod find_command {
    use crate::{context::LintContext, rules::prefer_builtin_commands::rule};

    #[test]
    fn replaces_find_with_ls_glob() {
        let source = r#"^find . -name "*.rs""#;

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            assert_eq!(violations.len(), 1);
            let fix = violations[0].fix.as_ref().expect("Fix should be generated");
            assert_eq!(fix.replacements[0].new_text.as_ref(), "ls **/*.rs");
            assert!(
                fix.description.contains("glob") || fix.description.contains("ls"),
                "Fix should mention glob patterns: {}",
                fix.description
            );
        });
    }

    #[test]
    fn replaces_find_directory() {
        let source = "^find src";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            let fix = violations[0].fix.as_ref().unwrap();
            assert_eq!(fix.replacements[0].new_text.as_ref(), "ls src/**/*");
        });
    }
}

#[cfg(test)]
mod sort_uniq_commands {
    use crate::{context::LintContext, rules::prefer_builtin_commands::rule};

    #[test]
    fn replaces_sort() {
        let source = "^sort file.txt";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            assert_eq!(violations.len(), 1);
            let fix = violations[0].fix.as_ref().expect("Fix should be generated");
            assert_eq!(fix.replacements[0].new_text.as_ref(), "sort");
            assert!(
                fix.description.contains("any data type") || fix.description.contains("natural"),
                "Fix should mention data type flexibility: {}",
                fix.description
            );
        });
    }

    #[test]
    fn replaces_uniq() {
        let source = "^uniq file.txt";

        LintContext::test_with_parsed_source(source, |context| {
            let violations = rule().check(&context);

            let fix = violations[0].fix.as_ref().unwrap();
            assert_eq!(fix.replacements[0].new_text.as_ref(), "uniq");
            assert!(
                fix.description.contains("structured") || fix.description.contains("uniq-by"),
                "Fix should mention structured data support: {}",
                fix.description
            );
        });
    }
}
