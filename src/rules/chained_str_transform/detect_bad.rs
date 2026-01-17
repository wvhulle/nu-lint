use super::RULE;

#[test]
fn detect_two_consecutive_str_replace() {
    RULE.assert_detects("$text | str replace 'a' 'b' | str replace 'c' 'd'");
}

#[test]
fn detect_three_consecutive_str_replace() {
    RULE.assert_detects("$text | str replace 'a' 'b' | str replace 'c' 'd' | str replace 'e' 'f'");
}

#[test]
fn detect_in_function() {
    RULE.assert_detects(
        r#"
        def normalize [] {
            $in | str replace 'a' 'b' | str replace 'c' 'd'
        }
    "#,
    );
}

#[test]
fn detect_with_all_flag() {
    RULE.assert_detects("$text | str replace -a 'x' 'y' | str replace -a 'z' 'w'");
}

#[test]
fn detect_with_regex_flags() {
    RULE.assert_detects("$text | str replace -r 'a+' 'a' | str replace -r 'b+' 'b'");
}
