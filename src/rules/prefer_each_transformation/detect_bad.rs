use super::rule;

#[test]
fn test_detect_simple_transformation() {
    let bad_code = r"
mut output = []
for x in $input {
    $output = ($output | append ($x * 2))
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_field_access_transformation() {
    let bad_code = r"
mut names = []
for item in $items {
    $names = ($names | append $item.name)
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_filtering_with_transformation() {
    let bad_code = r"
mut results = []
for x in $input {
    if $x > 5 {
        $results = ($results | append ($x * 2))
    }
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_complex_transformation() {
    let bad_code = r"
mut processed = []
for item in $data {
    $processed = ($processed | append ($item | str upcase))
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_filtering_with_field_transformation() {
    let bad_code = r"
mut active_names = []
for user in $users {
    if $user.active {
        $active_names = ($active_names | append $user.name)
    }
}
";

    rule().assert_detects(bad_code);
}
