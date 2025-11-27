use super::rule;

#[test]
fn test_detect_sed_substitution() {
    let bad_code = r"^sed 's/foo/bar/'";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_sed_global_flag() {
    let bad_code = r"^sed 's/old/new/g'";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_sed_with_file() {
    let bad_code = r"^sed 's/pattern/replacement/' file.txt";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_sed_inplace() {
    let bad_code = r"^sed -i 's/old/new/' file.txt";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_sed_inplace_global() {
    let bad_code = r"^sed -i 's/foo/bar/g' config.ini";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_sed_delete() {
    let bad_code = r"^sed '/pattern/d'";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_sed_combined_inplace_flags() {
    let bad_code = r"^sed -ie 's/test/prod/g' app.conf";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_sed_without_args() {
    let bad_code = r"^sed";
    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_gsed() {
    let bad_code = r"^gsed 's/foo/bar/'";
    rule().assert_detects(bad_code);
}
