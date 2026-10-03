use super::RULE;

#[test]
fn fix_piped_substitution() {
    RULE.assert_fixed_is(
        "$text | ^sed 's/foo/bar/g'",
        "$text | str replace --all 'foo' 'bar'",
    );
}

#[test]
fn fix_custom_delimiter() {
    RULE.assert_fixed_is(
        "$text | ^sed 's|/usr|/opt|g'",
        "$text | str replace --all '/usr' '/opt'",
    );
}

#[test]
fn fix_basic_regex_literals() {
    RULE.assert_fixed_is(
        "$text | ^sed 's/a+b/(c)/g'",
        "$text | str replace --all 'a+b' '(c)'",
    );
}

#[test]
fn fix_file_input() {
    RULE.assert_fixed_is(
        r#"^sed "s/it's/it is/g" notes.txt"#,
        r#"open --raw notes.txt | str replace --all "it's" 'it is'"#,
    );
}
