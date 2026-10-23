use super::RULE;

#[test]
fn function_at_line_limit() {
    let rule = RULE;
    let function = format!(
        r"def acceptable_function [] {{
{}
}}",
        (0..38)
            .map(|i| format!("    let x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_ignores(&function);
}

#[test]
fn short_function() {
    let rule = RULE;
    rule.assert_ignores(r"def short_function [] { print 'hello' }");
}

#[test]
fn function_with_few_lines() {
    let rule = RULE;
    rule.assert_ignores(
        r"def simple_function [x y] {
    let sum = $x + $y
    let product = $x * $y
    {sum: $sum, product: $product}
}",
    );
}

#[test]
fn empty_function() {
    let rule = RULE;
    rule.assert_ignores(r"def empty_function [] {}");
}

#[test]
fn function_with_comments() {
    let rule = RULE;
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
fn function_with_moderate_complexity() {
    let rule = RULE;
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
fn main_function_within_limit() {
    let rule = RULE;
    let function = format!(
        r"def main [] {{
{}
}}",
        (0..30)
            .map(|i| format!("    print {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_ignores(&function);
}

#[test]
fn exported_function_within_limit() {
    let rule = RULE;
    let function = format!(
        r"export def helper_function [] {{
{}
}}",
        (0..30)
            .map(|i| format!("    let x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_ignores(&function);
}

#[test]
fn multiple_short_functions() {
    let rule = RULE;
    rule.assert_ignores(
        r"def func1 [] { print 'one' }
def func2 [] { print 'two' }
def func3 [] { print 'three' }",
    );
}

#[test]
fn function_with_nested_blocks() {
    let rule = RULE;
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
fn closure_definitions() {
    let rule = RULE;
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
