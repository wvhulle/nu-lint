use nu_lint_analysis::log::init_test_log;

use super::RULE;

#[test]
fn test_detect_append_loop_over_literal_list() {
    init_test_log();

    let bad_code = r"
mut data = []
for x in [1 2 3] {
    $data = ($data | append $x)
}
";

    RULE.assert_detects(bad_code);
}
