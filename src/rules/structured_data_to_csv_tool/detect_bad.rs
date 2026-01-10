use super::RULE;

#[test]
fn detect_table_to_csv_tools() {
    RULE.assert_detects("ls | ^csvcut -c name");
    RULE.assert_detects("ls | ^csvstat");
    RULE.assert_detects("ls | ^csvgrep -c name -m test");
    RULE.assert_detects("ls | ^csvlook");
}
