use super::RULE;

#[test]
fn fix_get_zero_index() {
    RULE.assert_fixed_is("$list | get 0", "$list | get -o 0");
}

#[test]
fn fix_get_large_index() {
    RULE.assert_fixed_is("$list | get 42", "$list | get -o 42");
}

#[test]
fn fix_in_pipeline() {
    RULE.assert_fixed_is("ls | get 0", "ls | get -o 0");
}

#[test]
fn fix_in_closure() {
    RULE.assert_fixed_is(
        "$data | each {|row| $row | get 0 }",
        "$data | each {|row| $row | get -o 0 }",
    );
}
