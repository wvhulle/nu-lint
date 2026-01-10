use super::RULE;

#[test]
fn fix_simple_index_access() {
    RULE.assert_fixed_is("$list.0", "$list | get -o 0");
}

#[test]
fn fix_nested_cell_path() {
    RULE.assert_fixed_is("$data.items.0", "$data.items | get -o 0");
}

#[test]
fn fix_deeply_nested_path() {
    RULE.assert_fixed_is(
        "$response.data.results.0",
        "$response.data.results | get -o 0",
    );
}

#[test]
fn fix_large_index() {
    RULE.assert_fixed_is("$list.42", "$list | get -o 42");
}

#[test]
fn fix_in_pipeline() {
    RULE.assert_fixed_is(
        "$data | each {|row| $row.values.0 }",
        "$data | each {|row| $row.values | get -o 0 }",
    );
}
