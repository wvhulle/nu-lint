use super::RULE;

#[test]
fn function_with_four_typed_params() {
    let bad_code = r"
def complex-command [
    param1: string
    param2: int
    param3: bool
    param4: string
] {
    print $param1
}
";

    RULE.assert_detects(bad_code);
}

#[test]
fn function_with_five_inline_params() {
    let bad_code = r"
def too-many [a: int, b: int, c: int, d: int, e: int] {
    print $a
}
";

    RULE.assert_detects(bad_code);
}
