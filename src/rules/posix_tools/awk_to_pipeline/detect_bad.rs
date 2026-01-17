use super::RULE;

#[test]
fn detects_simple_awk() {
    RULE.assert_detects("^awk");
}

#[test]
fn detects_awk_with_program() {
    RULE.assert_detects(r#"^awk '{print $1}'"#);
}

#[test]
fn detects_awk_with_file() {
    RULE.assert_detects(r#"^awk '{print $1}' input.txt"#);
}

#[test]
fn detects_awk_with_field_separator() {
    RULE.assert_detects(r#"^awk -F: '{print $1}' /etc/passwd"#);
}

#[test]
fn detects_awk_with_separate_field_separator() {
    RULE.assert_detects(r#"^awk -F "," '{print $2}' data.csv"#);
}

#[test]
fn detects_awk_with_pattern() {
    RULE.assert_detects(r#"^awk '/error/' logfile"#);
}

#[test]
fn detects_awk_with_pattern_and_action() {
    RULE.assert_detects(r#"^awk '/warning/ {print $0}' logs.txt"#);
}

#[test]
fn detects_awk_with_variable_assignment() {
    RULE.assert_detects(r#"^awk -v name=value '{print $1}' file.txt"#);
}

#[test]
fn detects_gawk() {
    RULE.assert_detects(r#"^gawk '{print $1}'"#);
}

#[test]
fn detects_mawk() {
    RULE.assert_detects(r#"^mawk '{print $2}' input.txt"#);
}

#[test]
fn detects_awk_in_pipeline() {
    RULE.assert_detects(r#"cat file.txt | ^awk '{print $1}'"#);
}

#[test]
fn detects_awk_in_function() {
    let bad_code = r#"
def extract-field [file: string] {
    ^awk '{print $1}' $file
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_multiple_awk_uses() {
    let bad_code = r#"
^awk '{print $1}' file1.txt
^awk -F, '{print $2}' file2.csv
"#;
    RULE.assert_count(bad_code, 2);
}

#[test]
fn detects_awk_with_multiple_print_fields() {
    RULE.assert_detects(r#"^awk '{print $1, $3}' data.txt"#);
}

#[test]
fn detects_awk_with_nr() {
    RULE.assert_detects(r#"^awk '{print NR, $0}' file.txt"#);
}

#[test]
fn detects_awk_with_nf() {
    RULE.assert_detects(r#"^awk '{print NF}' file.txt"#);
}

#[test]
fn detects_awk_with_complex_pattern() {
    RULE.assert_detects(r#"^awk '/^[0-9]+/ {print $2}' input.txt"#);
}

#[test]
fn detects_awk_in_subexpression() {
    let bad_code = r#"
let result = (^awk '{print $1}' data.txt)
print $result
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detects_awk_in_closure() {
    let bad_code = r#"
ls | each { |file|
    ^awk '{print $1}' $file.name
}
"#;
    RULE.assert_detects(bad_code);
}
