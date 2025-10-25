use super::rule;

#[test]
fn test_ignore_builtin_ls() {
    let good_code = "ls -la";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_open_command() {
    let good_code = "open --raw config.toml";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_where_filter() {
    let good_code = r#"
$data | where name =~ "error"
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_first_last_commands() {
    let good_code = r"
$lines | first 5
$lines | last 10
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_builtin_sort_uniq() {
    let good_code = r"
$data | sort-by name | uniq-by id
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_external_commands_not_in_list() {
    let good_code = "^git status";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_specialized_external_tools() {
    let good_code = r"
^docker ps -a
^ffmpeg -i input.mp4 output.avi
^curl -X POST https://api.example.com/data
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_proper_pipeline_usage() {
    let good_code = r"
ls *.nu | where size > 1KB | sort-by modified | first 10
";
    rule().assert_ignores(good_code);
}
