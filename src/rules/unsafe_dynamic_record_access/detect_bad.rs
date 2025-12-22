use super::rule;

#[test]
fn detect_dynamic_variable_key() {
    rule().assert_detects("$record | get $key");
}

#[test]
fn detect_dynamic_variable_key_simple() {
    rule().assert_detects("$servers | get $name");
}

#[test]
fn detect_in_closure() {
    rule().assert_detects(
        r#"
        $items | each {|item|
            $data | get $item.name
        }
    "#,
    );
}

#[test]
fn detect_in_conditional() {
    rule().assert_detects("if ($record | get $key) == null { }");
}

#[test]
fn detect_full_cell_path_key() {
    rule().assert_detects("$record | get $config.field");
}

#[test]
fn detect_string_interpolation_key() {
    rule().assert_detects(r#"$record | get $"key_($suffix)""#);
}

#[test]
fn detect_subexpression_key() {
    rule().assert_detects("$record | get ($key | str trim)");
}

#[test]
fn detect_nested_get_with_dynamic_key() {
    rule().assert_detects(
        r#"
        def fetch-value [name: string] {
            $env | get $name
        }
    "#,
    );
}
