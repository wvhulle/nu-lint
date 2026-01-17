use super::RULE;

#[test]
fn fix_converts_external_cd_to_builtin() {
    let source = "^cd";
    RULE.assert_fixed_contains(source, "cd");
    RULE.assert_fix_explanation_contains(source, "built-in");
}

#[test]
fn fix_converts_cd_with_path() {
    let source = "^cd /tmp";
    RULE.assert_fixed_contains(source, "cd /tmp");
}

#[test]
fn fix_converts_cd_with_home() {
    let source = "^cd ~";
    RULE.assert_fixed_contains(source, "cd ~");
}

#[test]
fn fix_converts_cd_with_previous() {
    let source = "^cd -";
    RULE.assert_fixed_contains(source, "cd -");
    RULE.assert_fix_explanation_contains(source, "previous directory");
}

#[test]
fn fix_converts_physical_flag() {
    let source = "^cd -P /var";
    RULE.assert_fixed_contains(source, "cd --physical /var");
    RULE.assert_fix_explanation_contains(source, "--physical");
}

#[test]
fn fix_converts_long_physical_flag() {
    let source = "^cd --physical /var";
    RULE.assert_fixed_contains(source, "cd --physical /var");
}

#[test]
fn fix_strips_logical_flag() {
    let source = "^cd -L /home";
    RULE.assert_fixed_contains(source, "cd /home");
}

#[test]
fn fix_converts_parent_directory() {
    let source = "^cd ..";
    RULE.assert_fixed_contains(source, "cd ..");
}

#[test]
fn fix_explains_cannot_change_shell_directory() {
    let source = "^cd /tmp";
    RULE.assert_fix_explanation_contains(source, "cannot change the shell's directory");
}

#[test]
fn fix_mentions_triple_dot_feature() {
    let source = "^cd";
    RULE.assert_fix_explanation_contains(source, "...");
}
