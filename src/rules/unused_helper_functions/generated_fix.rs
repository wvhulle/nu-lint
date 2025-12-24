use super::RULE;

#[test]
fn fix_removes_unused_function() {
    let bad_code = r#"
def main [] {
  print "hello"
}

def helper [] {
  print "unused"
}
"#;
    RULE.assert_fix_explanation_contains(bad_code, "Remove unused function");
}

#[test]
fn fix_erases_function_definition() {
    let bad_code = r#"
def main [] {
  print "hello"
}

def unused [] {
  print "never called"
}
"#;
    RULE.assert_replacement_erases(bad_code, "def unused");
}

#[test]
fn fix_erases_function_body() {
    let bad_code = r#"
def main [] {
  print "hello"
}

def unused [] {
  print "never called"
}
"#;
    RULE.assert_replacement_erases(bad_code, "print \"never called\"");
}

#[test]
fn fix_erases_multiline_function_body() {
    let bad_code = r#"
def main [] {
  used-helper
}

def used-helper [] {
  print "I am used"
}

def unused-helper [] {
  let x = 1
  let y = 2
  print $"Result: ($x + $y)"
}
"#;
    RULE.assert_replacement_erases(bad_code, "def unused-helper");
    RULE.assert_replacement_erases(bad_code, "let x = 1");
    RULE.assert_replacement_erases(bad_code, "let y = 2");
}

#[test]
fn fix_erases_function_with_parameters() {
    let bad_code = r#"
def main [] {
  print "main"
}

def unused-with-params [name: string, count: int] {
  for i in 0..$count {
    print $"Hello ($name)"
  }
}
"#;
    RULE.assert_replacement_erases(bad_code, "def unused-with-params");
    RULE.assert_replacement_erases(bad_code, "name: string");
    RULE.assert_replacement_erases(bad_code, "for i in");
}

#[test]
fn fix_erases_exported_unused_function() {
    let bad_code = r#"
def main [] {
  print "main"
}

export def unused-exported [] {
  print "exported but unused"
}
"#;
    RULE.assert_replacement_erases(bad_code, "export def unused-exported");
    RULE.assert_replacement_erases(bad_code, "exported but unused");
}
