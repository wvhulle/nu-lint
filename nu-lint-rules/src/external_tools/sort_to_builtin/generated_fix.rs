use super::RULE;

#[test]
fn fix_piped_sort() {
    RULE.assert_fixed_is("$text | ^sort", "$text | lines | sort");
}

#[test]
fn fix_reverse() {
    RULE.assert_fixed_is("$text | ^sort --reverse", "$text | lines | sort --reverse");
}

#[test]
fn fix_unique() {
    RULE.assert_fixed_is("$text | ^sort -u", "$text | lines | sort | uniq");
}

#[test]
fn fix_case_insensitive_unique() {
    RULE.assert_fixed_is(
        "$text | ^sort -fu",
        "$text | lines | sort --ignore-case | uniq --ignore-case",
    );
}

#[test]
fn fix_absorbs_following_uniq() {
    RULE.assert_fixed_is(
        "$text | ^sort -r | ^uniq | first 3",
        "$text | lines | sort --reverse | uniq | first 3",
    );
}

#[test]
fn fix_keeps_counting_uniq() {
    RULE.assert_fixed_is(
        "$text | ^sort | ^uniq -c",
        "$text | lines | sort | ^uniq -c",
    );
}

#[test]
fn fix_file() {
    RULE.assert_fixed_is(
        r#"^sort "names list.txt""#,
        r#"open --raw "names list.txt" | lines | sort"#,
    );
}
