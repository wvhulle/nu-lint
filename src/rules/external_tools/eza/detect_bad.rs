use super::rule;
use crate::log::instrument;

#[test]
fn detects_eza_simple() {
    instrument();
    rule().assert_detects(r"^eza");
}

#[test]
fn detects_eza_with_all() {
    instrument();
    rule().assert_detects(r"^eza -a");
}

#[test]
fn detects_eza_with_long_all() {
    instrument();
    rule().assert_detects(r"^eza --all");
}

#[test]
fn detects_eza_with_long_view() {
    instrument();
    rule().assert_detects(r"^eza -l");
}

#[test]
fn detects_eza_combined_flags() {
    instrument();
    rule().assert_detects(r"^eza -la");
}

#[test]
fn detects_eza_with_recurse() {
    instrument();
    rule().assert_detects(r"^eza -R");
}

#[test]
fn detects_eza_with_tree() {
    instrument();
    rule().assert_detects(r"^eza -T");
}

#[test]
fn detects_eza_with_sort() {
    instrument();
    rule().assert_detects(r"^eza --sort=size");
}

#[test]
fn detects_eza_with_reverse() {
    instrument();
    rule().assert_detects(r"^eza -r");
}

#[test]
fn detects_eza_with_only_dirs() {
    instrument();
    rule().assert_detects(r"^eza -D");
}

#[test]
fn detects_eza_with_only_files() {
    instrument();
    rule().assert_detects(r"^eza -f");
}

#[test]
fn detects_eza_with_path() {
    instrument();
    rule().assert_detects(r"^eza src");
}

#[test]
fn detects_eza_with_multiple_paths() {
    instrument();
    rule().assert_detects(r"^eza src tests");
}

#[test]
fn detects_eza_with_icons() {
    instrument();
    rule().assert_detects(r"^eza --icons");
}

#[test]
fn detects_eza_with_git() {
    instrument();
    rule().assert_detects(r"^eza --git");
}

#[test]
fn detects_eza_with_header() {
    instrument();
    rule().assert_detects(r"^eza -h");
}

#[test]
fn detects_eza_with_level() {
    instrument();
    rule().assert_detects(r"^eza -T -L 2");
}

#[test]
fn detects_eza_with_color() {
    instrument();
    rule().assert_detects(r"^eza --color=always");
}

#[test]
fn detects_eza_group_directories_first() {
    instrument();
    rule().assert_detects(r"^eza --group-directories-first");
}
