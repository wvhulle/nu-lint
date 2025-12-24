use super::RULE;

#[test]
fn detect_simple_cat() {
    RULE.assert_detects("^cat file.txt");
}

#[test]
fn detect_cat_multiple_files() {
    RULE.assert_detects("^cat file1.txt file2.txt");
}

#[test]
fn detect_cat_with_flags() {
    let bad_codes = vec![
        "^cat -n file.txt",
        "^cat -b file.txt",
        "^cat -E file.txt",
        "^cat -T file.txt",
        "^cat -A file.txt",
    ];

    for code in bad_codes {
        RULE.assert_detects(code);
    }
}

#[test]
fn detect_cat_long_options() {
    let bad_codes = vec![
        "^cat --number file.txt",
        "^cat --number-nonblank file.txt",
        "^cat --show-ends file.txt",
        "^cat --show-tabs file.txt",
        "^cat --show-all file.txt",
    ];

    for code in bad_codes {
        RULE.assert_detects(code);
    }
}

#[test]
fn detect_tac() {
    RULE.assert_detects("^tac file.log");
}

#[test]
fn detect_more() {
    RULE.assert_detects("^more documentation.txt");
}

#[test]
fn detect_less() {
    RULE.assert_detects("^less output.log");
}

#[test]
fn detect_cat_in_pipeline() {
    RULE.assert_detects("^cat file.txt | head -5");
}

#[test]
fn detect_cat_in_function() {
    let bad_code = r"
def read-file [path] {
    ^cat $path
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_multiple_cat_uses() {
    let bad_code = r"
^cat file1.txt
^cat file2.txt
";
    RULE.assert_count(bad_code, 2);
}

#[test]
fn detect_cat_in_subexpression() {
    let bad_code = r"
let content = (^cat config.json)
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_cat_in_closure() {
    let bad_code = r"
ls | each { |file|
    ^cat $file.name
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_pager_commands() {
    let bad_codes = vec![
        ("^more readme.md", 1),
        ("^less changelog.txt", 1),
        ("^tac reversed.log", 1),
    ];

    for (code, expected) in bad_codes {
        RULE.assert_count(code, expected);
    }
}
