use super::RULE;

#[test]
fn fix_red_escape() {
    let source = r#"print "\e[31mError\e[0m""#;
    RULE.assert_fixed_is(source, r#"print $"(ansi red)Error(ansi reset)""#);
}

#[test]
fn fix_green_escape() {
    let source = r#"print "\e[32mSuccess\e[0m""#;
    RULE.assert_fixed_is(source, r#"print $"(ansi green)Success(ansi reset)""#);
}

#[test]
fn fix_yellow_escape() {
    let source = r#"print "\e[33mWarning\e[0m""#;
    RULE.assert_fixed_is(source, r#"print $"(ansi yellow)Warning(ansi reset)""#);
}

#[test]
fn fix_blue_escape() {
    let source = r#"print "\e[34mInfo\e[0m""#;
    RULE.assert_fixed_is(source, r#"print $"(ansi blue)Info(ansi reset)""#);
}

#[test]
fn fix_magenta_escape() {
    let source = r#"print "\e[35mPurple\e[0m""#;
    RULE.assert_fixed_is(source, r#"print $"(ansi magenta)Purple(ansi reset)""#);
}

#[test]
fn fix_cyan_escape() {
    let source = r#"print "\e[36mCyan\e[0m""#;
    RULE.assert_fixed_is(source, r#"print $"(ansi cyan)Cyan(ansi reset)""#);
}

#[test]
fn fix_bold_escape() {
    let source = r#"print "\e[1mBold\e[0m""#;
    RULE.assert_fixed_is(source, r#"print $"(ansi bold)Bold(ansi reset)""#);
}

#[test]
fn fix_underline_escape() {
    let source = r#"print "\e[4mUnderline\e[0m""#;
    RULE.assert_fixed_is(source, r#"print $"(ansi underline)Underline(ansi reset)""#);
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
