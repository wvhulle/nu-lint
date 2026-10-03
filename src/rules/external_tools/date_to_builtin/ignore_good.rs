use super::RULE;

#[test]
fn ignore_other_dates_and_settings() {
    for code in [
        "^date -d yesterday",
        "^date --date=@0",
        "^date -r file.txt",
        "^date -s '2020-01-01'",
        "^date -I",
        "^date -R",
    ] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_formats_chrono_renders_differently() {
    for code in ["^date +%N", "^date +%Z", "^date +%c", "^date +%x"] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_dynamic_format() {
    RULE.assert_ignores("let fmt = '+%Y'; ^date $fmt");
}

#[test]
fn ignore_builtin_date() {
    RULE.assert_ignores("date now | format date '%Y-%m-%d'");
}
