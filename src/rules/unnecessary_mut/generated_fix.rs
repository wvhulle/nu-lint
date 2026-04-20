use super::RULE;

#[test]
fn test_unnecessary_mut_fix_simple() {
    let bad_code = r"
def process [] {
    mut x = 5
    echo $x
}
";
    let expected = r"
def process [] {
    x = 5
    echo $x
}
";

    RULE.assert_fixed_is(bad_code, expected);
}

#[test]
fn test_unnecessary_mut_fix_after_cjk_comment() {
    let bad_code = r"
def process [] {
    # 这里有中文注释，确保 mut 前面存在多字节字符
    mut value = 5
    echo $value
}
";
    let expected = r"
def process [] {
    # 这里有中文注释，确保 mut 前面存在多字节字符
    value = 5
    echo $value
}
";

    RULE.assert_fixed_is(bad_code, expected);
}
