use super::RULE;

#[test]
fn ignore_inside_try_block() {
    RULE.assert_ignores("try { $list.0 }");
}

#[test]
fn ignore_inside_if_block() {
    // Assuming the if is a length/empty check
    RULE.assert_ignores("if ($list | is-not-empty) { $list.0 }");
}

#[test]
fn ignore_cell_path_string_access() {
    // String member access is for records
    RULE.assert_ignores("$record.name");
}

#[test]
fn ignore_nested_string_access() {
    RULE.assert_ignores("$record.user.name");
}

#[test]
fn ignore_string_after_validation() {
    RULE.assert_ignores(
        r#"
        if ($data | is-not-empty) {
            $data.status
        }
    "#,
    );
}
