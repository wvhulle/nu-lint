use nu_lint::{Config, apply_fixes_iteratively, lint_engine_with_all_rules};

#[test]
fn test_iterative_fixes_with_overlapping_spans() {
    let content = "^evtest /dev/input/event0 err> /dev/null | lines\n";
    let engine = lint_engine_with_all_rules(Config::default());

    let (fixed, count) = apply_fixes_iteratively(content, &engine);

    assert!(count > 0, "Expected at least one fix to be applied");
    assert!(
        !fixed.contains("err> /dev/null"),
        "Fixed content should not contain err> /dev/null"
    );
    assert!(
        fixed.contains("evtest"),
        "Fixed content should still contain command name"
    );
    assert!(
        fixed.contains("lines"),
        "Fixed content should still contain pipeline command"
    );
}

#[test]
fn test_iterative_fixes_multiple_rules_same_line() {
    let content = "^grep pattern file.txt err> /dev/null | lines\n";
    let engine = lint_engine_with_all_rules(Config::default());

    let (fixed, count) = apply_fixes_iteratively(content, &engine);

    assert!(count > 0, "Expected at least one fix to be applied");
    assert!(!fixed.is_empty(), "Fixed content should not be empty");
    assert!(
        fixed.len() < 200,
        "Fixed content should not be unreasonably long (corruption check)"
    );
}

#[test]
fn test_iterative_fixes_converge() {
    let content = "^curl https://example.com err> /dev/null | str trim\n";
    let engine = lint_engine_with_all_rules(Config::default());

    let (fixed, count) = apply_fixes_iteratively(content, &engine);

    assert!(
        count < 50,
        "Should converge within 50 iterations, got {count}"
    );

    let fixable_after = engine
        .lint_str(&fixed)
        .iter()
        .filter(|violation| violation.fix.is_some())
        .count();
    assert_eq!(
        fixable_after, 0,
        "After applying all fixes, there should be no more fixable violations"
    );
}

#[test]
fn test_iterative_fixes_preserve_utf8() {
    let content = "^echo 测试 err> /dev/null | lines\n";
    let engine = lint_engine_with_all_rules(Config::default());

    let (fixed, count) = apply_fixes_iteratively(content, &engine);

    assert!(count > 0, "Expected at least one fix to be applied");
    assert!(
        fixed.contains("测试"),
        "UTF-8 characters should be preserved"
    );
    assert!(
        !fixed.contains("err> /dev/null"),
        "Redirect should be removed"
    );
    assert!(
        !fixed.is_empty() && fixed.chars().all(|c| !c.is_control() || c.is_whitespace()),
        "Result should contain valid characters"
    );
}
