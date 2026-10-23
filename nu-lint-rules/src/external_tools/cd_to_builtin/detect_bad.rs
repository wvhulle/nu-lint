use super::RULE;

#[test]
fn detects_external_cd() {
    let source = "^cd";
    RULE.assert_detects(source);
}

#[test]
fn detects_external_cd_with_path() {
    let source = "^cd /tmp";
    RULE.assert_detects(source);
}

#[test]
fn detects_external_cd_with_home() {
    let source = "^cd ~";
    RULE.assert_detects(source);
}

#[test]
fn detects_external_cd_with_previous() {
    let source = "^cd -";
    RULE.assert_detects(source);
}

#[test]
fn detects_external_cd_with_physical_flag() {
    let source = "^cd -P /tmp";
    RULE.assert_detects(source);
}

#[test]
fn detects_external_cd_with_logical_flag() {
    let source = "^cd -L /home";
    RULE.assert_detects(source);
}

#[test]
fn detects_external_cd_with_long_physical_flag() {
    let source = "^cd --physical /var";
    RULE.assert_detects(source);
}

#[test]
fn detects_external_cd_to_parent() {
    let source = "^cd ..";
    RULE.assert_detects(source);
}
