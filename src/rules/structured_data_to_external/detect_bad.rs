use super::RULE;

#[test]
fn detect_table_to_external() {
    RULE.assert_detects("ls | ^cat");
}

#[test]
fn detect_table_to_external_script() {
    RULE.assert_detects("ls | ./process.nu");
}

#[test]
fn detect_record_to_external() {
    RULE.assert_detects("{ name: 'test', value: 42 } | ^echo");
}

#[test]
fn detect_list_to_external() {
    RULE.assert_detects("[1, 2, 3] | ^cat");
}

#[test]
fn detect_integer_to_external() {
    RULE.assert_detects("42 | ^cat");
}

#[test]
fn detect_each_map_to_external() {
    RULE.assert_detects("ls | each { |it| $it.name } | ^cat");
}

#[test]
fn detect_where_filter_to_external() {
    RULE.assert_detects("ls | where size > 100 | ^cat");
}

#[test]
fn detect_select_to_external() {
    RULE.assert_detects("ls | select name size | ^cat");
}
