use super::RULE;

#[test]
fn test_detect_sed_substitution() {
    let bad_code = r"^sed 's/foo/bar/'";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_sed_global_flag() {
    let bad_code = r"^sed 's/old/new/g'";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_sed_with_file() {
    let bad_code = r"^sed 's/pattern/replacement/' file.txt";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_sed_inplace() {
    let bad_code = r"^sed -i 's/old/new/' file.txt";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_sed_inplace_global() {
    let bad_code = r"^sed -i 's/foo/bar/g' config.ini";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_sed_combined_inplace_flags() {
    let bad_code = r"^sed -ie 's/test/prod/g' app.conf";
    RULE.assert_detects(bad_code);
}

#[test]
fn test_detect_gsed() {
    let bad_code = r"^gsed 's/foo/bar/'";
    RULE.assert_detects(bad_code);
}
