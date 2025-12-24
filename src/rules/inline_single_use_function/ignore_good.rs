use super::RULE;
use crate::log::instrument;

#[test]
fn function_used_multiple_times() {
    RULE.assert_ignores(
        r"
def helper [] {
    42
}

def main [] {
    let a = helper
    let b = helper
    $a + $b
}
",
    );
}

#[test]
fn multi_line_function() {
    RULE.assert_ignores(
        r"
def process [] {
    let x = 5
    let y = 10
    $x + $y
}

def main [] {
    process
}
",
    );
}

#[test]
fn function_with_actual_multi_line_body() {
    instrument();
    RULE.assert_ignores(
        r"
def transform [] {
    $in | where size > 10
    | select name
}

def main [] {
    ls | transform
}
",
    );
}

#[test]
fn main_function_not_flagged() {
    RULE.assert_ignores(
        r"
def main [] {
    print 'Hello'
}
",
    );
}

#[test]
fn function_not_used() {
    RULE.assert_ignores(
        r"
def helper [] {
    42
}

def main [] {
    print 'Nothing'
}
",
    );
}

#[test]
fn exported_function() {
    RULE.assert_ignores(
        r"
export def helper [] {
    42
}

def main [] {
    helper
}
",
    );
}

#[test]
fn function_used_in_another_helper() {
    RULE.assert_ignores(
        r"
def helper [] {
    42
}

def process [] {
    let x = helper
    let y = helper
    $x + $y
}

def main [] {
    process
}
",
    );
}

#[test]
fn empty_function_body() {
    RULE.assert_ignores(
        r"
def noop [] {
}

def main [] {
    noop
}
",
    );
}

#[test]
fn function_called_recursively() {
    RULE.assert_ignores(
        r"
def factorial [n: int] {
    if $n <= 1 { 1 } else { $n * (factorial ($n - 1)) }
}

def main [] {
    factorial 5
}
",
    );
}

#[test]
fn script_without_main() {
    RULE.assert_ignores(
        r"
def helper [] {
    42
}

helper
",
    );
}

#[test]
fn function_with_comments_only_body() {
    RULE.assert_ignores(
        r"
def helper [] {
    # This is a comment
    # Another comment
}

def main [] {
    helper
}
",
    );
}
