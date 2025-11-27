use super::rule;

#[test]
fn detects_fd_simple_pattern() {
    rule().assert_detects(r#"^fd "*.rs""#);
}

#[test]
fn detects_fd_with_directory() {
    rule().assert_detects("^fd test src");
}

#[test]
fn detects_fd_with_extension() {
    rule().assert_detects("^fd -e rs");
}

#[test]
fn detects_fd_with_type() {
    rule().assert_detects("^fd -t f");
}

#[test]
fn detects_fd_with_hidden() {
    rule().assert_detects("^fd -H");
}

#[test]
fn detects_fd_with_glob() {
    rule().assert_detects("^fd -g '*.rs'");
}
