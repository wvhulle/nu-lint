use super::rule;

#[test]
fn test_detect_missing_type_annotation_single_param() {
    let bad_code = r#"
def greet [name] {
    print $"Hello ($name)"
}
"#;
    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_missing_type_annotation_multiple_params() {
    let bad_code = r"
def add [x, y] {
    $x + $y
}
";
    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 2);
}

#[test]
fn test_detect_mixed_annotations() {
    let bad_code = r"
def process [data, format: string] {
    print $data
}
";
    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_nested_function() {
    let bad_code = r"
def outer [] {
    def inner [param] {
        print $param
    }
}
";

    rule().assert_detects(bad_code);
    rule().assert_violation_count_exact(bad_code, 1);
}
