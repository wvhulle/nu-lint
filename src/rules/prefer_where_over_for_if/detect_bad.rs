use super::rule;

#[test]
fn test_detect_simple_filtering() {
    let bad_code = r"
mut filtered = []
for x in $input {
    if $x > 5 {
        $filtered = ($filtered | append $x)
    }
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_filtering_with_condition() {
    let bad_code = r"
mut selected = []
for item in $items {
    if $item > 10 {
        $selected = ($selected | append $item)
    }
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_filtering_with_field_access() {
    let bad_code = r"
mut active = []
for user in $users {
    if $user.active {
        $active = ($active | append $user)
    }
}
";

    rule().assert_detects(bad_code);
}
