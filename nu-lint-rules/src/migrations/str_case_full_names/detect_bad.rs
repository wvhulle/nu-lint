use super::RULE;

#[test]
fn detect_str_upcase() {
    RULE.assert_detects("'nu' | str upcase");
}

#[test]
fn detect_str_downcase_on_cell_path() {
    RULE.assert_detects("[[name]; [NU]] | str downcase name");
}

#[test]
fn detect_both_case_commands() {
    RULE.assert_count("'a' | str upcase | str downcase", 2);
}
