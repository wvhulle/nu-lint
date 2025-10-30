use super::rule;

#[test]
fn test_variable_used_multiple_times_not_flagged() {
    let good_code = r"
def foo [] {
  let result = (some | pipeline)
  print $result
  $result
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_direct_return_not_flagged() {
    let good_code = r"
def foo [] {
  some | pipeline
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_variable_with_additional_logic_not_flagged() {
    let good_code = r"
def process [] {
  let data = (load | some | data)
  if ($data | is-empty) {
    error make { msg: 'No data' }
  }
  $data
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_const_variable_not_flagged() {
    let good_code = r"
def process [] {
  const MAX = 100
  $MAX
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_variable_modified_before_return() {
    let good_code = r"
def process [] {
  let result = (get | data)
  $result | transform
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_different_variable_returned() {
    let good_code = r"
def process [] {
  let x = (get | data)
  $y
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_variable_with_field_access() {
    let good_code = r"
def process [] {
  let data = (get | record)
  $data.field
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_mut_variable_not_flagged() {
    let good_code = r"
def process [] {
  mut result = (get | data)
  $result
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_variable_in_different_scope() {
    let good_code = r"
def outer [] {
  let x = (1 + 2)
  if true {
    $x
  }
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_multiple_statements_between() {
    let good_code = r"
def process [] {
  let data = (get | input)
  print 'Processing...'
  do-something
  $data
}
";

    rule().assert_ignores(good_code);
}

#[test]
fn test_variable_used_in_expression() {
    let good_code = r"
def process [] {
  let x = (10)
  $x + 5
}
";

    rule().assert_ignores(good_code);
}
