use super::RULE;

#[test]
fn detect_missing_output_type_annotation() {
    let bad_code = r"
def create-list [] {
    [1, 2, 3]
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_output_type_is_any() {
    let bad_code = r"
def create-list []: int -> any {
    [1, 2, 3]
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_function_with_in_missing_output() {
    let bad_code = r"
def transform [] {
    $in | each { |x| $x + 1 }
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_exported_function_missing_output_type() {
    let bad_code = r#"
export def "git age" [] {
  git branch | lines | str substring 2.. | wrap name | insert last_commit {
    get name | each {
      git show $in --no-patch --format=%as | into datetime
    }
  } | sort-by last_commit
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_multiple_functions_missing_output_types() {
    let bad_code = r"
def first [] { [1, 2, 3] }
def second [] { {a: 1, b: 2} }
";
    RULE.assert_count(bad_code, 2);
}
