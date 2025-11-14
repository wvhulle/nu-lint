use super::rule;

#[test]
fn test_detect_multiline_nested_if() {
    let bad_code = r#"
if $x > 0 {
    if $y > 0 {
        print "both positive"
    }
}
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_inline_nested_if() {
    let bad_code = r#"if $a { if $b { print "nested" } }"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_nested_if_inside_function() {
    let bad_code = r#"
def check-conditions [] {
    if $x == 1 {
        if $y == 2 {
            echo "matched"
        }
    }
}
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_nested_if_with_boolean_conditions() {
    let bad_code = r#"
if ($status == "active") {
    if ($enabled == true) {
        do-something
    }
}
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_nested_if_with_string_conditions() {
    let bad_code = r#"
if $env == "prod" {
    if $region == "us-west" {
        deploy
    }
}
"#;

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_nested_if_with_complex_expressions() {
    let bad_code = r"
if ($x > 10 and $x < 100) {
    if $y != null {
        process $x $y
    }
}
";

    rule().assert_detects(bad_code);
}
