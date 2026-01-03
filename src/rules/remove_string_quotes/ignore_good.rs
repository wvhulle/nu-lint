use super::RULE;
use crate::log::init_log;

#[test]
fn ignore_command_position() {
    init_log();
    let code = r#"
        def main [] {
            ".md"
        }
     "#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_command_position_if() {
    init_log();
    let code = r#"
        def main [] {
            if $in {
                ".md"    
            }
            
        }
     "#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_bare_words() {
    let code = r#"echo hello world"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_with_spaces() {
    let code = r#"echo "hello world""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_starting_with_dash() {
    let code = r#"echo "-flag""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_starting_with_dollar() {
    let code = r#"echo "$variable""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_with_pipe() {
    let code = r#"echo "a|b""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_interpolation() {
    let code = r#"let name = world; echo $"hello ($name)""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_backtick_strings() {
    let code = r#"ls `test`"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_raw_strings() {
    let code = r#"echo r#'test'#"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_empty_string() {
    let code = r#"echo """#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_quoted_number() {
    // Without quotes, "123" would be parsed as int
    let code = r#"echo "123""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_quoted_float() {
    // Without quotes, "3.14" would be parsed as float
    let code = r#"echo "3.14""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_quoted_true() {
    // Without quotes, "true" would be parsed as boolean
    let code = r#"echo "true""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_quoted_false() {
    // Without quotes, "false" would be parsed as boolean
    let code = r#"echo "false""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_quoted_null() {
    // Without quotes, "null" would be parsed as nothing
    let code = r#"echo "null""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_with_semicolon() {
    // Semicolon separates statements
    let code = r#"echo "a;b""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_with_single_quote() {
    // Single quote would start a quoted string
    let code = r#"echo "it's""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_starting_with_hash() {
    // Hash starts a comment
    let code = "echo \"#comment\"";
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_starting_with_paren() {
    // Opening paren starts a subexpression
    let code = r#"echo "(1 + 2)""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_starting_with_bracket() {
    // Opening bracket starts a list
    let code = r#"echo "[1, 2]""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_starting_with_brace() {
    // Opening brace starts a record/closure
    let code = r#"echo "{a: 1}""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_double_ampersand() {
    // && is rejected by the parser
    let code = r#"echo "&&""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_in_command_position_let() {
    // Bare word in command position would be interpreted as command name
    // let foo = bar would try to run `bar` as a command
    let code = r#"let foo = "bar""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_in_command_position_if_block() {
    // Bare word in if block would be interpreted as command name
    // if true { fizz } would try to run `fizz` as a command
    let code = r#"if true {"fizz"}"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_in_command_position_else_block() {
    // Bare word in else block would be interpreted as command
    let code = r#"if false {} else {"buzz"}"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_in_command_position_closure() {
    // Bare word as closure body would be interpreted as command
    let code = r#"[1 2] | each {|x| "result"}"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_in_command_position_match_arm() {
    // Bare word in match arm would be interpreted as command
    let code = r#"match $x { 1 => "one" }"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_in_let_rhs() {
    // In Nushell, `let x = my_variable` would try to run `my_variable` as a command
    // So quotes are necessary to keep it as a string value
    let code = r#"let x = "my_variable""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_string_in_let_rhs_with_dash() {
    // Same as above - bare word would be interpreted as command
    let code = r#"let x = "my-command""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_pattern_with_asterisk() {
    // Without quotes, "*.txt" would be a glob pattern matching files
    // With quotes, it's a literal string containing an asterisk
    let code = r#"echo "*.txt""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_pattern_with_question_mark() {
    // Without quotes, "?.txt" would be a glob pattern
    // With quotes, it's a literal string containing a question mark
    let code = r#"echo "?.txt""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_pattern_with_multiple_wildcards() {
    // Complex glob pattern with multiple metacharacters
    let code = r#"echo "**/*.rs""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_pattern_in_string() {
    // Question mark in middle of string
    let code = r#"echo "file?.txt""#;
    RULE.assert_ignores(code);
}

#[test]
fn ignores_glob_pattern_asterisk_in_middle() {
    // Asterisk in middle of string
    let code = r#"echo "test*file""#;
    RULE.assert_ignores(code);
}
