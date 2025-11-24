use crate::rules::bashisms::find::rule;

#[test]
fn ignores_unsupported_maxdepth_flag() {
    let source = r"^find . -maxdepth 2 -name '*.rs'";
    rule().assert_replacement_contains(source, "*.rs");
}

#[test]
fn ignores_unsupported_executable_flag() {
    let source = r"^find . -executable";
    rule().assert_replacement_contains(source, "ls ./**/*");
}

#[test]
fn handles_no_arguments() {
    let source = "^find";
    rule().assert_replacement_contains(source, "ls ./**/*");
}
