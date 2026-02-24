use super::RULE;
use crate::log::init_test_log;

#[test]
fn ignore_conditional_length_in() {
    init_test_log();

    let good_code = "def check-length [] { if ($in | length) > 5 { \"big\" } else { \"small\" } }";

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_conditional_in_empty() {
    init_test_log();

    let good_code =
        "def conditional-process [] { if ($in | is-empty) { [] } else { $in | first 10 } }";

    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_no_in_usage() {
    let good_codes = vec![
        "def greet [name] { $\"Hello, ($name)!\" }",
        "def add [a, b] { $a + $b }",
        "def get-current-time [] { date now }",
        "def create-list [size] { 1..$size }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_in_not_at_start() {
    let good_codes = vec![
        "def process [] { date now | if ($in | get hour) > 12 { \"PM\" } else { \"AM\" } }",
        "def combine [prefix] { $\"($prefix): \" + ($in | into string) }",
        "def validate [] { let input = $in; if ($input | is-empty) { \"empty\" } else { \"ok\" } }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_positional_parameter_usage() {
    let good_codes = vec![
        "def process-data [data] { $data | where active | select name }",
        "def transform [items] { $items | each { |x| $x * 2 } }",
        "def filter [numbers, threshold] { $numbers | where $it > $threshold }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_commands_without_pipelines() {
    let good_codes = vec![
        "def simple-calc [a, b] { $a + $b }",
        "def make-greeting [name] { $\"Hello, ($name)!\" }",
        "def get-constant [] { 42 }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_wrapped_in() {
    let good_codes = vec![
        "def min-zero [] { [$in 0] | math max }",
        "def json-record [] { {key: $in} | to json }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}
