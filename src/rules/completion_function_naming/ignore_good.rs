use super::rule;

#[test]
fn test_good_nu_complete_prefix() {
    let good = "def 'nu-complete git-branch' [] { git branch | lines }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_nu_complete_with_spaces() {
    let good = "def 'nu-complete file types' [] { ['txt', 'md', 'rs'] }";
    rule().assert_ignores(good);
}

#[test]
fn test_non_completion_function() {
    let good = "def process-data [] { echo 'processing' }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_simple_function() {
    let good = "def hello [name: string] { $'Hello ($name)!' }";
    rule().assert_ignores(good);
}

#[test]
fn test_good_complex_function() {
    let good = "def calculate-sum [numbers: list<int>] { $numbers | math sum }";
    rule().assert_ignores(good);
}
