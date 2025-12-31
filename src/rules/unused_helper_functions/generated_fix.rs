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
    RULE.assert_fix_erases(bad_code, "def unused");
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
    RULE.assert_fix_erases(bad_code, "print \"never called\"");
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
    RULE.assert_fix_erases(bad_code, "def unused-helper");
    RULE.assert_fix_erases(bad_code, "let x = 1");
    RULE.assert_fix_erases(bad_code, "let y = 2");
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
    RULE.assert_fix_erases(bad_code, "def unused-with-params");
    RULE.assert_fix_erases(bad_code, "name: string");
    RULE.assert_fix_erases(bad_code, "for i in");
}

#[test]
fn fix_erases_non_exported_unused_function() {
    // Exported functions are NOT flagged as unused (they may be used by external
    // importers) This test verifies only non-exported functions are removed
    let bad_code = r#"
def main [] {
  print "main"
}

def unused-helper [] {
  print "not exported, not called"
}

export def exported-helper [] {
  print "exported, should be kept"
}
"#;
    RULE.assert_fix_erases(bad_code, "def unused-helper");
    RULE.assert_fix_erases(bad_code, "not exported, not called");
}
