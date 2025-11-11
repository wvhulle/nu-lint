use crate::log::instrument;

use super::rule;

#[test]
fn test_untyped_pipeline_input() {
    let bad_code = r"
def double [] {
    $in * 2
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_untyped_pipeline_output() {
    let bad_code = r"
def create-list [] {
    [1, 2, 3]
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_missing_pipeline_annot_parse() {
    instrument();

    let bad_code = r"
export def parse [] {
  let token = ($in | token)
  let sign = (get-sign $token.0)
  mut sw = ''
  mut pos = []
  mut opt = {}
  for c in $token {
    if ($sw | is-empty) {
      if ($c | str starts-with '-') {
        let c = if ($c | str substring 1..<2) != '-' {
          let k = ($c | str substring 1..)
          if $k in $sign.name {
            $'($sign.name | get $k)'
          } else {
            $k
          }
        } else {
          $c | str substring 2..
        }
        if $c in $sign.switch {
          $opt = ($opt | upsert $c true)
        } else {
          $sw = $c
        }
      } else {
        $pos ++= [$c]
      }
    } else {
      $opt = ($opt | upsert $sw $c)
      $sw = ''
    }
  }
  $opt._args = $pos
  let p = $pos | slice 1..($sign.positional | length)
  let rest = $pos | slice (($sign.positional | length) + 1)..-1
  $opt._pos = (
    $p | enumerate
    | reduce -f {} {|it acc|
      $acc | upsert ($sign.positional | get $it.index) $it.item
    }
  )
  if ($sign.rest | length) > 0 {
    $opt._pos = ($opt._pos | upsert $sign.rest.0 $rest)
  }
  $opt
}

";

    rule().assert_violation_count_exact(bad_code, 1);
}

#[test]
fn test_untyped_both_input_output() {
    let bad_code = r"
def transform [] {
    $in | each { |x| $x + 1 }
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_exported_untyped_pipeline_input() {
    let bad_code = r"
export def process [] {
    $in | str trim
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_untyped_with_parameters() {
    let bad_code = r"
def multiply [factor: int] {
    $in * $factor
}
";
    rule().assert_detects(bad_code);
}

#[test]
fn test_multiple_untyped_commands() {
    let bad_code = r"
def first [] { $in | first }
def last [] { $in | last }
";
    rule().assert_violation_count_exact(bad_code, 2);
}
