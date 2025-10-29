use crate::{context::LintContext, rules::prefer_builtin::find::rule};

// Size filter tests

#[test]
fn converts_size_greater_than() {
    let source = r"^find . -size +1M";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where size > 1mb"
        );
    });
}

#[test]
fn converts_size_less_than() {
    let source = r"^find . -size -500k";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where size < 500kb"
        );
    });
}

#[test]
fn converts_size_exact() {
    let source = r"^find . -size 1G";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where size == 1gb"
        );
    });
}

#[test]
fn converts_size_in_bytes() {
    let source = r"^find . -size 1024";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where size == 1024b"
        );
    });
}

// Time filter tests

#[test]
fn converts_mtime_older_than() {
    let source = r"^find . -mtime +7";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where modified < ((date now) - 7day)"
        );
    });
}

#[test]
fn converts_mtime_newer_than() {
    let source = r"^find . -mtime -3";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where modified > ((date now) - 3day)"
        );
    });
}

#[test]
fn converts_mmin_for_minutes() {
    let source = r"^find . -mmin -60";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where modified > ((date now) - 60day)"
        );
    });
}

// Type filter tests

#[test]
fn converts_type_file() {
    let source = r"^find . -type f";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where type == file"
        );
    });
}

#[test]
fn converts_type_directory() {
    let source = r"^find . -type d";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where type == dir"
        );
    });
}

#[test]
fn converts_type_symlink() {
    let source = r"^find . -type l";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where type == symlink"
        );
    });
}

// Combined filter tests

#[test]
fn combines_multiple_filters_in_pipeline() {
    let source = r#"^find . -name "*.rs" -type f -size +100k -mtime -7"#;

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();

        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/*.rs | where type == file | where size > 100kb | where modified > ((date \
             now) - 7day)"
        );

        assert!(fix.description.contains("type:"));
        assert!(fix.description.contains("size:"));
        assert!(fix.description.contains("time:"));
    });
}

#[test]
fn handles_empty_and_type_together() {
    let source = r"^find . -type f -empty";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls ./**/* | where type == file | where size == 0b"
        );
    });
}
