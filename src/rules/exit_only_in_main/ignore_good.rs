use super::rule;

#[test]
fn exit_in_main_is_allowed() {
    rule().assert_ignores(
        "
def main [] {
  exit 0
}
",
    );
}

#[test]
fn exit_at_script_level() {
    rule().assert_ignores(
        r#"
print "done"
exit 0
"#,
    );
}

#[test]
fn return_in_function() {
    rule().assert_ignores(
        "
def helper [] {
  return
}
",
    );
}

#[test]
fn error_make_in_function() {
    rule().assert_ignores(
        r#"
def helper [] {
  error make { msg: "error" }
}
"#,
    );
}

#[test]
fn function_without_exit() {
    rule().assert_ignores(
        r#"
def helper [] {
  print "hello"
  42
}
"#,
    );
}

#[test]
fn main_with_other_commands() {
    rule().assert_ignores(
        "
def main [] {
  let result = run-checks
  if $result != 0 {
    exit $result
  }
}
",
    );
}
