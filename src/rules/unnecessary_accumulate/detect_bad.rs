use super::RULE;
use crate::log::init_env_log;

#[test]
fn test_detect_append_loop_over_literal_list() {
    init_env_log();

    let bad_code = r"
mut data = []
for x in [1 2 3] {
    $data = ($data | append $x)
}
";

    RULE.assert_detects(bad_code);
}
