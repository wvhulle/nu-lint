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
