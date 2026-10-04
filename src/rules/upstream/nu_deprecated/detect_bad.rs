use super::RULE;
#[test]
fn detect_get() {
    let source = "{name: 1} | get -i name";
    RULE.assert_detects(source);
}

#[test]
fn detect_str_upcase() {
    RULE.assert_detects("'nu' | str upcase");
}

#[test]
fn detect_str_downcase_on_cell_path() {
    RULE.assert_detects("[[name]; [NU]] | str downcase name");
}
