use super::rule;
use crate::log::instrument;

#[test]
fn test_detect_string_equality_chain_in_function() {
    instrument();
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
    instrument();
    let bad_code = r#"let priority = if $level == "high" { 1 } else if $level == "medium" { 2 } else if $level == "low" { 3 } else { 0 }"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_numeric_equality_chain() {
    let bad_code = r#"
if $code == 200 {
    "ok"
} else if $code == 404 {
    "not found"
} else if $code == 500 {
    "server error"
} else {
    "unknown"
}
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_four_branch_equality_chain() {
    let bad_code = r#"
if $status == "pending" {
    1
} else if $status == "approved" {
    2
} else if $status == "rejected" {
    3
} else if $status == "cancelled" {
    4
} else {
    0
}
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_chain_in_closure() {
    let bad_code = r#"
let mapper = {|item|
    if $item == "a" {
        1
    } else if $item == "b" {
        2
    } else if $item == "c" {
        3
    } else {
        0
    }
}
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_minimal_three_branch_chain() {
    let bad_code = r#"
if $x == 1 { "one" } else if $x == 2 { "two" } else if $x == 3 { "three" }
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_chain_with_not_equal_operator() {
    let bad_code = r#"
if $type != "string" {
    "not string"
} else if $type != "int" {
    "not int"
} else if $type != "bool" {
    "not bool"
} else {
    "unknown"
}
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_chain_without_final_else() {
    let bad_code = r#"
if $mode == "read" {
    open $file
} else if $mode == "write" {
    save $file
} else if $mode == "append" {
    $content | save -a $file
}
"#;

    rule().assert_detects(bad_code);
}
