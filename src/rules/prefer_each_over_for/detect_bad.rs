use super::rule;

#[test]
fn test_detect_for_loop_with_string_processing() {
    let bad_code = r"
for name in $names {
    $name | str capitalize
}
";
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_for_loop_with_record_access() {
    let bad_code = r"
for item in $users {
    $item.name
}
";
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_for_loop_with_math_operations() {
    let bad_code = r"
for x in $numbers {
    ($x | math sqrt) + 1
}
";
    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_for_loop_with_data_transformation() {
    let bad_code = r"
for file in (ls | get name) {
    $file | path parse | get stem
}
";

    rule().assert_violation_count_exact(bad_code, 1);
}
