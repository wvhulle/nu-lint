use super::RULE;
use crate::log::instrument;

#[test]
fn test_detect_append_loop_over_literal_list() {
    instrument();

    let bad_code = r"
mut data = []
for x in [1 2 3] {
    $data = ($data | append $x)
}
";

    RULE.assert_detects(bad_code);
}
