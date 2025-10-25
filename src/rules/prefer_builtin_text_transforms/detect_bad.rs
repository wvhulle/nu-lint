use super::rule;

#[test]
fn test_detect_external_sed() {
    let bad_code = "^sed 's/foo/bar/' file.txt";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_sed_in_place() {
    let bad_code = "^sed -i 's/old/new/g' *.txt";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_sed_pipeline() {
    let bad_code = "cat file.txt | ^sed 's/pattern/replacement/'";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_awk() {
    let bad_code = "^awk '{print $1}' file.txt";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_awk_field_separator() {
    let bad_code = "^awk -F',' '{print $2}' data.csv";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_awk_filtering() {
    let bad_code = "^awk '$3 > 100 {print $0}' data.txt";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_cut() {
    let bad_code = "^cut -d ',' -f 1 file.csv";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_wc() {
    let bad_code = "^wc -l file.txt";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_tr() {
    let bad_code = "^tr 'a-z' 'A-Z' file.txt";

    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_detect_external_tr_delete() {
    let bad_code = "^tr -d '\n' < file.txt";

    rule().assert_detects(bad_code);
}

#[test]
fn test_detect_external_wc_lines_words() {
    let bad_code = "^wc -lw *.txt";

    rule().assert_violation_count_exact(bad_code, 1);
}
