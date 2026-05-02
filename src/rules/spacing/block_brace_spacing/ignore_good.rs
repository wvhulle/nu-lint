use crate::log::init_test_log;

use super::RULE;

#[test]
fn if_block_with_spaces() {
    let good = "if true { echo 'yes' }";
    RULE.assert_ignores(good);
}

#[test]
fn ambiguous_record_or_block() {
    init_test_log();
    let good = r#"
        let env_record = {$name: $value}
        load-env $env_record
    "#;
    RULE.assert_ignores(good);
}
