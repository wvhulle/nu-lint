use super::RULE;

#[test]
fn test_unnecessary_mut_fix_simple() {
    let bad_code = r"
def process [] {
    mut x = 5
    echo $x
}
";
    RULE.assert_detects(bad_code);
    RULE.assert_fix_explanation_contains(bad_code, "Remove 'mut'");
}

#[test]
fn test_unnecessary_mut_fix_description() {
    let bad_code = r"
def test [] {
    mut unused = 123
}
";
    RULE.assert_fix_explanation_contains(bad_code, "Remove 'mut' keyword");
    RULE.assert_fix_explanation_contains(bad_code, "unused");
}
