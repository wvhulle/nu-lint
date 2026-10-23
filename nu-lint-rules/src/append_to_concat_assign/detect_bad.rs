use super::RULE;

#[test]
fn test_simple_append_with_parentheses() {
    let bad_code = r#"
mut list = []
let x = 1
$list = ($list | append $x)
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_simple_append_without_parentheses() {
    let bad_code = r#"
mut a = [1 2]
$a = $a | append 3
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_env_variable_with_parentheses() {
    let bad_code = r#"
mut binding = {}
$env.config.keybindings = ($env.config.keybindings | append $binding)
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_env_variable_without_parentheses() {
    let bad_code = r#"
mut binding = {}
$env.config.keybindings = $env.config.keybindings | append $binding
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_cell_path_with_parentheses() {
    let bad_code = r#"
mut config = {items: []}
mut new_item = 1
$config.items = ($config.items | append $new_item)
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_cell_path_without_parentheses() {
    let bad_code = r#"
mut config = {items: []}
mut new_item = 1
$config.items = $config.items | append $new_item
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_list_literal_value_with_parentheses() {
    let bad_code = r#"
mut list = []
$list = ($list | append [1, 2])
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_list_literal_value_without_parentheses() {
    let bad_code = r#"
mut list = []
$list = $list | append [1, 2]
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_variable_value_with_parentheses() {
    let bad_code = r#"
mut list = []
mut items = [1 2]
$list = ($list | append $items)
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_variable_value_without_parentheses() {
    let bad_code = r#"
mut list = []
mut items = [1 2]
$list = $list | append $items
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_subexpression_value_with_parentheses() {
    let bad_code = r#"
mut list = []
$list = ($list | append (get-items))
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_subexpression_value_without_parentheses() {
    let bad_code = r#"
mut list = []
$list = $list | append (get-items)
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_multiple_separate_assignments() {
    let bad_code = r#"
mut a = []
mut b = []
let x = 1
let y = 2
$a = ($a | append $x)
$b = ($b | append $y)
"#;
    RULE.assert_count(bad_code, 2);
}
