use super::RULE;
use crate::log::instrument;

#[test]
fn flag_checked_with_not_equal_null() {
    instrument();
    let good_code = r#"
def my-command [--verbose] {
    if $verbose != null {
        print "Verbose mode"
    }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_checked_with_equal_null() {
    instrument();
    let good_code = r#"
def my-command [--verbose] {
    if $verbose == null {
        print "Not verbose"
    } else {
        print "Verbose mode"
    }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn command_without_flags() {
    instrument();
    let good_code = r#"
def my-command [name: string] {
    print $name
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_declared_but_not_used() {
    instrument();
    let good_code = r#"
def my-command [--verbose] {
    print "Command executed"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn multiple_flags_all_checked() {
    instrument();
    let good_code = r#"
def my-command [--verbose --debug] {
    if $verbose != null {
        print "Verbose"
    }
    if $debug != null {
        print "Debug"
    }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_null_check_in_nested_expression() {
    instrument();
    let good_code = r#"
def my-command [--count: int] {
    if $count != null and $count > 0 {
        1..$count
    }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_with_reversed_null_check() {
    instrument();
    let good_code = r#"
def my-command [--verbose] {
    if null != $verbose {
        print "Verbose mode"
    }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_with_default_value() {
    instrument();
    let good_code = r#"
def my-command [--output: string = "output.txt"] {
    save $output
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_checked_with_not() {
    instrument();
    let good_code = r#"
def my-command [--silent] {
    if (not $silent) {
        print "Not silent mode"
    }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_checked_with_not_no_parens() {
    instrument();
    let good_code = r#"
def my-command [--silent] {
    if not $silent {
        print "Not silent mode"
    }
}
"#;
    RULE.assert_ignores(good_code);
}
