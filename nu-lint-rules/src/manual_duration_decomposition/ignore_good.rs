use super::RULE;

#[test]
fn ignore_floor_division_alone() {
    RULE.assert_ignores("def to-minutes [total_seconds: int] { $total_seconds // 60 }");
}

#[test]
fn ignore_modulo_alone() {
    RULE.assert_ignores("def seconds-part [total_seconds: int] { $total_seconds mod 60 }");
}

#[test]
fn ignore_unrelated_divisors() {
    RULE.assert_ignores(
        "def split-pages [item_count: int] { [($item_count // 25) ($item_count mod 25)] }",
    );
}

#[test]
fn ignore_different_variables() {
    RULE.assert_ignores(
        "def split-values [first_total: int, second_total: int] { [($first_total // 60) \
         ($second_total mod 60)] }",
    );
}

#[test]
fn ignore_into_record_decomposition() {
    RULE.assert_ignores("def split-time [elapsed: duration] { $elapsed | into record }");
}
