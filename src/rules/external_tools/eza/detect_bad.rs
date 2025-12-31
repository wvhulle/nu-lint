use super::RULE;
use crate::log::init_env_log;

#[test]
fn detects_eza_simple() {
    init_env_log();
    RULE.assert_detects(r"^eza");
}

#[test]
fn detects_eza_with_all() {
    init_env_log();
    RULE.assert_detects(r"^eza -a");
}

#[test]
fn detects_eza_with_long_all() {
    init_env_log();
    RULE.assert_detects(r"^eza --all");
}

#[test]
fn detects_eza_with_long_view() {
    init_env_log();
    RULE.assert_detects(r"^eza -l");
}

#[test]
fn detects_eza_combined_flags() {
    init_env_log();
    RULE.assert_detects(r"^eza -la");
}

#[test]
fn detects_eza_with_recurse() {
    init_env_log();
    RULE.assert_detects(r"^eza -R");
}

#[test]
fn detects_eza_with_tree() {
    init_env_log();
    RULE.assert_detects(r"^eza -T");
}

#[test]
fn detects_eza_with_sort() {
    init_env_log();
    RULE.assert_detects(r"^eza --sort=size");
}

#[test]
fn detects_eza_with_reverse() {
    init_env_log();
    RULE.assert_detects(r"^eza -r");
}

#[test]
fn detects_eza_with_only_dirs() {
    init_env_log();
    RULE.assert_detects(r"^eza -D");
}

#[test]
fn detects_eza_with_only_files() {
    init_env_log();
    RULE.assert_detects(r"^eza -f");
}

#[test]
fn detects_eza_with_path() {
    init_env_log();
    RULE.assert_detects(r"^eza src");
}

#[test]
fn detects_eza_with_multiple_paths() {
    init_env_log();
    RULE.assert_detects(r"^eza src tests");
}

#[test]
fn detects_eza_with_icons() {
    init_env_log();
    RULE.assert_detects(r"^eza --icons");
}

#[test]
fn detects_eza_with_git() {
    init_env_log();
    RULE.assert_detects(r"^eza --git");
}

#[test]
fn detects_eza_with_header() {
    init_env_log();
    RULE.assert_detects(r"^eza -h");
}

#[test]
fn detects_eza_with_level() {
    init_env_log();
    RULE.assert_detects(r"^eza -T -L 2");
}

#[test]
fn detects_eza_with_color() {
    init_env_log();
    RULE.assert_detects(r"^eza --color=always");
}

#[test]
fn detects_eza_group_directories_first() {
    init_env_log();
    RULE.assert_detects(r"^eza --group-directories-first");
}
