use super::RULE;

#[test]
fn ignore_nushell_explore() {
    RULE.assert_ignores("open --raw file.txt | explore");
}

#[test]
fn ignore_explore_structured() {
    RULE.assert_ignores("open data.json | explore");
}

#[test]
fn ignore_watch_command() {
    RULE.assert_ignores("watch log.txt { open --raw log.txt | lines | last 20 }");
}

#[test]
fn ignore_open_command() {
    RULE.assert_ignores("open file.txt");
}

#[test]
fn ignore_open_raw() {
    RULE.assert_ignores("open --raw file.txt");
}

#[test]
fn ignore_table_explore() {
    RULE.assert_ignores("ls | explore");
}
