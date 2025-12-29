use super::RULE;

#[test]
fn detect_red_text() {
    RULE.assert_detects(r#"print "\e[31mRed text\e[0m""#);
}

#[test]
fn detect_green_text() {
    RULE.assert_detects(r#"print "\e[32mGreen text\e[0m""#);
}

#[test]
fn detect_blue_text() {
    RULE.assert_detects(r#"print "\e[34mBlue text\e[0m""#);
}

#[test]
fn detect_yellow_warning() {
    RULE.assert_detects(r#"echo "\e[33mWarning\e[0m""#);
}

#[test]
fn detect_bold_style() {
    RULE.assert_detects(r#"print "\e[1mBold text\e[0m""#);
}

#[test]
fn detect_underline_style() {
    RULE.assert_detects(r#"print "\e[4mUnderlined\e[0m""#);
}

#[test]
fn detect_magenta_color() {
    RULE.assert_detects(r#"print "\e[35mMagenta\e[0m""#);
}

#[test]
fn detect_cyan_color() {
    RULE.assert_detects(r#"print "\e[36mCyan\e[0m""#);
}

#[test]
fn detect_white_color() {
    RULE.assert_detects(r#"print "\e[37mWhite\e[0m""#);
}

#[test]
fn detect_in_interpolated_string() {
    RULE.assert_detects(r#"let msg = $"Status: \e[32mOK\e[0m""#);
}

#[test]
fn detect_multiple_sequences() {
    let code = r#"print "\e[31mError:\e[0m \e[1mFailed\e[0m""#;
    // One violation per string (not per escape sequence)
    RULE.assert_count(code, 1);
}

#[test]
fn detect_bright_red() {
    RULE.assert_detects(r#"print "\e[91mBright red\e[0m""#);
}

#[test]
fn detect_italic_style() {
    RULE.assert_detects(r#"print "\e[3mItalic\e[0m""#);
}
