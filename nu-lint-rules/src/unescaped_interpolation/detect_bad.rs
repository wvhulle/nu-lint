#[test]
fn detects_not_operator_without_escaping() {
    let rule = super::RULE;

    let code = r#"
def bad_example [] {
    let path = "/some/path"
    print $"File ($path) (not x)"
}
"#;

    rule.assert_detects(code);
}

#[test]
fn detects_or_operator_in_conditional_branch() {
    let rule = super::RULE;

    let code = r#"
def write_sysfs [path: string, value: string] {
    if ($path | path exists) {
        do -i { $value | save -f $path }
        print -e $"<7>Set ($path) = ($value)"
    } else {
        print -e $"<7>Skipped ($path) (or)"
    }
}
"#;

    rule.assert_count(code, 1);
}

#[test]
fn detects_and_or_operators_without_escaping() {
    let rule = super::RULE;

    for (keyword, var_val) in [("and", "y"), ("or", "z")] {
        let code = format!(
            r#"
def test [] {{
    let var = "x"
    print $"($var) ({keyword} {var_val})"
}}
"#
        );
        rule.assert_detects(&code);
    }
}

#[test]
fn detects_multiple_unescaped_operators_in_single_string() {
    let rule = super::RULE;

    let code = r#"
def test [] {
    let name = "test"
    print $"Item ($name) (or maybe not) (and another)"
}
"#;

    rule.assert_detects(code);
    rule.assert_count(code, 1); // One string with multiple violations is still one violation
}

#[test]
fn detects_unescaped_operators_in_error_messages() {
    let rule = super::RULE;

    let code = r#"
def bad_error_msg [path: string] {
    error make {
        msg: $"File ($path) (not found or not accessible)"
    }
}
"#;

    rule.assert_detects(code);
}

#[test]
fn detects_and_operator_after_interpolated_values() {
    let rule = super::RULE;

    let code = r#"
def bad_nested [] {
    let a = "x"
    let b = "y"
    print $"Values ($a) and ($b) (and)"
}
"#;

    rule.assert_detects(code);
}

#[test]
fn detects_multiple_unescaped_operators_across_strings() {
    let rule = super::RULE;

    let code = r#"
def test_errors [] {
    let file = "config.txt"
    print $"File ($file) (not x)"
    print $"Access ($file) (and y)"
    print $"Service ($file) (or z)"
}
"#;

    rule.assert_detects(code);
    rule.assert_count(code, 3);
}

#[test]
fn detects_incomplete_comparison_and_arithmetic_operators() {
    let rule = super::RULE;

    let code = r#"
def test_incomplete [] {
    let x = "value"
    print $"Result ($x) (==)"
    print $"Status ($x) (+)"
    print $"Check ($x) (if condition)"
}
"#;

    rule.assert_detects(code);
}

#[test]
fn detects_unescaped_boolean_expressions() {
    let rule = super::RULE;

    let code = r#"
def test_complex [] {
    let x = "test"
    print $"Check ($x) (x > 0 and y < 10)"
    print $"Validate ($x) (not empty and not null)"
}
"#;

    rule.assert_detects(code);
}

#[test]
fn detects_common_unescaped_operator_patterns() {
    let rule = super::RULE;

    let code = r#"
def test_common [] {
    let item = "data"
    print $"Item ($item) (not available)"
    print $"Status ($item) (and more)"
    print $"Result ($item) (or else)"
    print $"Service ($item) (unable to process)"
}
"#;

    rule.assert_detects(code);
}

#[test]
fn detects_unescaped_or_operator_in_ansi_string() {
    let rule = super::RULE;

    let code = r#"
def fix_file [file: string] {
    if $has_errors {
        print $"(ansi yellow)⚠️  Could not apply some fixes to ($file) (or)(ansi reset)"
    } else {
        print $"(ansi green)✅ No issues in ($file)(ansi reset)"
    }
}
"#;

    rule.assert_detects(code);
    rule.assert_count(code, 1);
}
