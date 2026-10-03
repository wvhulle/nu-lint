use super::RULE;

#[test]
fn fix_plain_listing() {
    RULE.assert_fixed_is("^ls", "ls");
    RULE.assert_fixed_is("^ls src tests", "ls src tests");
}

#[test]
fn fix_keeps_quoted_path() {
    RULE.assert_fixed_is(r#"^ls -la "my dir""#, r#"ls --all "my dir""#);
}

#[test]
fn fix_display_flags_are_dropped() {
    RULE.assert_fixed_is("^ls -lh", "ls");
}

#[test]
fn fix_hidden_files() {
    RULE.assert_fixed_is("^ls -A", "ls --all");
    RULE.assert_fixed_is("^ls --all", "ls --all");
}

#[test]
fn fix_newest_first() {
    RULE.assert_fixed_is("^ls -t", "ls | sort-by modified --reverse");
}

#[test]
fn fix_oldest_first() {
    RULE.assert_fixed_is("^ls -ltr src", "ls src | sort-by modified");
}

#[test]
fn fix_largest_first() {
    RULE.assert_fixed_is("^ls -S", "ls | sort-by size --reverse");
}

#[test]
fn fix_reverse_name_order() {
    RULE.assert_fixed_is("^ls -r", "ls | reverse");
}

#[test]
fn fix_directory_itself() {
    RULE.assert_fixed_is("^ls -d src", "ls --directory src");
}
