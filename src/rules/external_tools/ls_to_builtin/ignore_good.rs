use super::RULE;

#[test]
fn ignore_builtin_ls() {
    for code in ["ls", "ls -a /tmp", "ls | where size > 1kb"] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_untranslatable_flags() {
    for code in ["^ls -R", "^ls -1", "^ls --color=always", "^ls -i", "^ls -X"] {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_globs_expanded_before_listing() {
    RULE.assert_ignores("^ls -d */");
    RULE.assert_ignores("^ls *.rs");
}

#[test]
fn ignore_output_piped_to_external() {
    RULE.assert_ignores("^ls -t | ^head -n 1");
}

#[test]
fn ignore_other_listing_tools() {
    RULE.assert_ignores("^eza --tree");
}
