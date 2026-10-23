use super::RULE;

#[test]
fn detect_piped_grep() {
    RULE.assert_detects(r#"^cat log.txt | ^grep "error""#);
}

#[test]
fn detect_piped_ripgrep() {
    RULE.assert_detects(r#"ls | to text | ^rg "toml""#);
}

#[test]
fn detect_grep_on_single_file() {
    RULE.assert_detects(r#"^grep "error" logs.txt"#);
}

#[test]
fn detect_grep_with_translatable_flags() {
    for code in [
        r#"$text | ^grep -i "warning""#,
        r#"$text | ^grep -v "debug""#,
        r#"$text | ^grep -c "error""#,
        r#"$text | ^grep -F "a.b""#,
        r#"$text | ^grep -E "a+b""#,
        r#"$text | ^grep -ivc "warning""#,
        r#"$text | ^grep --ignore-case --invert-match "debug""#,
        r#"$text | ^rg -s "x""#,
    ] {
        RULE.assert_detects(code);
    }
}

#[test]
fn detect_grep_in_closure() {
    RULE.assert_detects(r#"ls | each {|file| open --raw $file.name | ^grep "TODO" }"#);
}

#[test]
fn detect_without_fix_when_basic_regex_diverges() {
    RULE.assert_detects_without_fix(r#"$text | ^grep "a+b""#);
    RULE.assert_detects_without_fix(r#"$text | ^grep 'x\(y\)'"#);
}

#[test]
fn detect_without_fix_when_basic_regex_is_dynamic() {
    RULE.assert_detects_without_fix(r#"let pattern = "x"; $text | ^grep $pattern"#);
}
