use super::RULE;

#[test]
fn test_detect_equal_true() {
    let bad_code = r#"
if $flag == true { print "yes" }
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_equal_false() {
    let bad_code = r#"
if $flag == false { print "no" }
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_not_equal_true() {
    let bad_code = r#"
if $enabled != true { print "disabled" }
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_not_equal_false() {
    let bad_code = r#"
if $active != false { print "active" }
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_true_on_left() {
    let bad_code = r#"
if true == $flag { print "yes" }
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_false_on_left() {
    let bad_code = r#"
if false == $flag { print "no" }
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_in_parentheses() {
    let bad_code = r#"
if ($is_valid == true) { proceed }
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_in_let_binding() {
    let bad_code = r#"
let result = $check == true
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_in_where_clause() {
    let bad_code = r#"
$items | where { $in.active == true }
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_multiple_violations() {
    let bad_code = r#"
if $a == true and $b == false { do_something }
"#;
    RULE.assert_count(bad_code, 2);
}
