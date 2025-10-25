use super::rule;

#[test]
fn test_detect_if_chain_in_function() {
    let bad_code = r#"
def get-color [scope: string] {
    if $scope == "wan" {
        "red"
    } else if $scope == "lan" {
        "yellow"
    } else if $scope == "local" {
        "blue"
    } else {
        "green"
    }
}
"#;

    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_inline_if_chain() {
    let bad_code = r#"let priority = if $level == "high" { 1 } else if $level == "medium" { 2 } else if $level == "low" { 3 } else { 0 }"#;

    rule().assert_detects(bad_code);
}
