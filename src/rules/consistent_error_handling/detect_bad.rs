use super::rule;

#[test]
fn test_missing_exit_code_check() {
    let bad_code = r"
let result = (^bluetoothctl info $mac | complete)
let output = $result.stdout
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_risky_external_function() {
    let bad_code = r#"
def risky-external [] {
    let result = (^bluetoothctl info "AA:BB" | complete)
    print $result.stdout
}
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_another_risky_function() {
    let bad_code = r"
def another-risky [] {
    let output = (^git status | complete)
    print $output.stdout
}
";

    rule().assert_detects(bad_code);
}
