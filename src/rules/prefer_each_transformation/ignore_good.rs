use super::rule;

#[test]
fn test_functional_each_transformation() {
    let good = "$input | each { |x| $x * 2 }";
    rule().assert_ignores(good);
}

#[test]
fn test_functional_where_and_each() {
    let good = "$input | where $it > 5 | each { |x| $x * 2 }";
    rule().assert_ignores(good);
}

#[test]
fn test_simple_filtering_without_transformation() {
    // Should not flag simple filtering (that's for prefer_where_over_for_if)
    let good = r"
mut filtered = []
for x in $input {
    if $x > 5 {
        $filtered = ($filtered | append $x)
    }
}
";
    rule().assert_ignores(good);
}

#[test]
fn test_direct_copy_without_transformation() {
    // Should not flag direct copying (that's for prefer_direct_use)
    let good = r"
mut data = []
for x in [1 2 3] {
    $data = ($data | append $x)
}
";
    rule().assert_ignores(good);
}

#[test]
fn test_pagination_with_transformation() {
    // Should not flag complex pagination patterns
    let good = r"
mut stars = []
mut end_cursor = ''
loop {
    let part = gh get stars $end_cursor
    $stars = $stars | append $part.stars
    if $part.pageInfo?.hasNextPage? == true {
        $end_cursor = $part.pageInfo.endCursor
    } else {
        break
    }
}
";
    rule().assert_ignores(good);
}

#[test]
fn test_accumulation_with_side_effects() {
    // Should not flag when there are side effects
    let good = r"
mut results = []
for file in $files {
    print $'Processing ($file)'
    $results = $results | append (process $file)
}
";
    rule().assert_ignores(good);
}

#[test]
fn test_immutable_transformation() {
    let good = "let output = $input | each { |x| $x * 2 }";
    rule().assert_ignores(good);
}
