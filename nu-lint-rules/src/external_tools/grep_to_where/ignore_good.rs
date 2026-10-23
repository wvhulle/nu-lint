use super::RULE;

#[test]
fn ignore_ripgrep_searching_files() {
    for code in [
        r"^rg -F 'needle' -t py --json",
        r#"^rg "pattern""#,
        r#"^rg "pattern" src/"#,
        r#"^rg -g '*.rs' "fn main""#,
    ] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_grep_searching_files() {
    for code in [
        r#"^grep -r "TODO" ."#,
        r#"^grep -rn "TODO" src/"#,
        r#"^grep -l "pattern" *.txt"#,
        r#"^grep "pattern" file1.txt file2.txt"#,
        r#"^grep "pattern" *.log"#,
    ] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_grep_reading_terminal() {
    RULE.assert_ignores(r#"^grep "pattern""#);
}

#[test]
fn ignore_untranslatable_flags() {
    for code in [
        r#"$text | ^grep -A 3 "pattern""#,
        r#"$text | ^grep -o "[0-9]+""#,
        r#"$text | ^grep -n "TODO""#,
        r#"$text | ^grep -w "word""#,
        r#"$text | ^grep -x "line""#,
        r#"$text | ^grep -m 1 "first""#,
        r#"$text | ^grep --color=always "x""#,
        r#"$text | ^rg -S "x""#,
        r#"$text | ^rg --json "x""#,
    ] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_spread_arguments() {
    RULE.assert_ignores(r"let args = [-r x .]; ^grep ...$args");
}

#[test]
fn ignore_nushell_filters() {
    for code in [
        r#"lines | where $it =~ "pattern""#,
        r#"open --raw file.txt | lines | where ($it | str contains "error")"#,
        r#"ls | where name =~ "test""#,
        r#"find "pattern""#,
    ] {
        RULE.assert_ignores(code);
    }
}
