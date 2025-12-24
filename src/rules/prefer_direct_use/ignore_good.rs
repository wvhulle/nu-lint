use super::RULE;

#[test]
fn test_immutable_list() {
    let good = "let items = [1, 2, 3]";
    RULE.assert_ignores(good);
}

#[test]
fn test_direct_assignment() {
    let good = "let data = [1 2 3]";
    RULE.assert_ignores(good);
}

#[test]
fn test_copy_from_variable() {
    // Should not flag when copying from a variable, not a literal
    let good = r"
mut data = []
for x in $input {
    $data = ($data | append $x)
}
";
    RULE.assert_ignores(good);
}

#[test]
fn test_transformation_in_loop() {
    // Should not flag when there's transformation
    let good = r"
mut data = []
for x in [1 2 3] {
    $data = ($data | append ($x * 2))
}
";
    RULE.assert_ignores(good);
}

#[test]
fn test_filtering_in_loop() {
    // Should not flag when there's filtering
    let good = r"
mut data = []
for x in [1 2 3] {
    if $x > 1 {
        $data = ($data | append $x)
    }
}
";
    RULE.assert_ignores(good);
}
