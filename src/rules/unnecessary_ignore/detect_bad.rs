use super::rule;

#[test]
fn test_mkdir_with_ignore() {
    let bad_code = "mkdir /tmp/test | ignore";
    rule().assert_detects(bad_code);
}

#[test]
fn test_cd_with_ignore() {
    let bad_code = "cd /tmp | ignore";
    rule().assert_detects(bad_code);
}

#[test]
fn test_rm_with_ignore() {
    let bad_code = "rm /tmp/file.txt | ignore";
    rule().assert_detects(bad_code);
}

#[test]
fn test_mv_with_ignore() {
    let bad_code = "mv old.txt new.txt | ignore";
    rule().assert_detects(bad_code);
}

#[test]
fn test_touch_with_ignore() {
    let bad_code = "touch newfile.txt | ignore";
    rule().assert_detects(bad_code);
}

#[test]
fn test_in_function() {
    let bad_code = r"
def setup [] {
    mkdir /tmp/data | ignore
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_in_closure() {
    let bad_code = r#"
[1 2 3] | each { |x| mkdir $"/tmp/dir($x)" | ignore }
"#;
    rule().assert_detects(bad_code);
}
