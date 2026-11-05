use nu_lint_analysis::log::init_test_log;

use super::RULE;

#[test]
fn test_simple_string_chain_fix() {
    init_test_log();
    let bad_code = r#"
if $status == "ok" {
    "success"
} else if $status == "error" {
    "failed"
} else if $status == "pending" {
    "waiting"
} else {
    "unknown"
}
"#;
    RULE.assert_fixed_contains(bad_code, "match $status");
    RULE.assert_fixed_contains(bad_code, r#""ok" =>"#);
    RULE.assert_fixed_contains(bad_code, r#""error" =>"#);
    RULE.assert_fixed_contains(bad_code, r#""pending" =>"#);
    RULE.assert_fixed_contains(bad_code, "_ =>");
    RULE.assert_fixed_contains(bad_code, r#""success""#);
    RULE.assert_fixed_contains(bad_code, r#""failed""#);
}

#[test]
fn test_numeric_chain_fix() {
    let bad_code = r#"
if $code == 200 {
    "ok"
} else if $code == 404 {
    "not found"
} else if $code == 500 {
    "error"
} else {
    "unknown"
}
"#;
    RULE.assert_fixed_contains(bad_code, "match $code");
    RULE.assert_fixed_contains(bad_code, "200 =>");
    RULE.assert_fixed_contains(bad_code, "404 =>");
    RULE.assert_fixed_contains(bad_code, "500 =>");
    RULE.assert_fixed_contains(bad_code, "_ =>");
}

#[test]
fn no_fix_when_last_branch_tests_another_condition() {
    RULE.assert_detects_without_fix(
        r#"
def classify [kind: string, enabled: bool] {
    if $kind == "a" { 1 } else if $kind == "b" { 2 } else if $enabled { 3 } else { 4 }
}
"#,
    );
}

#[test]
fn no_fix_when_branches_test_inequality() {
    RULE.assert_detects_without_fix(
        r#"
def classify [kind: string] {
    if $kind != "a" { 1 } else if $kind != "b" { 2 } else if $kind != "c" { 3 }
}
"#,
    );
}

#[test]
fn no_fix_when_branch_compares_to_variable() {
    RULE.assert_detects_without_fix(
        r#"
def classify [kind: string, other: string] {
    if $kind == "a" { 1 } else if $kind == $other { 2 } else if $kind == "c" { 3 }
}
"#,
    );
}

#[test]
fn fix_matches_on_local_variable_with_dollar() {
    RULE.assert_fixed_is(
        r#"let picked = 1
if $picked == 1 { "a" } else if $picked == 2 { "b" } else if $picked == 3 { "c" }"#,
        r#"let picked = 1
match $picked {
    1 => { "a" },
    2 => { "b" },
    3 => { "c" },
}"#,
    );
}

#[test]
fn fix_matches_on_flag_variable() {
    RULE.assert_fixed_is(
        r#"def pick [--variant: int] { if $variant == 1 { "a" } else if $variant == 2 { "b" } else if $variant == 3 { "c" } }"#,
        r#"def pick [--variant: int] { match $variant {
    1 => { "a" },
    2 => { "b" },
    3 => { "c" },
} }"#,
    );
}

#[test]
fn no_fix_when_branches_compare_different_fields() {
    RULE.assert_detects_without_fix(
        r#"def pick [row: list] {
    if $row.0 == 1 { "a" } else if $row.1 == 2 { "b" } else if $row.0 == 3 { "c" }
}"#,
    );
}
