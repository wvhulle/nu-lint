use super::rule;

#[test]
fn test_detect_external_in_pipeline() {
    let bad_code = r"
^git status | where status == 'modified'
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_in_longer_pipeline() {
    let bad_code = r"
^ls -la | lines | where $it =~ 'test'
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_with_builtin_filter() {
    let bad_code = "^git status | lines | where { $in | str contains 'modified' }";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_chained_external_commands() {
    let bad_code = "^curl -s api.com/data | ^jq '.items[]' | lines";
    rule().assert_detects(bad_code);
}
