use super::RULE;

#[test]
fn test_fix_string_to_path() {
    let bad_code = "def process-file [file_path: string] { open $file_path }";
    RULE.assert_fixed_contains(bad_code, "file_path: path");
}

#[test]
fn test_fix_untyped_to_path() {
    let bad_code = "def process-file [file_path] { open $file_path }";
    RULE.assert_fixed_contains(bad_code, "file_path: path");
}

#[test]
fn test_fix_optional_string_to_path() {
    let bad_code =
        "def process-file [file_path?: string] { if ($file_path != null) { open $file_path } }";
    RULE.assert_fixed_contains(bad_code, "file_path?: path");
}

#[test]
fn fix_keeps_parameter_after_space() {
    RULE.assert_fixed_contains(
        "def read-file [file other: int] { open $file; print $other }",
        "def read-file [file: path other: int]",
    );
}

#[test]
fn fix_keeps_default_value() {
    RULE.assert_fixed_contains(
        r#"def read-file [file: string = "a,b", other?: int] { open $file; print $other }"#,
        r#"def read-file [file: path = "a,b", other?: int]"#,
    );
}
