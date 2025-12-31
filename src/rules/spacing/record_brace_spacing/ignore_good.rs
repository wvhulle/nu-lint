use super::RULE;

#[test]
fn test_good_record_no_spaces() {
    // Style guide correct example
    let good = "{x: 1, y: 2}";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_record_without_commas() {
    // Style guide correct example (no commas between items)
    let good = "{x: 1 y: 2}";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_record_assigned_to_variable() {
    let good = "let page = {header: $header_data, document: $doc}";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_record_with_variables() {
    let good = "let data = {name: $name, value: $value}";
    RULE.assert_ignores(good);
}

#[test]
fn test_multiline_record_ignored() {
    // Multiline records have different formatting rules
    let good = r"{
    key1: value1
    key2: value2
}";
    RULE.assert_ignores(good);
}

#[test]
fn test_multiline_record_with_spacing_ignored() {
    // Even multiline records with internal spacing are ignored
    let good = r"{ 
    key1: value1
    key2: value2
 }";
    RULE.assert_ignores(good);
}

#[test]
fn test_block_ignored() {
    // Blocks are handled by block_body_spacing rule
    let good = "do { print 'hello' }";
    RULE.assert_ignores(good);
}

#[test]
fn test_closure_with_params_ignored() {
    // Closures are handled by closure_param_spacing rule
    let good = "[1 2] | each {|x| $x * 2 }";
    RULE.assert_ignores(good);
}

#[test]
fn test_empty_record() {
    // Empty records are ignored
    let good = "{}";
    RULE.assert_ignores(good);
}
