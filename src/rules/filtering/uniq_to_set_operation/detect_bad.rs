use super::RULE;

#[test]
fn detect_append_list_literal_then_uniq() {
    RULE.assert_detects("[3 1 2] | append [1 4] | uniq");
}

#[test]
fn detect_append_list_variable_then_uniq() {
    let bad_code = r#"
let left = [3 1 2]
let right = [1 4]
$left | append $right | uniq
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_where_in_then_uniq() {
    let bad_code = r#"
let allowed = [1 4]
[3 1 2 1 4] | where $it in $allowed | uniq
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_where_not_in_then_uniq() {
    let bad_code = r#"
let excluded = [1 4]
[3 1 2 1 4] | where $it not-in $excluded | uniq
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_inside_typed_function() {
    let bad_code = r#"
def merge [left: list<string>, right: list<string>]: nothing -> list<string> {
    $left | append $right | uniq
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_table_operands() {
    RULE.assert_detects("[[a]; [1]] | append [[a]; [2]] | uniq");
}
