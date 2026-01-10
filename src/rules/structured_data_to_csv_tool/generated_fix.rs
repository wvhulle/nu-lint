use super::RULE;

#[test]
fn fix_adds_to_csv() {
    RULE.assert_fixed_is("ls | ^csvcut -c name", "ls | to csv | ^csvcut -c name");
    RULE.assert_fixed_is("ls | ^csvstat", "ls | to csv | ^csvstat");
    RULE.assert_fixed_is("ls | ^csvlook", "ls | to csv | ^csvlook");
}
