use super::RULE;

#[test]
fn test_detect_in_non_main_function() {
    let code = r#"
        def check-file [path: string] {
            if not ($path | path exists) {
                print --stderr "File does not exist"
            }
        }
    "#;
    RULE.assert_detects(code);
}

#[test]
fn test_detect_in_helper_function() {
    let code = r#"
        def validate [input: string] {
            if ($input | is-empty) {
                print --stderr "Input cannot be empty"
            }
        }
    "#;
    RULE.assert_detects(code);
}

#[test]
fn test_detect_in_try_block() {
    let code = r#"
        def main [] {
            try {
                print --stderr "Error occurred"
            }
        }
    "#;
    RULE.assert_detects(code);
}

#[test]
fn test_detect_in_try_block_top_level() {
    let code = r#"
        try {
            print --stderr "Error in try"
        }
    "#;
    RULE.assert_detects(code);
}

#[test]
fn test_detect_in_exported_function() {
    let code = r#"
        export def process-data [data] {
            if ($data | is-empty) {
                print --stderr "No data provided"
            }
        }
    "#;
    RULE.assert_detects(code);
}

#[test]
fn test_detect_multiple_in_function() {
    let code = r#"
        def validate [input] {
            if ($input | is-empty) {
                print --stderr "Empty input"
            }
            if ($input | str length) > 100 {
                print --stderr "Input too long"
            }
        }
    "#;
    RULE.assert_count(code, 2);
}

#[test]
fn test_detect_nested_function() {
    let code = r#"
        def outer [] {
            def inner [] {
                print --stderr "Inner error"
            }
        }
    "#;
    RULE.assert_detects(code);
}
