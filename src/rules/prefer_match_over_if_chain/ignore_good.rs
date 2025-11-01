use super::rule;

#[test]
fn test_good_match_statement() {
    let good = "match $status { 'ok' => 'success', 'error' => 'failed', _ => 'unknown' }";

    rule().assert_ignores(good);
}

#[test]
fn test_good_simple_if() {
    let good = "if $condition { 'yes' } else { 'no' }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_different_variables() {
    let good = "if $x == 1 { 'one' } else if $y == 2 { 'two' } else { 'other' }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_complex_conditions() {
    let good = "if $x > 5 and $y < 10 { 'range' } else { 'outside' }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_match_with_guard() {
    let good = "match $value { x if $x > 10 => 'big', x if $x > 5 => 'medium', _ => 'small' }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_single_if_no_else() {
    let good = "if $condition { print 'yes' }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_two_branch_if_else() {
    // Only 2 branches should not trigger (we require 3+ for a chain)
    let good = r#"
if $x == 1 {
    "one"
} else if $x == 2 {
    "two"
} else {
    "other"
}
"#;
    rule().assert_ignores(good);
}

#[test]
fn test_good_match_with_alternatives() {
    let good = "match $val { 1 | 2 | 3 => 'low', 4 | 5 | 6 => 'mid', _ => 'high' }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_match_with_range() {
    let good = "match $num { 1..10 => 'small', 11..100 => 'medium', _ => 'large' }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_match_with_record_destructuring() {
    let good = "match $rec { {a: $val} => $val, _ => 0 }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_match_with_list_destructuring() {
    let good = "match $list { [$first, ..$rest] => $first, _ => null }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_if_with_boolean_operators() {
    let good = r#"
if $x > 0 and $x < 10 {
    "range"
} else if $x >= 10 or $x <= -10 {
    "extreme"
} else {
    "normal"
}
"#;
    rule().assert_ignores(good);
}

#[test]
fn test_good_if_with_method_calls() {
    let good = r#"
if ($str | str contains "error") {
    "error"
} else if ($str | str contains "warn") {
    "warning"
} else {
    "info"
}
"#;
    rule().assert_ignores(good);
}

#[test]
fn test_good_if_with_not_operator() {
    let good = "if not $flag { 'disabled' } else { 'enabled' }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_match_in_pipeline() {
    let good = r#"
$data | match $in {
    {type: "user"} => "User data",
    {type: "admin"} => "Admin data",
    _ => "Unknown data"
}
"#;
    rule().assert_ignores(good);
}
