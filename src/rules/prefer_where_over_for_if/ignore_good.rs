use super::RULE;
use crate::log::instrument;

#[test]
fn test_ignore_functional_where_usage() {
    instrument();

    let good = "$input | where $it > 5";
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_for_loop_with_transformation() {
    instrument();

    // Should not flag when there's transformation applied
    let good = r"
mut results = []
for x in $input {
    if $x > 5 {
        $results = ($results | append ($x * 2))
    }
}
";
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_for_loop_with_multiple_statements() {
    instrument();

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
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_for_loop_without_filtering() {
    instrument();

    // Should not flag when there's no if statement
    let good = r"
mut output = []
for x in $input {
    $output = ($output | append ($x * 2))
}
";
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_for_loop_simple_copying() {
    instrument();

    // Should not flag simple copying
    let good = r"
mut data = []
for x in [1 2 3] {
    $data = ($data | append $x)
}
";
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_for_loop_complex_if_else_structure() {
    instrument();

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
    RULE.assert_ignores(good);
}
