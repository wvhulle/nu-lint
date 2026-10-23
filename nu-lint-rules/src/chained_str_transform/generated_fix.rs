use super::RULE;

// Auto-fix only applies when:
// 1. All calls use -a flag (same semantics)
// 2. All calls have the same replacement value
// 3. All patterns are string literals (not variables)
// 4. No calls use the -r (regex) flag
// 5. Patterns don't overlap (no pattern is substring of another)

#[test]
fn fix_with_all_flag() {
    let bad = r#"$text | str replace -a "x" "_" | str replace -a "y" "_""#;
    RULE.assert_fixed_contains(bad, r#"str replace -ar "x|y" "_""#);
}

#[test]
fn fix_three_replaces_with_all_flag() {
    let bad = r#"$text | str replace -a "a" "X" | str replace -a "b" "X" | str replace -a "c" "X""#;
    RULE.assert_fixed_contains(bad, r#"str replace -ar "a|b|c" "X""#);
}

#[test]
fn fix_escapes_regex_metacharacters() {
    let bad = r#"$text | str replace -a "." "X" | str replace -a "*" "X""#;
    // Dots and asterisks are escaped to \. and \*
    RULE.assert_fixed_contains(bad, r#"str replace -ar "\.|"#);
}

#[test]
fn fix_in_function() {
    let bad = r#"
def clean [] {
    $in | str replace -a "old" "new" | str replace -a "foo" "new"
}
"#;
    RULE.assert_fixed_contains(bad, r#"str replace -ar "old|foo" "new""#);
}

// Cases where auto-fix does NOT apply (but detection still works)

#[test]
fn no_fix_without_all_flag() {
    // Without -a flag, chained replaces have different semantics:
    // - Chained: replaces first occurrence of EACH pattern
    // - Regex: replaces first match overall
    let code = r#"$text | str replace "a" "X" | str replace "b" "X""#;
    RULE.assert_detects(code);
}

#[test]
fn no_fix_different_replacements() {
    // Different replacement values - cannot combine
    let code = r#"$text | str replace -a "a" "X" | str replace -a "b" "Y""#;
    RULE.assert_detects(code);
}

#[test]
fn no_fix_mixed_flags() {
    // Mixed -a and non -a cannot be safely combined
    let code = r#"$text | str replace -a "a" "X" | str replace "b" "X""#;
    RULE.assert_detects(code);
}

#[test]
fn no_fix_with_regex_flag() {
    // Already using regex - user should combine manually
    let code = r#"$text | str replace -ar "a+" "X" | str replace -ar "b+" "X""#;
    RULE.assert_detects(code);
}

#[test]
fn no_fix_overlapping_patterns() {
    // "ab" contains "a" - order matters in regex alternation
    // Original: str replace -a "ab" "X" | str replace -a "a" "X"
    // Would need -ar "ab|a" but regex tries left-to-right, may differ
    let code = r#"$text | str replace -a "ab" "X" | str replace -a "a" "X""#;
    RULE.assert_detects(code);
}

#[test]
fn no_fix_overlapping_patterns_reverse() {
    // "a" is contained in "ab"
    let code = r#"$text | str replace -a "a" "X" | str replace -a "ab" "X""#;
    RULE.assert_detects(code);
}
