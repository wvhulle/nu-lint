use crate::{context::LintContext, rules::prefer_builtin::find::rule};

#[test]
fn detects_external_find_with_name_pattern() {
    let source = r#"^find . -name "*.rs""#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "prefer_builtin_find");
        assert!(
            violations[0].message.contains("ls") || violations[0].message.contains("glob"),
            "Message should mention ls or glob: {}",
            violations[0].message
        );
    });
}

#[test]
fn replaces_find_name_with_ls_glob() {
    let source = r#"^find . -name "*.rs""#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().expect("Fix should be generated");
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls ./**/*.rs");
        assert!(
            fix.description.contains("**") && fix.description.contains("subdirectories"),
            "Fix should explain glob pattern recursion: {}",
            fix.description
        );
        assert!(
            fix.description.contains("structured data"),
            "Fix should mention structured data advantage: {}",
            fix.description
        );
    });
}

#[test]
fn replaces_find_directory_traversal() {
    let source = "^find src";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls src/**/*");
        assert!(
            fix.description.contains("recursive file search"),
            "Fix should explain recursive search: {}",
            fix.description
        );
        assert!(
            fix.description.contains("structured data"),
            "Fix should mention structured data: {}",
            fix.description
        );
    });
}

#[test]
fn replaces_find_current_directory() {
    let source = "^find .";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls ./**/*");
        assert!(
            fix.description.contains("ls") || fix.description.contains("glob"),
            "Fix should suggest ls with glob: {}",
            fix.description
        );
    });
}

#[test]
fn converts_complex_find_with_type_and_mtime() {
    let source = r"^find . -type f -mtime +30";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();

        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where type == file | where modified < ((date now) - 30day)"
        );

        assert!(
            fix.description
                .contains("Pipeline filters replace find flags"),
            "Should explain pipeline filters: {}",
            fix.description
        );
        assert!(
            fix.description.contains("type:") && fix.description.contains("time:"),
            "Should mention both filter types: {}",
            fix.description
        );
    });
}

#[test]
fn converts_find_with_name_and_type() {
    let source = r#"^find /var/log -name "*.log" -type f"#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();

        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls /var/log/**/*.log | where type == file"
        );

        assert!(
            fix.description.contains("**") && fix.description.contains("pattern"),
            "Should explain glob pattern: {}",
            fix.description
        );
        assert!(
            fix.description.contains("where type == file"),
            "Should show the type filter example: {}",
            fix.description
        );
    });
}

#[test]
fn converts_find_with_size_filter() {
    let source = r"^find . -type f -size +100k";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();

        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where type == file | where size > 100kb"
        );

        assert!(
            fix.description.contains("size:"),
            "Should explain size filter: {}",
            fix.description
        );
    });
}

#[test]
fn converts_find_with_empty_flag() {
    let source = r"^find . -empty";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();

        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where size == 0b"
        );

        assert!(
            fix.description.contains("empty:"),
            "Should explain empty filter: {}",
            fix.description
        );
    });
}

#[test]
fn ignores_builtin_find_for_data_filtering() {
    let source = r"ls | find toml";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(
            violations.len(),
            0,
            "Nushell's built-in find is for filtering data, not finding files"
        );
    });
}

#[test]
fn ignores_builtin_find_with_regex() {
    let source = r#"[abc bde arc abf] | find --regex "ab""#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(
            violations.len(),
            0,
            "Nushell's find with --regex is for content search"
        );
    });
}

#[test]
fn ignores_builtin_find_on_strings() {
    let source = r"'Cargo.toml' | find cargo";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(
            violations.len(),
            0,
            "Nushell's find on strings is for text search"
        );
    });
}

#[test]
fn detects_fd_simple_pattern() {
    let source = r#"^fd "*.rs""#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(
            violations.len(),
            1,
            "fd is external; ls provides structured data"
        );
        assert!(
            violations[0].message.contains("ls") || violations[0].message.contains("glob"),
            "Should suggest ls/glob as native alternative: {}",
            violations[0].message
        );
    });
}

#[test]
fn detects_fd_with_directory() {
    let source = "^fd test src/";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(
            violations.len(),
            1,
            "Simple fd usage should suggest ls alternative"
        );
    });
}

#[test]
fn distinguishes_bash_find_from_nushell_find() {
    let bash_find_source = r#"^find . -name "*.toml""#;
    let nushell_find_source = r"ls | find toml";

    LintContext::test_with_parsed_source(bash_find_source, |context| {
        let violations = rule().check(&context);
        assert_eq!(
            violations.len(),
            1,
            "Bash find (^find) is for finding files/directories"
        );
    });

    LintContext::test_with_parsed_source(nushell_find_source, |context| {
        let violations = rule().check(&context);
        assert_eq!(
            violations.len(),
            0,
            "Nushell find is for filtering data structures"
        );
    });
}

#[test]
fn explains_nushell_structured_data_advantage() {
    let source = r#"^find . -name "*.rs""#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let message = &violations[0].message;
        let suggestion = violations[0]
            .suggestion
            .as_ref()
            .expect("Should have a suggestion");

        assert!(
            message.contains("built-in")
                || suggestion.contains("portable")
                || suggestion.contains("error handling")
                || suggestion.contains("structured"),
            "Should explain advantages of built-in ls: message='{message}', \
             suggestion='{suggestion}'"
        );
    });
}
