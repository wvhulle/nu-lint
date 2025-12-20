use super::rule;
use crate::log::instrument;

#[test]
fn issue_64() {
    instrument();
    rule().assert_ignores("use std/log");
}

#[test]
fn function_with_no_params() {
    let good = "def hello [] { 'Hello World!' }";
    rule().assert_ignores(good);
}

#[test]
fn function_with_one_param() {
    let good = "def greet [name: string] { $'Hello ($name)!' }";
    rule().assert_ignores(good);
}

#[test]
fn function_with_two_params() {
    let good = "def add [a: int, b: int] { $a + $b }";
    rule().assert_ignores(good);
}

#[test]
fn function_with_flags() {
    let good = "def process [input: string, --verbose (-v), --output (-o): string] { echo $input }";
    rule().assert_ignores(good);
}

#[test]
fn function_with_optional_param() {
    let good = "def calc [x: int, y?: int] { $x + ($y | default 0) }";
    rule().assert_ignores(good);
}
