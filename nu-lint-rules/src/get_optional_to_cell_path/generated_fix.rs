use super::RULE;

#[test]
fn fix_marks_every_member_optional() {
    RULE.assert_fixed_is(
        "let sample = {a: {b: 1}}\n$sample | get -o a.b",
        "let sample = {a: {b: 1}}\n$sample.a?.b?",
    );
}

#[test]
fn fix_keeps_following_commands() {
    RULE.assert_fixed_is(
        "let sample = {a: 1}\n$sample | get --optional a | default 0",
        "let sample = {a: 1}\n$sample.a? | default 0",
    );
}

#[test]
fn fix_quotes_member_with_space() {
    RULE.assert_fixed_is(
        "let sample = {\"a b\": 1}\n$sample | get -o \"a b\"",
        "let sample = {\"a b\": 1}\n$sample.\"a b\"?",
    );
}
