use super::RULE;
use crate::log::init_test_log;

#[test]
fn test_ignore_functional_where_usage() {
    init_test_log();

    let good = "$input | where $it > 5";
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_for_loop_with_transformation() {
    init_test_log();

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
    init_test_log();

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
    init_test_log();

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
    init_test_log();

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
    init_test_log();

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
