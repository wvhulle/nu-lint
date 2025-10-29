use crate::{context::LintContext, rules::prefer_builtin::sed::rule};

#[test]
fn replaces_simple_sed_substitution() {
    let source = r"^sed 's/foo/bar/'";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().expect("Fix should be generated");
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "str replace 'foo' 'bar'"
        );
        assert!(
            fix.description.contains("str replace"),
            "Fix should mention str replace: {}",
            fix.description
        );
    });
}

#[test]
fn handles_global_flag() {
    let source = r"^sed 's/old/new/g'";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "str replace --all 'old' 'new'"
        );
        assert!(
            fix.description.contains("--all") && fix.description.contains("/g"),
            "Fix should explain --all flag replaces sed's /g: {}",
            fix.description
        );
    });
}

#[test]
fn handles_file_input() {
    let source = r"^sed 's/pattern/replacement/' file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "open file.txt | str replace 'pattern' 'replacement'"
        );
        assert!(
            fix.description.contains("open"),
            "Fix should mention open for file input: {}",
            fix.description
        );
    });
}

#[test]
fn handles_in_place_editing() {
    let source = r"^sed -i 's/old/new/' file.txt";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "open file.txt | str replace 'old' 'new' | save -f file.txt"
        );
        assert!(
            fix.description.contains("in-place") && fix.description.contains("save"),
            "Fix should explain in-place editing with save: {}",
            fix.description
        );
    });
}

#[test]
fn handles_in_place_with_global() {
    let source = r"^sed -i 's/foo/bar/g' config.ini";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "open config.ini | str replace --all 'foo' 'bar' | save -f config.ini"
        );
        assert!(
            fix.description.contains("--all"),
            "Fix should mention --all flag: {}",
            fix.description
        );
    });
}

#[test]
fn handles_delete_operation() {
    let source = r"^sed '/pattern/d'";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "lines | where $it !~ 'pattern'"
        );
        assert!(
            fix.description.contains("where") || fix.description.contains("filter"),
            "Fix should suggest where for delete operations: {}",
            fix.description
        );
    });
}

#[test]
fn handles_combined_flags() {
    let source = r"^sed -ie 's/test/prod/g' app.conf";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert!(
            fix.replacements[0].new_text.contains("open")
                && fix.replacements[0].new_text.contains("save"),
            "Fix should handle combined -i flag: {}",
            fix.replacements[0].new_text
        );
    });
}

#[test]
fn provides_default_suggestion_for_complex_sed() {
    let source = r"^sed";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert!(
            fix.description.contains("str replace"),
            "Fix should provide str replace guidance: {}",
            fix.description
        );
    });
}

#[test]
fn ignores_builtin_str_replace() {
    let source = "str replace 'old' 'new'";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);
        assert_eq!(violations.len(), 0, "Should not flag builtin str replace");
    });
}

#[test]
fn detects_gsed() {
    let source = r"^gsed 's/foo/bar/'";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert!(
            fix.replacements[0].new_text.contains("str replace"),
            "Should detect gsed (GNU sed): {}",
            fix.replacements[0].new_text
        );
    });
}
