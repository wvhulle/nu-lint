use super::rule;

type LineGenerator = fn(usize) -> String;

#[test]
fn test_detect_long_functions_with_various_patterns() {
    let rule = rule();

    let test_cases: Vec<(usize, &str, LineGenerator)> = vec![
        (81, "very_long_function", |i| format!("    let x{i} = {i}")),
        (100, "extremely_long_function", |i| {
            format!("    let x{i} = {i}")
        }),
        (85, "complex_function", |i| {
            format!("    if $value == {i} {{ print {i} }}")
        }),
        (90, "process_data", |i| {
            format!("    for item in $items {{ print {i} }}")
        }),
        (88, "process_pipeline", |i| {
            format!("    $data | where value == {i} | each {{ |x| $x + 1 }}")
        }),
    ];

    for (line_count, func_name, line_fn) in test_cases {
        let long_function = format!(
            r"def {func_name} [] {{
{}
}}",
            (0..line_count - 1)
                .map(line_fn)
                .collect::<Vec<_>>()
                .join("\n")
        );
        rule.assert_detects(&long_function);
    }
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
