use super::RULE;
use crate::log::init_test_log;

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

#[test]
fn closure_with_params_in_pipeline() {
    let good = "[1] | each {|x| $x }";
    RULE.assert_ignores(good);
}

#[test]
fn closure_with_params_in_assignment() {
    let good = r"
        let f = {|x| $x }
        do $f 1
    ";
    RULE.assert_ignores(good);
}

#[test]
fn closure_with_empty_params() {
    let good = r"
        let f = {|| 1 }
        do $f
    ";
    RULE.assert_ignores(good);
}

#[test]
fn record_in_assignment() {
    let good = r"
        let x = {a: 1}
        print $x
    ";
    RULE.assert_ignores(good);
}
