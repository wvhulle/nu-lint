use crate::rules::bashisms::ls::rule;

#[test]
fn converts_external_ls_to_builtin() {
    let source = "^ls";
    rule().assert_replacement_contains(source, "ls");
    rule().assert_fix_explanation_contains(source, "structured");
    rule().assert_fix_explanation_contains(source, "data");
}

#[test]
fn converts_ls_with_directory_path() {
    let source = "^ls /tmp";
    rule().assert_replacement_contains(source, "ls /tmp");
    rule().assert_fix_explanation_contains(source, "structured");
}

#[test]
fn converts_ls_with_multiple_paths() {
    let source = "^ls src tests";
    rule().assert_replacement_contains(source, "ls src tests");
}

#[test]
fn converts_ls_with_glob_pattern() {
    let source = "^ls *.rs";
    rule().assert_replacement_contains(source, "ls *.rs");
    rule().assert_fix_explanation_contains(source, "structured");
}

#[test]
fn detects_exa_command() {
    let source = "^exa";
    rule().assert_replacement_contains(source, "ls");
    rule().assert_fix_explanation_contains(source, "structured");
}

#[test]
fn detects_eza_command() {
    let source = "^eza -la";
    rule().assert_replacement_contains(source, "ls --all");
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
