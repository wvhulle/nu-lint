use super::RULE;

#[test]
fn fix_tac_with_file() {
    RULE.assert_fixed_is("^tac file.txt", "open --raw file.txt | lines | reverse");
}

#[test]
fn fix_piped_tac() {
    RULE.assert_fixed_is("$text | ^tac", "$text | lines | reverse");
}

#[test]
fn fix_keeps_variable_file() {
    RULE.assert_fixed_is(
        "def reverse-file [path] { ^tac $path }",
        "def reverse-file [path] { open --raw $path | lines | reverse }",
    );
}
