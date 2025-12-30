use super::RULE;

#[test]
fn fix_red_escape() {
    let source = r#"print "\e[31mError\e[0m""#;
    RULE.assert_fixed_contains(source, "ansi red");
}

#[test]
fn fix_green_escape() {
    let source = r#"print "\e[32mSuccess\e[0m""#;
    RULE.assert_fixed_contains(source, "ansi green");
}

#[test]
fn fix_reset_escape() {
    let source = r#"print "Text\e[0m""#;
    RULE.assert_fixed_contains(source, "ansi reset");
}

#[test]
fn fix_bold_escape() {
    let source = r#"print "\e[1mBold\e[0m""#;
    RULE.assert_fixed_contains(source, "ansi bold");
}

#[test]
fn fix_uses_interpolation() {
    let source = r#"print "\e[33mWarning\e[0m""#;
    RULE.assert_fixed_contains(source, "$(ansi yellow)");
}

#[test]
fn fix_explanation_mentions_ansi() {
    let source = r#"print "\e[31mRed\e[0m""#;
    RULE.assert_fix_explanation_contains(source, "ansi");
}

#[test]
fn fix_explanation_mentions_color() {
    let source = r#"print "\e[34mBlue\e[0m""#;
    RULE.assert_fix_explanation_contains(source, "blue");
}

#[test]
fn fix_cyan_color() {
    let source = r#"print "\e[36mCyan\e[0m""#;
    RULE.assert_fixed_contains(source, "ansi cyan");
}

#[test]
fn fix_underline_style() {
    let source = r#"print "\e[4mUnderline\e[0m""#;
    RULE.assert_fixed_contains(source, "ansi underline");
}

#[test]
fn fix_magenta_color() {
    let source = r#"print "\e[35mPurple\e[0m""#;
    RULE.assert_fixed_contains(source, "ansi magenta");
}
