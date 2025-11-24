use super::rule;

#[test]
fn fix_jq_length() {
    let source = "^jq 'length' data.json";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "open $file | from json | length");
}

#[test]
fn fix_jq_keys() {
    let source = "^jq 'keys' object.json";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "columns");
    rule().assert_replacement_contains(source, "from json");
}

#[test]
fn fix_to_json_then_jq_add() {
    let source = "$numbers | to json | ^jq 'add'";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "math sum");
}

#[test]
fn fix_to_json_then_jq_length() {
    let source = "$values | to json | ^jq 'length'";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "length");
}

#[test]
fn fix_to_json_then_jq_sort() {
    let source = "$items | to json | ^jq 'sort'";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "sort");
}

#[test]
fn fix_to_json_then_jq_unique() {
    let source = "$data | to json | ^jq 'unique'";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "uniq");
}

#[test]
fn fix_to_json_then_jq_flatten() {
    let source = "$nested | to json | ^jq 'flatten'";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "flatten");
}

#[test]
fn fix_jq_array_index() {
    let source = "^jq '.[0]' data.json";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "get 0");
    rule().assert_replacement_contains(source, "from json");
}

#[test]
fn fix_jq_negative_index() {
    let source = "^jq '.[-1]' items.json";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "last");
}

#[test]
fn fix_jq_field_access() {
    let source = "^jq '.name' user.json";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "get name");
    rule().assert_replacement_contains(source, "from json");
}

#[test]
fn fix_jq_nested_field_access() {
    let source = "^jq '.user.email' data.json";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "get user.email");
}

#[test]
fn fix_jq_array_iteration() {
    let source = "^jq '.[]' array.json";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "each");
}

#[test]
fn fix_jq_field_array_iteration() {
    let source = "^jq '.users[]' data.json";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "get users | each");
}

#[test]
fn fix_jq_map_field() {
    let source = "$users | to json | ^jq 'map(.name)'";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "get name");
}

#[test]
fn fix_jq_group_by() {
    let source = "$records | to json | ^jq 'group_by(.category)'";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "group-by category");
}

#[test]
fn fix_jq_sort_by() {
    let source = "$events | to json | ^jq 'sort_by(.timestamp)'";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "sort-by timestamp");
}

#[test]
fn fix_jq_min() {
    let source = "$numbers | to json | ^jq 'min'";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "math min");
}

#[test]
fn fix_jq_max() {
    let source = "$numbers | to json | ^jq 'max'";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "math max");
}

#[test]
fn fix_jq_type() {
    let source = "^jq 'type' value.json";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "describe");
}

#[test]
fn fix_jq_reverse() {
    let source = "$list | to json | ^jq 'reverse'";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "reverse");
}

#[test]
fn fix_preserves_file_argument() {
    let source = "^jq '.name' /path/to/user.json";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "open $file");
    rule().assert_replacement_contains(source, "get name");
}

#[test]
fn fix_handles_piped_data_without_open() {
    let source = "$data | to json | ^jq '.name'";
    rule().assert_count(source, 1);
    rule().assert_replacement_contains(source, "get name");
}
