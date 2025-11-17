use super::rule;

#[test]
fn ignore_main_with_stdin_flag_standard() {
    let source = r"#!/usr/bin/env -S nu --stdin

def main []: string -> string {
    $in | str upcase
}
";
    rule().assert_count(source, 0);
}

#[test]
fn ignore_main_with_stdin_flag_direct() {
    let source = r"#!/usr/bin/env nu --stdin

def main []: string -> string {
    $in | str upcase
}
";
    rule().assert_count(source, 0);
}

#[test]
fn ignore_main_without_pipeline_input() {
    let source = r#"#!/usr/bin/env nu

def main [] {
    print "Hello"
}
"#;
    rule().assert_count(source, 0);
}

#[test]
fn ignore_main_with_nothing_input_type() {
    let source = r#"#!/usr/bin/env nu

def main []: nothing -> string {
    "Hello"
}
"#;
    rule().assert_count(source, 0);
}

#[test]
fn ignore_main_with_positional_args_only() {
    let source = r#"#!/usr/bin/env nu

def main [name: string] {
    print $"Hello ($name)"
}
"#;
    rule().assert_count(source, 0);
}

#[test]
fn ignore_helper_function_uses_in() {
    let source = r#"#!/usr/bin/env nu

def helper [] {
    let data = $in
    print $data
}

def main [] {
    "test" | helper
}
"#;
    rule().assert_count(source, 0);
}

#[test]
fn ignore_script_without_shebang() {
    let source = r"
def main []: string -> string {
    $in | str upcase
}
";
    rule().assert_count(source, 0);
}

#[test]
fn ignore_main_with_stdin_and_uses_in() {
    let source = r"#!/usr/bin/env -S nu --stdin

def main [] {
    let data = $in
    print $data
}
";
    rule().assert_count(source, 0);
}
