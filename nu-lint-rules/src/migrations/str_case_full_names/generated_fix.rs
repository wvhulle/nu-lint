use super::RULE;

#[test]
fn fix_str_upcase_to_uppercase() {
    RULE.assert_fixed_is("'nu' | str upcase", "'nu' | str uppercase");
}

#[test]
fn fix_str_downcase_keeps_cell_path_arguments() {
    RULE.assert_fixed_is(
        "[[name]; [NU]] | str downcase name",
        "[[name]; [NU]] | str lowercase name",
    );
}

#[test]
fn fix_str_upcase_inside_closure() {
    RULE.assert_fixed_is(
        "['a' 'b'] | each {|s| $s | str upcase }",
        "['a' 'b'] | each {|s| $s | str uppercase }",
    );
}
