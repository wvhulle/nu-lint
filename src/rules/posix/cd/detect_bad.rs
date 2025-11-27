use crate::rules::posix::cd::rule;

#[test]
fn detects_external_cd() {
    let source = "^cd";
    rule().assert_detects(source);
}

#[test]
fn detects_external_cd_with_path() {
    let source = "^cd /tmp";
    rule().assert_detects(source);
}

#[test]
fn detects_external_cd_with_home() {
    let source = "^cd ~";
    rule().assert_detects(source);
}

#[test]
fn detects_external_cd_with_previous() {
    let source = "^cd -";
    rule().assert_detects(source);
}

#[test]
fn detects_external_cd_with_physical_flag() {
    let source = "^cd -P /tmp";
    rule().assert_detects(source);
}

#[test]
fn detects_external_cd_with_logical_flag() {
    let source = "^cd -L /home";
    rule().assert_detects(source);
}

#[test]
fn detects_external_cd_with_long_physical_flag() {
    let source = "^cd --physical /var";
    rule().assert_detects(source);
}

#[test]
fn detects_external_cd_to_parent() {
    let source = "^cd ..";
    rule().assert_detects(source);
}
