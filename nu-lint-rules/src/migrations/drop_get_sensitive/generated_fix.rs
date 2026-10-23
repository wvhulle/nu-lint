use super::RULE;

#[test]
fn fix_removes_flag() {
    RULE.assert_fixed_is("{a: 1} | get --sensitive a", "{a: 1} | get a");
}

#[test]
fn fix_removes_flag_between_cell_paths() {
    RULE.assert_fixed_is("{a: {b: 1}} | get a --sensitive b", "{a: {b: 1}} | get a b");
}
