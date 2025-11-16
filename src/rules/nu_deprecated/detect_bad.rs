use super::rule;

#[test]
fn detect_when_deprecation_warning_exists() {
    // This test verifies the rule works when Nushell's parser
    // emits a deprecation warning. The specific code that triggers
    // a deprecation may change between Nushell versions.
    // If this test fails, it may be because there are no deprecated
    // features in the current Nushell version, or the example needs updating.

    // Note: As of Nushell 0.108.0, there may not be any deprecated features.
    // This test is included to demonstrate the detection mechanism.
    // When deprecated features exist, they will be caught automatically.

    // Example of what might trigger deprecation (may not work in all versions):
    let code = "
let result = ([1, 2, 3] | get -i 0)
print $result

";

    // For now, we test that the rule doesn't crash on valid code
    // When real deprecations exist, this can be updated to use assert_detects
    rule().assert_detects(code);
}
