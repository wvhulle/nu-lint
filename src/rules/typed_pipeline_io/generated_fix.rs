use super::rule;

#[test]
fn test_fix_untyped_input() {
    let bad_code = r"
def double [] {
    $in * 2
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: any -> any");
    rule().assert_suggestion_contains(bad_code, "pipeline input type annotation");
}

#[test]
fn test_fix_untyped_output() {
    let bad_code = r"
def create-list [] {
    [1, 2, 3]
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: nothing -> any");
    rule().assert_suggestion_contains(bad_code, "pipeline output type annotation");
}

#[test]
fn test_fix_both_input_and_output() {
    let bad_code = r"
def transform [] {
    $in | each { |x| $x + 1 }
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: any -> any");
}

#[test]
fn test_fix_with_parameters() {
    let bad_code = r"
def multiply [factor: int] {
    $in * $factor
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[factor: int]: any -> any");
}

#[test]
fn test_fix_description_mentions_type_annotations() {
    let bad_code = "def double [] { $in * 2 }";
    rule().assert_fix_description_contains(bad_code, "type annotations");
}

#[test]
fn test_fix_preserves_optional_parameters() {
    let bad_code = r"
def process [data?, --verbose] {
    $in | str trim
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "data?");
    rule().assert_fix_contains(bad_code, "--verbose");
    rule().assert_fix_contains(bad_code, ": any -> any");
}

#[test]
fn test_fix_exported_function() {
    let bad_code = r"
export def process [] {
    $in | str trim
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: any -> any");
}

#[test]
fn test_fix_output_only() {
    let bad_code = r"
def get-value [] {
    42
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_contains(bad_code, "[]: nothing -> any");
}
