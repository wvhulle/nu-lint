use super::RULE;

#[test]
fn ignore_nushell_open() {
    RULE.assert_ignores("open file.txt");
}

#[test]
fn ignore_open_raw() {
    RULE.assert_ignores("open --raw file.txt");
}

#[test]
fn ignore_open_structured() {
    let good_codes = vec![
        "open data.json",
        "open config.toml",
        "open data.csv",
        "open settings.yaml",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_cat() {
    RULE.assert_ignores("^cat file.txt");
}

#[test]
fn ignore_other_commands() {
    RULE.assert_ignores("ls | where size > 1kb");
}
