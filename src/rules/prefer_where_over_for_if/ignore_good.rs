use super::rule;

#[test]
fn test_functional_where_filter() {
    let good = "$input | where $it > 5";
    rule().assert_ignores(good);
}

#[test]
fn test_filtering_with_transformation() {
    // Should not flag when there's transformation applied
    let good = r"
mut results = []
for x in $input {
    if $x > 5 {
        $results = ($results | append ($x * 2))
    }
}
";
    rule().assert_ignores(good);
}

#[test]
fn test_multiple_statements_in_loop() {
    // Should not flag when there are multiple statements
    let good = r"
mut filtered = []
for x in $input {
    print $x
    if $x > 5 {
        $filtered = ($filtered | append $x)
    }
}
";
    rule().assert_ignores(good);
}

#[test]
fn test_simple_transformation_without_filtering() {
    // Should not flag when there's no if statement
    let good = r"
mut output = []
for x in $input {
    $output = ($output | append ($x * 2))
}
";
    rule().assert_ignores(good);
}

#[test]
fn test_direct_copy_without_filtering() {
    // Should not flag simple copying
    let good = r"
mut data = []
for x in [1 2 3] {
    $data = ($data | append $x)
}
";
    rule().assert_ignores(good);
}

#[test]
fn test_complex_filtering_logic() {
    // Should not flag complex if-else structures
    let good = r"
mut switches = []
mut named = []
for it in $params {
    if $it.type == 'switch' {
        $switches = $switches | append $it.name
    } else if $it.type == 'named' {
        $named = $named | append $it.name
    }
}
";
    rule().assert_ignores(good);
}
