use crate::rules::pipeline_handle_errors::rule;

fn init_logger() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        crate::clean_log::log();
    });
}

#[test]
fn test_suggestion_recommends_do_ignore_when_appropriate() {
    instrument();
    let bad_code = r"^mkdir -p /tmp/test | ignore";
    rule().assert_fix_description_contains(bad_code, "do -i");
    rule().assert_fix_description_contains(bad_code, "ignore");
}


#[test]
fn test_suggestion_recommends_try_for_simple_cases() {
    instrument();
    let bad_code = r"^git status | lines";
    rule().assert_fix_description_contains(bad_code, "try");
    rule().assert_fix_description_contains(bad_code, "simple");
}
