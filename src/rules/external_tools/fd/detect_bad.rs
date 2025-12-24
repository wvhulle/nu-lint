use super::RULE;

#[test]
fn detects_fd_simple_pattern() {
    RULE.assert_detects(r#"^fd "*.rs""#);
}

#[test]
fn detects_fd_with_directory() {
    RULE.assert_detects("^fd test src");
}

#[test]
fn detects_fd_with_extension() {
    RULE.assert_detects("^fd -e rs");
}

#[test]
fn detects_fd_with_type() {
    RULE.assert_detects("^fd -t f");
}

#[test]
fn detects_fd_with_hidden() {
    RULE.assert_detects("^fd -H");
}

#[test]
fn detects_fd_with_glob() {
    RULE.assert_detects("^fd -g '*.rs'");
}
