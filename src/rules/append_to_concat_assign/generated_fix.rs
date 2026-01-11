use super::RULE;

#[test]
fn test_fix_simple_with_parentheses() {
    RULE.assert_fixed_is(
        r#"mut list = []
let x = 1
$list = ($list | append $x)"#,
        r#"mut list = []
let x = 1
$list ++= [$x]"#,
    );
}

#[test]
fn test_fix_simple_without_parentheses() {
    RULE.assert_fixed_is(
        r#"mut a = [1 2]
$a = $a | append 3"#,
        r#"mut a = [1 2]
$a ++= [3]"#,
    );
}

#[test]
fn test_fix_with_mut_declaration() {
    RULE.assert_fixed_is(
        r#"mut a = [1 2]
$a = $a | append 3"#,
        r#"mut a = [1 2]
$a ++= [3]"#,
    );
}

#[test]
fn test_fix_env_variable_with_parentheses() {
    RULE.assert_fixed_is(
        r#"mut kb = {}
$env.config.keybindings = ($env.config.keybindings | append $kb)"#,
        r#"mut kb = {}
$env.config.keybindings ++= [$kb]"#,
    );
}

#[test]
fn test_fix_env_variable_without_parentheses() {
    RULE.assert_fixed_is(
        r#"mut kb = {}
$env.config.keybindings = $env.config.keybindings | append $kb"#,
        r#"mut kb = {}
$env.config.keybindings ++= [$kb]"#,
    );
}

#[test]
fn test_fix_list_value_no_double_wrap() {
    RULE.assert_fixed_is(
        r#"mut list = []
$list = ($list | append [1, 2])"#,
        r#"mut list = []
$list ++= [1, 2]"#,
    );
}

#[test]
fn test_fix_list_value_no_double_wrap_no_parens() {
    RULE.assert_fixed_is(
        r#"mut list = []
$list = $list | append [1, 2]"#,
        r#"mut list = []
$list ++= [1, 2]"#,
    );
}

#[test]
fn test_fix_nested_cell_path() {
    RULE.assert_fixed_is(
        r#"mut data = {items: []}
mut new = 1
$data.items = ($data.items | append $new)"#,
        r#"mut data = {items: []}
mut new = 1
$data.items ++= [$new]"#,
    );
}

#[test]
fn test_fix_nested_cell_path_no_parens() {
    RULE.assert_fixed_is(
        r#"mut data = {items: []}
mut new = 1
$data.items = $data.items | append $new"#,
        r#"mut data = {items: []}
mut new = 1
$data.items ++= [$new]"#,
    );
}

#[test]
fn test_fix_subexpression_value() {
    RULE.assert_fixed_is(
        r#"mut list = []
$list = ($list | append (get-items))"#,
        r#"mut list = []
$list ++= [(get-items)]"#,
    );
}
