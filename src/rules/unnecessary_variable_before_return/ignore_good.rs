use super::RULE;

#[test]
fn test_variable_used_multiple_times_not_flagged() {
    let good_code = r"
def foo [] {
  let result = (some | pipeline)
  print $result
  $result
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_direct_return_not_flagged() {
    let good_code = r"
def foo [] {
  some | pipeline
}
";

    RULE.assert_ignores(good_code);
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

    RULE.assert_ignores(good_code);
}

#[test]
fn test_const_variable_not_flagged() {
    let good_code = r"
def process [] {
  const MAX = 100
  $MAX
}
";

    RULE.assert_ignores(good_code);
}

#[test]
fn test_mut_variable_not_flagged() {
    let good_code = r"
def process [] {
  mut result = (get | data)
  $result
}
";

    RULE.assert_ignores(good_code);
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

    RULE.assert_ignores(good_code);
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

    RULE.assert_ignores(good_code);
}

#[test]
fn test_variable_used_in_expression() {
    let good_code = r"
def process [] {
  let x = (10)
  $x + 5
}
";

    RULE.assert_ignores(good_code);
}
