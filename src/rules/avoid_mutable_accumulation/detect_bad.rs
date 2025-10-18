use super::rule;

#[test]
fn test_detect_mutable_list_accumulation() {
    let bad_code = r"
mut results = []
for item in [1 2 3] {
    $results = ($results | append $item)
}
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_conditional_mutable_accumulation() {
    let bad_code = r"
mut filtered = []
for x in $data {
    if $x > 10 {
        $filtered = ($filtered | append $x)
    }
}
";

    rule().assert_detects(bad_code);
}
