use super::RULE;
use crate::log::init_env_log;

#[test]
fn test_detect_for_loop_simple_numeric_filtering() {
    init_env_log();

    let bad_code = r"
mut filtered = []
for x in $input {
    if $x > 5 {
        $filtered = ($filtered | append $x)
    }
}
";

    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_for_loop_field_access_filtering() {
    init_env_log();

    let bad_code = r"
mut active = []
for user in $users {
    if $user.active {
        $active = ($active | append $user)
    }
}
";

    RULE.assert_detects(bad_code);
}
