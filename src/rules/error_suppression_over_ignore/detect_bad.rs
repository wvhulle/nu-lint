use super::rule;

#[test]
fn test_rm_with_ignore_detected() {
    let bad_code = r"
rm /tmp/some_file.txt | ignore
";

    rule().assert_detects(bad_code);
}

#[test]
fn test_mv_with_ignore_detected() {
    let bad_code = "mv old.txt new.txt | ignore";

    rule().assert_detects(bad_code);
}

#[test]
fn test_cp_with_ignore_detected() {
    let bad_code = "cp source.txt dest.txt | ignore";

    rule().assert_detects(bad_code);
}

#[test]
fn test_mkdir_with_ignore_detected() {
    let bad_code = "mkdir new_directory | ignore";

    rule().assert_detects(bad_code);
}

#[test]
fn test_touch_with_ignore_detected() {
    let bad_code = "touch new_file.txt | ignore";

    rule().assert_detects(bad_code);
}

#[test]
fn test_rmdir_with_ignore_detected() {
    let bad_code = "rmdir old_directory | ignore";

    rule().assert_detects(bad_code);
}

#[test]
fn test_file_operation_with_pipeline_detected() {
    let bad_code = r"
ls | where name == 'test' | each { |x| rm $x.name } | ignore
";

    rule().assert_detects(bad_code);
}
