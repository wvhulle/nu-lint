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
fn test_variable_assigned_without_parens_not_flagged() {
    let good_code = r"
def process [] {
  let result = $input | transform
  $result
}
";

    rule().assert_ignores(good_code);
}
