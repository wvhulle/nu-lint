use super::RULE;

#[test]
fn ignore_multiple_parameters() {
    let good_codes = vec![
        "def filter-range [data, min, max] { $data | where $it >= $min and $it <= $max }",
        "def process-with-config [items, config] { $items | each { |x| $x * $config.multiplier } }",
        "def backup-file [source, destination] { cp $source $destination }",
        "def greet [name, greeting] { $\"($greeting), ($name)!\" }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_no_parameters() {
    let good_codes = vec![
        "def get-current-time [] { date now }",
        "def list-files [] { ls }",
        "def show-help [] { help commands }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_optional_parameters() {
    let good_codes = vec![
        "def greet [name?] { $\"Hello, ($name | default 'World')!\" }",
        "def process [data, multiplier?: int] { $data | each { |x| $x * ($multiplier | default 1) \
         } }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_rest_parameters() {
    let good_codes = vec![
        "def multi-greet [...names] { $names | each { |name| $\"Hello, ($name)!\" } }",
        "def sum-all [...numbers] { $numbers | math sum }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_generator_commands() {
    let good_codes = vec![
        "def generate-range [start, end] { $start..$end }",
        "def create-list [size] { 1..$size }",
        "def make-record [name] { { name: $name, created: (date now) } }",
        "def build-table [rows] { 1..$rows | each { |i| { id: $i, value: ($i * 2) } } }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_configuration_parameters() {
    let good_codes = vec![
        "def configure-server [host, port] { $env.SERVER_HOST = $host; $env.SERVER_PORT = $port }",
        "def set-level [level] { $env.LOG_LEVEL = $level }",
        "def create-file [filename] { touch $filename }",
        "def delete-file [path] { rm $path }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_non_pipeline_usage() {
    let good_codes = vec![
        "def get-info [name] { { name: $name, timestamp: (date now) } }",
        "def calculate [value] { $value * 2 + 1 }",
        "def make-greeting [name] { $\"Hello, ($name)!\" }",
        "def create-config [setting] { { config: $setting, created: true } }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_path_and_filename_parameters() {
    let good_codes = vec![
        "def read-config [path] { open $path | from toml }",
        "def process-file [filename] { open $filename | lines | length }",
        "def save-data [data, path] { $data | save $path }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}

#[test]
fn ignore_typed_non_data_parameters() {
    let good_codes = vec![
        "def repeat [count: int] { 1..$count | each { |i| $\"Item ($i)\" } }",
        "def create-url [host: string, port: int] { $\"http://($host):($port)\" }",
        "def set-flag [enabled: bool] { if $enabled { \"on\" } else { \"off\" } }",
    ];

    for code in good_codes {
        RULE.assert_ignores(code);
    }
}
