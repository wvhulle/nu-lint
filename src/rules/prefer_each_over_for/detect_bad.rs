use super::rule;
use crate::LintContext;

#[test]
fn test_detect_for_loop_with_string_processing() {
    let bad_code = r"
for name in $names {
    $name | str capitalize
}
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(!(rule().check)(&context).is_empty());
    });
}

#[test]
fn test_detect_for_loop_with_record_access() {
    let bad_code = r"
for item in $users {
    $item.name
}
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(!(rule().check)(&context).is_empty());
    });
}

#[test]
fn test_detect_for_loop_with_math_operations() {
    let bad_code = r"
for x in $numbers {
    ($x | math sqrt) + 1
}
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(!(rule().check)(&context).is_empty());
    });
}

#[test]
fn test_detect_for_loop_with_data_transformation() {
    let bad_code = r"
for file in (ls | get name) {
    $file | path parse | get stem
}
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(!(rule().check)(&context).is_empty());
    });
}
