use super::RULE;
use crate::log::instrument;

#[test]
fn detects_eza_simple() {
    instrument();
    RULE.assert_detects(r"^eza");
}

#[test]
fn detects_eza_with_all() {
    instrument();
    RULE.assert_detects(r"^eza -a");
}

#[test]
fn detects_eza_with_long_all() {
    instrument();
    RULE.assert_detects(r"^eza --all");
}

#[test]
fn detects_eza_with_long_view() {
    instrument();
    RULE.assert_detects(r"^eza -l");
}

#[test]
fn detects_eza_combined_flags() {
    instrument();
    RULE.assert_detects(r"^eza -la");
}

#[test]
fn detects_eza_with_recurse() {
    instrument();
    RULE.assert_detects(r"^eza -R");
}

#[test]
fn detects_eza_with_tree() {
    instrument();
    RULE.assert_detects(r"^eza -T");
}

#[test]
fn detects_eza_with_sort() {
    instrument();
    RULE.assert_detects(r"^eza --sort=size");
}

#[test]
fn detects_eza_with_reverse() {
    instrument();
    RULE.assert_detects(r"^eza -r");
}

#[test]
fn detects_eza_with_only_dirs() {
    instrument();
    RULE.assert_detects(r"^eza -D");
}

#[test]
fn detects_eza_with_only_files() {
    instrument();
    RULE.assert_detects(r"^eza -f");
}

#[test]
fn detects_eza_with_path() {
    instrument();
    RULE.assert_detects(r"^eza src");
}

#[test]
fn detects_eza_with_multiple_paths() {
    instrument();
    RULE.assert_detects(r"^eza src tests");
}

#[test]
fn detects_eza_with_icons() {
    instrument();
    RULE.assert_detects(r"^eza --icons");
}

#[test]
fn detects_eza_with_git() {
    instrument();
    RULE.assert_detects(r"^eza --git");
}

#[test]
fn detects_eza_with_header() {
    instrument();
    RULE.assert_detects(r"^eza -h");
}

#[test]
fn detects_eza_with_level() {
    instrument();
    RULE.assert_detects(r"^eza -T -L 2");
}

#[test]
fn detects_eza_with_color() {
    instrument();
    RULE.assert_detects(r"^eza --color=always");
}

#[test]
fn detects_eza_group_directories_first() {
    instrument();
    RULE.assert_detects(r"^eza --group-directories-first");
}
