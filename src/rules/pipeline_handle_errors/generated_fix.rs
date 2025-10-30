use crate::context::LintContext;

fn init_logger() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
    });
}

#[test]
fn test_suggestion_recommends_do_ignore_when_appropriate() {
    init_logger();
    let bad_code = r"^mkdir -p /tmp/test | ignore";
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    let suggestion = violations[0].suggestion.as_ref().unwrap();
    assert!(suggestion.contains("do -i"), "Should mention do -i");
    assert!(
        suggestion.contains("ignore"),
        "Should explain when errors can be ignored"
    );
}

#[test]
fn test_suggestion_recommends_complete_for_custom_handling() {
    init_logger();
    let bad_code = r"^docker build -t myapp . | lines";
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    let suggestion = violations[0].suggestion.as_ref().unwrap();
    assert!(suggestion.contains("complete"), "Should recommend complete");
    assert!(
        suggestion.contains("custom"),
        "Should indicate complete is for custom error handling"
    );
    assert!(
        suggestion.contains("$result.stderr"),
        "Should show how to access error output"
    );
}

// Test that suggestions include the three main patterns
#[test]
fn test_suggestion_recommends_try_for_simple_cases() {
    init_logger();
    let bad_code = r"^git status | lines";
    let violations =
        LintContext::test_with_parsed_source(bad_code, |context| rule().check(&context));

    let suggestion = violations[0].suggestion.as_ref().unwrap();
    assert!(suggestion.contains("try"), "Should recommend try blocks");
    assert!(
        suggestion.contains("simple"),
        "Should indicate try is for simple cases"
    );
}
