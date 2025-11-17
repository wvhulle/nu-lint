use super::rule;

#[test]
fn exit_in_helper_function() {
    rule().assert_detects(
        "
def helper [] {
  exit 0
}
",
    );
}

#[test]
fn exit_in_check_function() {
    rule().assert_detects(
        "
def check-tools [] {
  if (some-condition) {
    exit 1
  }
}
",
    );
}

#[test]
fn exit_in_exported_function() {
    rule().assert_detects(
        r#"
export def my-command [] {
  print "error"
  exit 1
}
"#,
    );
}

#[test]
fn multiple_functions_with_exit() {
    rule().assert_count(
        "
def helper1 [] {
  exit 0
}

def helper2 [] {
  exit 1
}
",
        2,
    );
}
