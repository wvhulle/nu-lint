use super::RULE;

#[test]
fn fix_simple_index_access() {
    RULE.assert_fixed_is("$list.0", "$list.0?");
}

#[test]
fn fix_nested_cell_path() {
    RULE.assert_fixed_is("$data.items.0", "$data.items.0?");
}

#[test]
fn fix_deeply_nested_path() {
    RULE.assert_fixed_is("$response.data.results.0", "$response.data.results.0?");
}

#[test]
fn fix_large_index() {
    RULE.assert_fixed_is("$list.42", "$list.42?");
}

#[test]
fn fix_in_pipeline() {
    RULE.assert_fixed_is(
        "$data | each {|row| $row.values.0 }",
        "$data | each {|row| $row.values.0? }",
    );
}

#[test]
fn fix_preserves_path_after_index() {
    // If there's a path after the index, it should be preserved
    RULE.assert_fixed_is("$list.0.name", "$list.0?.name");
}
