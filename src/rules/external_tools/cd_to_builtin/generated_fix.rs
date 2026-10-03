use super::RULE;

#[test]
fn fix_without_path() {
    RULE.assert_fixed_is("^cd", "cd");
}

#[test]
fn fix_keeps_path() {
    for path in ["/tmp", "~", "-", ".."] {
        RULE.assert_fixed_is(&format!("^cd {path}"), &format!("cd {path}"));
    }
}

#[test]
fn fix_keeps_quoted_path() {
    RULE.assert_fixed_is(r#"^cd "my dir""#, r#"cd "my dir""#);
}

#[test]
fn fix_physical_flag() {
    RULE.assert_fixed_is("^cd -P /var", "cd --physical /var");
    RULE.assert_fixed_is("^cd --physical /var", "cd --physical /var");
}

#[test]
fn fix_drops_default_logical_flag() {
    RULE.assert_fixed_is("^cd -L /home", "cd /home");
}

#[test]
fn no_fix_for_unknown_flags() {
    RULE.assert_detects_without_fix("^cd -e /tmp");
}
