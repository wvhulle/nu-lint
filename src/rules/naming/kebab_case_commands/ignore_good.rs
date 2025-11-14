use super::rule;

#[test]
fn ignore_simple_kebab_case() {
    let good_code = r#"
def my-command [] {
    print "correctly named"
}
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn ignore_single_word_command() {
    let good_code = r#"
def test [] {
    print "single word is fine"
}
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn ignore_multi_word_kebab_case() {
    let good_code = r#"
def my-long-command-name [] {
    print "multiple hyphens are fine"
}
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn ignore_exported_kebab_case_command() {
    let good_code = r#"
export def my-exported-command [] {
    print "exported commands should also use kebab-case"
}
"#;

    rule().assert_ignores(good_code);
}

#[test]
fn ignore_kebab_case_with_numbers() {
    let good_code = r#"
def command-v2 [] {
    print "numbers are allowed"
}
"#;

    rule().assert_ignores(good_code);
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

    rule().assert_ignores(good_code);
}
