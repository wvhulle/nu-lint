use super::RULE;
use crate::log::init_env_log;

#[test]
fn detect_missing_input_type_annotation() {
    let bad_code = r"
def double [] {
    $in * 2
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_missing_output_type_annotation() {
    let bad_code = r"
def create-list [] {
    [1, 2, 3]
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_complex_function_missing_pipeline_types() {
    init_env_log();

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

    RULE.assert_count(bad_code, 1);
}

#[test]
fn detect_missing_input_and_output_types() {
    let bad_code = r"
def transform [] {
    $in | each { |x| $x + 1 }
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_exported_function_missing_input_type() {
    let bad_code = r"
export def process [] {
    $in | str trim
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_missing_pipeline_type_with_params() {
    let bad_code = r"
def multiply [factor: int] {
    $in * $factor
}
";
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_multiple_functions_missing_pipeline_types() {
    let bad_code = r"
def first [] { $in | first }
def last [] { $in | last }
";
    RULE.assert_count(bad_code, 2);
}
