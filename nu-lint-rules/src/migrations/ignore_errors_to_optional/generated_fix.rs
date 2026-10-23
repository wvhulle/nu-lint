use super::RULE;

#[test]
fn fix_get_short_flag() {
    RULE.assert_fixed_is("{a: 1} | get -i a", "{a: 1} | get -o a");
}

#[test]
fn fix_get_long_flag() {
    RULE.assert_fixed_is(
        "{a: 1} | get --ignore-errors a",
        "{a: 1} | get --optional a",
    );
}

#[test]
fn fix_keeps_cell_path_with_letter_i() {
    RULE.assert_fixed_is(
        "{file-id: 1} | get -i file-id",
        "{file-id: 1} | get -o file-id",
    );
}

#[test]
fn fix_select_short_flag() {
    RULE.assert_fixed_is("[[a]; [1]] | select -i a", "[[a]; [1]] | select -o a");
}

#[test]
fn fix_reject_long_flag() {
    RULE.assert_fixed_is(
        "[[a b]; [1 2]] | reject --ignore-errors a",
        "[[a b]; [1 2]] | reject --optional a",
    );
}
