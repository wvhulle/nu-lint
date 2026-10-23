use super::RULE;

#[test]
fn fix_name_search() {
    RULE.assert_fixed_is("^find . -name '*.rs'", "glob './**/*.rs'");
}

#[test]
fn fix_exact_name() {
    RULE.assert_fixed_is("^find -name Cargo.toml", "glob './**/Cargo.toml'");
}

#[test]
fn fix_files_only() {
    RULE.assert_fixed_is("^find src/ -type f", "glob 'src/**' --no-dir --no-symlink");
}

#[test]
fn fix_directories_by_name() {
    RULE.assert_fixed_is(
        "^find . -type d -name target",
        "glob './**/target' --no-file --no-symlink",
    );
}

#[test]
fn fix_directory_with_spaces() {
    RULE.assert_fixed_is(r#"^find "my dir" -name "*.txt""#, "glob 'my dir/**/*.txt'");
}

#[test]
fn fix_root_directory() {
    RULE.assert_fixed_is("^find / -name hosts", "glob '/**/hosts'");
}
