use super::RULE;
use crate::log::init_test_log;

#[test]
fn boolean_switch_without_null_check() {
    init_test_log();
    // Boolean switches (flags without a type) are never null - they are true when
    // present, false when absent
    let good_code = r#"
def opt_flag [--opt] {
    $opt
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn boolean_switch_used_in_if() {
    init_test_log();
    // Boolean switches can be used directly in conditionals without null check
    let good_code = r#"
def my-command [--verbose] {
    if $verbose { print "Verbose mode" }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_checked_with_not_equal_null() {
    init_test_log();
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
    init_test_log();
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
    init_test_log();
    let good_code = r#"
def my-command [name: string] {
    print $name
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_declared_but_not_used() {
    init_test_log();
    let good_code = r#"
def my-command [--verbose] {
    print "Command executed"
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn multiple_flags_all_checked() {
    init_test_log();
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
    init_test_log();
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
    init_test_log();
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
    init_test_log();
    let good_code = r#"
def my-command [--output: string = "output.txt"] {
    save $output
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_checked_with_not() {
    init_test_log();
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
    init_test_log();
    let good_code = r#"
def my-command [--silent] {
    if not $silent {
        print "Not silent mode"
    }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_only_used_in_null_comparison() {
    init_test_log();
    let good_code = r#"
def my-command [--verbose] {
    let has_verbose = ($verbose != null)
    print $has_verbose
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_used_only_in_equality_check() {
    init_test_log();
    let good_code = r#"
def my-command [--mode: string] {
    if $mode == null {
        print "No mode"
    }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_in_compound_null_check() {
    init_test_log();
    let good_code = r#"
def my-command [--value: int] {
    if ($value != null) and ($value > 0) {
        print $value
    }
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn flag_checked_before_string_interpolation() {
    init_test_log();
    let good_code = r#"
def my-command [--name: string] {
    if $name != null {
        print $"Hello ($name)"
    }
}
"#;
    RULE.assert_ignores(good_code);
}
