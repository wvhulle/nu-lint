use crate::rules::replace_by_builtin::ls::rule;

#[test]
fn detects_external_ls() {
    let source = "^ls";
    rule().assert_detects(source);
}

#[test]
fn replaces_simple_ls() {
    let source = "^ls";
    rule().assert_fix_contains(source, "ls");
    rule().assert_fix_description_contains(source, "structured");
    rule().assert_fix_description_contains(source, "data");
}

#[test]
fn preserves_directory_argument() {
    let source = "^ls /tmp";
    rule().assert_fix_contains(source, "ls /tmp");
    rule().assert_fix_description_contains(source, "structured");
}

#[test]
fn preserves_multiple_paths() {
    let source = "^ls src tests";
    rule().assert_fix_contains(source, "ls src tests");
}

#[test]
fn preserves_glob_pattern() {
    let source = "^ls *.rs";
    rule().assert_fix_contains(source, "ls *.rs");
    rule().assert_fix_description_contains(source, "structured");
}

#[test]
fn detects_exa_command() {
    let source = "^exa";
    rule().assert_fix_contains(source, "ls");
    rule().assert_fix_description_contains(source, "structured");
}

#[test]
fn detects_eza_command() {
    let source = "^eza -la";
    rule().assert_fix_contains(source, "ls --all");
}

#[test]
fn ignores_builtin_ls() {
    let source = "ls";
    rule().assert_ignores(source);
}

#[test]
fn ignores_builtin_ls_with_args() {
    let source = "ls --all *.rs";
    rule().assert_ignores(source);
}
