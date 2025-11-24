use super::rule;

#[test]
fn detect_main_with_pipeline_input_type_missing_stdin() {
    let source = r"#!/usr/bin/env nu

def main []: string -> string {
    $in | str upcase
}
";
    rule().assert_count(source, 1);
}

#[test]
fn detect_main_uses_in_variable_missing_stdin() {
    let source = r"#!/usr/bin/env nu

def main [] {
    let data = $in
    print $data
}
";
    rule().assert_count(source, 1);
}

#[test]
fn detect_main_with_list_input_type_missing_stdin() {
    let source = r#"#!/usr/bin/env nu

def main []: list<string> -> string {
    $in | str join ", "
}
"#;
    rule().assert_count(source, 1);
}

#[test]
fn detect_main_with_any_input_type_missing_stdin() {
    let source = r"#!/usr/bin/env nu

def main []: any -> any {
    $in
}
";
    rule().assert_count(source, 1);
}

#[test]
fn detect_main_with_table_input_missing_stdin() {
    let source = r"#!/usr/bin/env nu

def main []: table -> int {
    $in | length
}
";
    rule().assert_count(source, 1);
}

#[test]
fn detect_main_with_bare_each() {
    let source = r"#!/usr/bin/env nu

def main [] {
    each { |x| $x * 2 }
}
";
    rule().assert_count(source, 1);
}

#[test]
fn detect_main_with_bare_where() {
    let source = r"#!/usr/bin/env nu

def main [] {
    where { |x| $x > 2 }
}
";
    rule().assert_count(source, 1);
}

#[test]
fn detect_main_with_bare_reduce() {
    let source = r"#!/usr/bin/env nu

def main [] {
    reduce { |it, acc| $it + $acc }
}
";
    rule().assert_count(source, 1);
}

#[test]
fn detect_main_with_bare_items() {
    let source = r"#!/usr/bin/env nu

def main [] {
    items { |k, v| {key: $k, value: $v} }
}
";
    rule().assert_count(source, 1);
}

#[test]
fn detect_main_with_bare_select() {
    let source = r"#!/usr/bin/env nu

def main [] {
    select name age
}
";
    rule().assert_count(source, 1);
}
