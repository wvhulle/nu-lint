use super::rule;

#[test]
fn test_unnecessary_mut_fix_simple() {
    let bad_code = r"
def process [] {
    mut x = 5
    echo $x
}
";
    rule().assert_detects(bad_code);
    rule().assert_fix_description_contains(bad_code, "Remove 'mut'");
}

#[test]
fn test_unnecessary_mut_fix_description() {
    let bad_code = r"
def test [] {
    mut unused = 123
}
";
    rule().assert_fix_description_contains(bad_code, "Remove 'mut' keyword");
    rule().assert_fix_description_contains(bad_code, "unused");
}
