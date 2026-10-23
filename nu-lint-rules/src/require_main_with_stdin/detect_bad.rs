use super::RULE;

#[test]
fn detects_in_var_in_top_level() {
    RULE.assert_detects(
        r#"
print $in
"#,
    );
}

#[test]
fn detects_in_var_in_subexpression() {
    RULE.assert_detects(
        r#"
let x = ($in | length)
"#,
    );
}

#[test]
fn detects_in_var_with_operator() {
    RULE.assert_detects(
        r#"
not $in
"#,
    );
}

#[test]
fn detects_in_var_in_pipeline() {
    RULE.assert_detects(
        r#"
$in | lines | length
"#,
    );
}

#[test]
fn detects_in_var_in_command_argument() {
    RULE.assert_detects(
        r#"
print ($in | to text)
"#,
    );
}
