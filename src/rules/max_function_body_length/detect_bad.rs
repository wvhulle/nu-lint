use super::rule;

#[test]
fn test_detect_function_with_81_lines() {
    let rule = rule();
    let long_function = format!(
        r"def very_long_function [] {{
{}
}}",
        (0..80)
            .map(|i| format!("    let x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_detects(&long_function);
}

#[test]
fn test_detect_function_with_100_lines() {
    let rule = rule();
    let long_function = format!(
        r"def extremely_long_function [] {{
{}
}}",
        (0..99)
            .map(|i| format!("    let x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_detects(&long_function);
}

#[test]
fn test_detect_function_with_many_if_statements() {
    let rule = rule();
    let long_function = format!(
        r"def complex_function [value] {{
{}
}}",
        (0..85)
            .map(|i| format!("    if $value == {i} {{ print {i} }}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_detects(&long_function);
}

#[test]
fn test_detect_exported_long_function() {
    let rule = rule();
    let long_function = format!(
        r"export def long_exported_function [] {{
{}
}}",
        (0..82)
            .map(|i| format!("    let x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_detects(&long_function);
}

#[test]
fn test_detect_function_with_loops() {
    let rule = rule();
    let long_function = format!(
        r"def process_data [items] {{
{}
}}",
        (0..90)
            .map(|i| format!("    for item in $items {{ print {i} }}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_detects(&long_function);
}

#[test]
fn test_detect_function_with_match_statements() {
    let rule = rule();
    let long_function = format!(
        r"def handle_cases [value] {{
    match $value {{
{}
    }}
}}",
        (0..85)
            .map(|i| format!("        {i} => {{ print {i} }}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_detects(&long_function);
}

#[test]
fn test_detect_multiple_long_functions() {
    let rule = rule();
    let code = format!(
        r"def first_long_function [] {{
{}
}}

def second_long_function [] {{
{}
}}",
        (0..81)
            .map(|i| format!("    let x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n"),
        (0..85)
            .map(|i| format!("    let y{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_violation_count_exact(&code, 2);
}

#[test]
fn test_detect_function_with_pipeline_operations() {
    let rule = rule();
    let long_function = format!(
        r"def process_pipeline [data] {{
{}
}}",
        (0..88)
            .map(|i| format!("    $data | where value == {i} | each {{ |x| $x + 1 }}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_detects(&long_function);
}
