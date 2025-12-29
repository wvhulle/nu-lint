use super::RULE;

#[test]
fn test_good_parse_multiline() {
    let good = r#"
let logs = [
    "[INFO] Starting server"
    "[ERROR] Connection failed"
]
$logs | each { |line| $line | parse '[{level}] {message}' }
"#;
    RULE.assert_ignores(good);
}

#[test]
fn test_ignore_each_without_split() {
    let good_code = r"
seq 1 10 | each { |x| $x * 2 }
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_good_split_for_iteration() {
    let good = "'a,b,c' | split row ',' | each { |item| $item | str upcase }";
    RULE.assert_ignores(good);
}

#[test]
fn test_good_parse_command() {
    let good = "'name:john age:30' | parse '{name}:{age}'";
    RULE.assert_ignores(good);
}
