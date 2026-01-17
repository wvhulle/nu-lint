use super::RULE;

#[test]
fn detect_missing_type_annotation_single_param() {
    let bad_code = r#"
def greet [name] {
    print $"Hello ($name)"
}
"#;
    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn detect_missing_type_annotation_multiple_params() {
    let bad_code = r"
def add [x, y] {
    $x + $y
}
";
    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 2);
}

#[test]
fn detect_mixed_param_annotations() {
    let bad_code = r"
def process [data, format: string] {
    print $data
}
";
    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}

#[test]
fn detect_missing_annotation_in_nested_function() {
    let bad_code = r"
def outer [] {
    def inner [param] {
        print $param
    }
}
";

    RULE.assert_detects(bad_code);
    RULE.assert_count(bad_code, 1);
}
