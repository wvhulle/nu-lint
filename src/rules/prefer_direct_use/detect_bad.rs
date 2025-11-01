use super::rule;

#[test]
fn test_detect_simple_copy_accumulation() {
    let bad_code = r"
mut data = []
for x in [1 2 3] {
    $data = ($data | append $x)
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_copy_from_literal_list() {
    let bad_code = r"
mut items = []
for item in [1, 2, 3, 4, 5] {
    $items = ($items | append $item)
}
";

    rule().assert_detects(bad_code);
}
