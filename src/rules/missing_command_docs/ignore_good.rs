use super::rule;

#[test]
fn test_command_with_docs_not_flagged() {
    let good_code = r"
# This is a documented command
def my-command [] {
    echo 'hello'
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_command_with_multiline_docs_not_flagged() {
    let good_code = r"
# Process some data
# Takes input and processes it
def process-data [input: string] {
    print $input
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_command_with_doc_comment_not_flagged() {
    let good_code = r#"
# A simple greeting command
def greet [name: string] {
    print $"Hello ($name)"
}
"#;

    rule().assert_ignores(good_code);
}
