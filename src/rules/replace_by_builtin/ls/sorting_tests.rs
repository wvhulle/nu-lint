use crate::{context::LintContext, rules::replace_by_builtin::ls::rule};

#[test]
fn converts_sort_by_time() {
    let source = "^ls -t";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls | sort-by modified"
        );
        assert!(
            fix.description.contains("sort-by modified"),
            "Fix should explain sort-by modified: {}",
            fix.description
        );
    });
}

#[test]
fn converts_sort_by_size() {
    let source = "^ls -S";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls | sort-by size");
        assert!(
            fix.description.contains("sort-by size"),
            "Fix should explain sort-by size: {}",
            fix.description
        );
    });
}

#[test]
fn converts_reverse_sort() {
    let source = "^ls -r";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacements[0].new_text.as_ref(), "ls | reverse");
        assert!(
            fix.description.contains("reverse"),
            "Fix should explain reverse: {}",
            fix.description
        );
    });
}

#[test]
fn combines_sort_and_reverse() {
    let source = "^ls -tr";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls | sort-by modified | reverse"
        );
        assert!(
            fix.description.contains("sort-by modified") && fix.description.contains("reverse"),
            "Fix should explain both transformations: {}",
            fix.description
        );
    });
}

#[test]
fn combines_all_sort_with_reverse() {
    let source = "^ls -Str";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls | sort-by modified | sort-by size | reverse"
        );
    });
}

#[test]
fn combines_flags_with_path() {
    let source = "^ls -lat /var/log";

    LintContext::test_with_parsed_source(source, |context| {
        let violations = rule().check(&context);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacements[0].new_text.as_ref(),
            "ls /var/log --all | sort-by modified"
        );
        assert!(
            fix.description.contains("-l") && fix.description.contains("not needed"),
            "Fix should mention -l is not needed: {}",
            fix.description
        );
    });
}
