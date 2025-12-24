use super::RULE;

#[test]
fn long_function_with_assignments() {
    let rule = RULE;
    let long_function = format!(
        r"def very_long_function [] {{
{}
}}",
        (0..42)
            .map(|i| format!("    let x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_detects(&long_function);
}

#[test]
fn exported_long_function() {
    let rule = RULE;
    let long_function = format!(
        r"export def long_exported_function [] {{
{}
}}",
        (0..42)
            .map(|i| format!("    let x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_detects(&long_function);
}

#[test]
fn long_function_with_match_statements() {
    let rule = RULE;
    let long_function = format!(
        r"def handle_cases [value] {{
    match $value {{
{}
    }}
}}",
        (0..45)
            .map(|i| format!("        {i} => {{ print {i} }}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_detects(&long_function);
}

#[test]
fn multiple_long_functions() {
    let rule = RULE;
    let code = format!(
        r"def first_long_function [] {{
{}
}}

def second_long_function [] {{
{}
}}",
        (0..42)
            .map(|i| format!("    let x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n"),
        (0..45)
            .map(|i| format!("    let y{i} = {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    rule.assert_count(&code, 2);
}
