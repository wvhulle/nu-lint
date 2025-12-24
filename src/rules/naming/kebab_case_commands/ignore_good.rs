use super::RULE;

#[test]
fn ignore_simple_kebab_case() {
    let good_code = r#"
def my-command [] {
    print "correctly named"
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_single_word_command() {
    let good_code = r#"
def test [] {
    print "single word is fine"
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_multi_word_kebab_case() {
    let good_code = r#"
def my-long-command-name [] {
    print "multiple hyphens are fine"
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_exported_kebab_case_command() {
    let good_code = r#"
export def my-exported-command [] {
    print "exported commands should also use kebab-case"
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_kebab_case_with_numbers() {
    let good_code = r#"
def command-v2 [] {
    print "numbers are allowed"
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_multiple_kebab_case_commands() {
    let good_code = r#"
def first-command [] {
    print "first"
}

def second-command [] {
    print "second"
}

def third [] {
    print "third"
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_subcommand_with_space() {
    let good_code = r#"
def "tests calculate-brightness" [] {
    print "subcommands use space separator"
}
"#;

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_nested_subcommand() {
    let good_code = r#"
def "my-module my-subcommand do-action" [] {
    print "deeply nested subcommands"
}
"#;

    RULE.assert_ignores(good_code);
}
