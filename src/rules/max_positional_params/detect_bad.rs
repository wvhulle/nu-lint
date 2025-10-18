
use super::rule;

#[test]
fn test_detect_too_many_positional_params_complex() {
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

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_too_many_positional_params_simple() {
    let bad_code = r"
def too-many [a: int, b: int, c: int, d: int, e: int] {
    print $a
}
";

    rule().assert_detects(bad_code);
}
