use super::rule;

#[test]
fn ignore_conditional_length_in() {
    crate::clean_log::log();

    let good_code = "def check-length [] { if ($in | length) > 5 { \"big\" } else { \"small\" } }";

    rule().assert_ignores(good_code);
}

#[test]
fn ignore_conditional_in_empty() {
    crate::clean_log::log();

    let good_code =
        "def conditional-process [] { if ($in | is-empty) { [] } else { $in | first 10 } }";

    rule().assert_ignores(good_code);
}

#[test]
fn ignore_no_in_usage() {
    let good_codes = vec![
        // Commands that don't use $in at all
        "def greet [name] { $\"Hello, ($name)!\" }",
        "def add [a, b] { $a + $b }",
        "def get-current-time [] { date now }",
        "def create-list [size] { 1..$size }",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_in_not_at_start() {
    let good_codes = vec![
        // $in used later in the pipeline, not at the start
        "def process [] { date now | if ($in | get hour) > 12 { \"PM\" } else { \"AM\" } }",
        "def combine [prefix] { $\"($prefix): \" + ($in | into string) }",
        "def validate [] { let input = $in; if ($input | is-empty) { \"empty\" } else { \"ok\" } }",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_positional_parameter_usage() {
    let good_codes = vec![
        // Commands that use positional parameters properly
        "def process-data [data] { $data | where active | select name }",
        "def transform [items] { $items | each { |x| $x * 2 } }",
        "def filter [numbers, threshold] { $numbers | where $it > $threshold }",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}

#[test]
fn ignore_commands_without_pipelines() {
    let good_codes = vec![
        // Commands that don't use pipelines
        "def simple-calc [a, b] { $a + $b }",
        "def make-greeting [name] { $\"Hello, ($name)!\" }",
        "def get-constant [] { 42 }",
    ];

    for code in good_codes {
        rule().assert_ignores(code);
    }
}
