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
}
