use super::RULE;

#[test]
fn fix_jq_length() {
    let source = "^jq 'length' data.json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "open $file | from json | length");
}

#[test]
fn fix_jq_keys() {
    let source = "^jq 'keys' object.json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "columns");
    RULE.assert_replacement_contains(source, "from json");
}

#[test]
fn fix_to_json_then_jq_add() {
    let source = "$numbers | to json | ^jq 'add'";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "math sum");
}

#[test]
fn fix_to_json_then_jq_length() {
    let source = "$values | to json | ^jq 'length'";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "length");
}

#[test]
fn fix_to_json_then_jq_sort() {
    let source = "$items | to json | ^jq 'sort'";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "sort");
}

#[test]
fn fix_to_json_then_jq_unique() {
    let source = "$data | to json | ^jq 'unique'";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "uniq");
}

#[test]
fn fix_to_json_then_jq_flatten() {
    let source = "$nested | to json | ^jq 'flatten'";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "flatten");
}

#[test]
fn fix_jq_array_index() {
    let source = "^jq '.[0]' data.json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "get 0");
    RULE.assert_replacement_contains(source, "from json");
}

#[test]
fn fix_jq_negative_index() {
    let source = "^jq '.[-1]' items.json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "last");
}

#[test]
fn fix_jq_field_access() {
    let source = "^jq '.name' user.json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "get name");
    RULE.assert_replacement_contains(source, "from json");
}

#[test]
fn fix_jq_nested_field_access() {
    let source = "^jq '.user.email' data.json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "get user.email");
}

#[test]
fn fix_jq_array_iteration() {
    let source = "^jq '.[]' array.json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "each");
}

#[test]
fn fix_jq_field_array_iteration() {
    let source = "^jq '.users[]' data.json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "get users | each");
}

#[test]
fn fix_jq_map_field() {
    let source = "$users | to json | ^jq 'map(.name)'";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "get name");
}

#[test]
fn fix_jq_group_by() {
    let source = "$records | to json | ^jq 'group_by(.category)'";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "group-by category");
}

#[test]
fn fix_jq_sort_by() {
    let source = "$events | to json | ^jq 'sort_by(.timestamp)'";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "sort-by timestamp");
}

#[test]
fn fix_jq_min() {
    let source = "$numbers | to json | ^jq 'min'";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "math min");
}

#[test]
fn fix_jq_max() {
    let source = "$numbers | to json | ^jq 'max'";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "math max");
}

#[test]
fn fix_jq_type() {
    let source = "^jq 'type' value.json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "describe");
}

#[test]
fn fix_jq_reverse() {
    let source = "$list | to json | ^jq 'reverse'";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "reverse");
}

#[test]
fn fix_preserves_file_argument() {
    let source = "^jq '.name' /path/to/user.json";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "open $file");
    RULE.assert_replacement_contains(source, "get name");
}

#[test]
fn fix_handles_piped_data_without_open() {
    let source = "$data | to json | ^jq '.name'";
    RULE.assert_count(source, 1);
    RULE.assert_replacement_contains(source, "get name");
}
