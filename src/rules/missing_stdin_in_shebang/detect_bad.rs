use super::RULE;

#[test]
fn detect_main_with_pipeline_input_type_missing_stdin() {
    let source = r"#!/usr/bin/env nu

def main []: string -> string {
    $in | str upcase
}
";
    RULE.assert_count(source, 1);
}

#[test]
fn detect_main_uses_in_variable_missing_stdin() {
    let source = r"#!/usr/bin/env nu

def main [] {
    let data = $in
    print $data
}
";
    RULE.assert_count(source, 1);
}

#[test]
fn detect_main_with_list_input_type_missing_stdin() {
    let source = r#"#!/usr/bin/env nu

def main []: list<string> -> string {
    $in | str join ", "
}
"#;
    RULE.assert_count(source, 1);
}

#[test]
fn detect_main_with_any_input_type_missing_stdin() {
    let source = r"#!/usr/bin/env nu

def main []: any -> any {
    $in
}
";
    RULE.assert_count(source, 1);
}

#[test]
fn detect_main_with_table_input_missing_stdin() {
    let source = r"#!/usr/bin/env nu

def main []: table -> int {
    $in | length
}
";
    RULE.assert_count(source, 1);
}

#[test]
fn detect_main_with_bare_each() {
    let source = r"#!/usr/bin/env nu

def main [] {
    each { |x| $x * 2 }
}
";
    RULE.assert_count(source, 1);
}

#[test]
fn detect_main_with_bare_where() {
    let source = r"#!/usr/bin/env nu

def main [] {
    where { |x| $x > 2 }
}
";
    RULE.assert_count(source, 1);
}

#[test]
fn detect_main_with_bare_reduce() {
    let source = r"#!/usr/bin/env nu

def main [] {
    reduce { |it, acc| $it + $acc }
}
";
    RULE.assert_count(source, 1);
}

#[test]
fn detect_main_with_bare_items() {
    let source = r"#!/usr/bin/env nu

def main [] {
    items { |k, v| {key: $k, value: $v} }
}
";
    RULE.assert_count(source, 1);
}

#[test]
fn detect_main_with_bare_select() {
    let source = r"#!/usr/bin/env nu

def main [] {
    select name age
}
";
    RULE.assert_count(source, 1);
}
