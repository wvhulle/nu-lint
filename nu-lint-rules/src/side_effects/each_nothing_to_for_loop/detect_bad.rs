use super::RULE;

#[test]
fn test_detect_each_with_print() {
    let bad_code = r"[1 2 3] | each {|x| print $x}";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_each_with_multiple_prints() {
    let bad_code = r#"
[1 2 3] | each {|x|
    print "Value:"
    print $x
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_complex_pipeline_with_each() {
    let bad_code = r"[1 2 3] | where $it > 1 | each {|x| print $x}";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_multi_stage_pipeline() {
    let bad_code = r"ls | where size > 1000 | sort-by name | each {|file| print $file.name}";
    RULE.assert_detects(bad_code);
}
