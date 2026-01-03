use super::RULE;

#[test]
fn fix_table_to_jq_with_json() {
    RULE.assert_fixed_is("ls | ^jq '.name'", "ls | to json | ^jq '.name'");
}

#[test]
fn fix_table_to_cat_with_text() {
    RULE.assert_fixed_is("ls | ^cat", "ls | to text | ^cat");
}

#[test]
fn fix_record_to_external_with_json() {
    RULE.assert_fixed_is(
        "{ name: 'test', value: 42 } | ^echo",
        "{ name: 'test', value: 42 } | to json | ^echo",
    );
}

#[test]
fn fix_list_to_grep_with_lines() {
    RULE.assert_fixed_is(
        "[1, 2, 3] | ^grep 2",
        "[1, 2, 3] | str join (char newline) | ^grep 2",
    );
}

#[test]
fn fix_table_to_grep_with_lines() {
    RULE.assert_fixed_is("ls | ^grep test", "ls | to text | ^grep test");
}

#[test]
fn fix_integer_to_external_with_text() {
    RULE.assert_fixed_is("42 | ^cat", "42 | to text | ^cat");
}

#[test]
fn fix_table_to_csvcut_with_csv() {
    RULE.assert_fixed_is("ls | ^csvcut -c name", "ls | to csv | ^csvcut -c name");
}

#[test]
fn fix_record_to_jq_with_json() {
    RULE.assert_fixed_is(
        "{ a: 1, b: 2 } | ^jq '.a'",
        "{ a: 1, b: 2 } | to json | ^jq '.a'",
    );
}

#[test]
fn fix_list_to_cat_with_lines() {
    RULE.assert_fixed_is(
        "[1, 2, 3] | ^cat",
        "[1, 2, 3] | str join (char newline) | ^cat",
    );
}

#[test]
fn fix_table_to_awk_with_lines() {
    RULE.assert_fixed_is("ls | ^awk '{print $1}'", "ls | to text | ^awk '{print $1}'");
}

#[test]
fn fix_table_to_script_with_json() {
    RULE.assert_fixed_is("ls | ./process.nu", "ls | to json | ./process.nu");
}

#[test]
fn fix_each_result_to_external() {
    RULE.assert_fixed_is(
        "ls | each { |it| $it.name } | ^cat",
        "ls | each { |it| $it.name } | str join (char newline) | ^cat",
    );
}

#[test]
fn fix_table_with_err_redirection() {
    RULE.assert_fixed_is("ls err>| ^cat", "ls | to text err>| ^cat");
}

#[test]
fn fix_list_with_out_and_err_redirection() {
    RULE.assert_fixed_is(
        "[1, 2, 3] out+err>| ^grep 2",
        "[1, 2, 3] | str join (char newline) out+err>| ^grep 2",
    );
}
