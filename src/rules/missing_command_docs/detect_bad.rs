use super::rule;

#[test]
fn test_command_without_docs_detected() {
    let bad_code = r"
def my-command [] {
    echo 'hello'
}
";
    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_command_with_params_without_docs_detected() {
    let bad_code = r"
def process-data [input: string, --verbose] {
    print $input
}
";
    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_multiple_commands_without_docs_detected() {
    let bad_code = r"
def first-command [] {
    echo 'first'
}

def second-command [] {
    echo 'second'
}
";

    rule().assert_violation_count_exact(bad_code, 2);
}
