use super::rule;

#[test]
fn test_ignore_function_with_exactly_80_lines() {
    let rule = rule();
    let function = format!(
        r"def acceptable_function [] {{
{}
}}",
        (0..78)
            .map(|i| format!("    let x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_ignores(&function);
}

#[test]
fn test_ignore_short_function() {
    let rule = rule();
    rule.assert_ignores(r"def short_function [] { print 'hello' }");
}

#[test]
fn test_ignore_function_with_few_lines() {
    let rule = rule();
    rule.assert_ignores(
        r"def simple_function [x y] {
    let sum = $x + $y
    let product = $x * $y
    {sum: $sum, product: $product}
}",
    );
}

#[test]
fn test_ignore_empty_function() {
    let rule = rule();
    rule.assert_ignores(r"def empty_function [] {}");
}

#[test]
fn test_ignore_function_with_comments() {
    let rule = rule();
    let function = format!(
        r"def documented_function [] {{
    # This function does something
{}
    # End of function
}}",
        (0..20)
            .map(|i| format!("    let x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_ignores(&function);
}

#[test]
fn test_ignore_function_with_moderate_complexity() {
    let rule = rule();
    rule.assert_ignores(
        r"def process_items [items] {
    let filtered = $items | where value > 10
    let transformed = $filtered | each { |x| $x * 2 }
    let grouped = $transformed | group-by type
    $grouped
}",
    );
}

#[test]
fn test_ignore_main_function_within_limit() {
    let rule = rule();
    let function = format!(
        r"def main [] {{
{}
}}",
        (0..50)
            .map(|i| format!("    print {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_ignores(&function);
}

#[test]
fn test_ignore_exported_function_within_limit() {
    let rule = rule();
    let function = format!(
        r"export def helper_function [] {{
{}
}}",
        (0..40)
            .map(|i| format!("    let x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_ignores(&function);
}

#[test]
fn test_ignore_multiple_short_functions() {
    let rule = rule();
    rule.assert_ignores(
        r"def func1 [] { print 'one' }
def func2 [] { print 'two' }
def func3 [] { print 'three' }",
    );
}

#[test]
fn test_ignore_function_with_nested_blocks() {
    let rule = rule();
    rule.assert_ignores(
        r"def process_data [data] {
    if ($data | is-empty) {
        return null
    } else {
        $data
        | where active == true
        | each { |item|
            if $item.value > 100 {
                $item | update value { |x| $x.value / 2 }
            } else {
                $item
            }
        }
        | sort-by value
    }
}",
    );
}

#[test]
fn test_ignore_closure_definitions() {
    let rule = rule();
    rule.assert_ignores(
        r"def create_processor [] {
    let processor = { |x|
        $x
        | where valid == true
        | each { |item| $item * 2 }
    }
    $processor
}",
    );
}
