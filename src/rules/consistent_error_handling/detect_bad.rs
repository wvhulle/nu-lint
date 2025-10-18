use super::rule;
use crate::LintContext;

#[test]
fn test_missing_exit_code_check() {
    let bad_code = r"
let result = (^bluetoothctl info $mac | complete)
let output = $result.stdout
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect missing exit_code check"
        );
    });
}

#[test]
fn test_risky_external_function() {
    let bad_code = r#"
def risky-external [] {
    let result = (^bluetoothctl info "AA:BB" | complete)
    print $result.stdout
}
"#;

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect missing exit_code check in risky-external function"
        );
    });
}

#[test]
fn test_another_risky_function() {
    let bad_code = r"
def another-risky [] {
    let output = (^git status | complete)
    print $output.stdout
}
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect missing exit_code check in another-risky function"
        );
    });
}
