use super::rule;

#[test]
fn single_unused_helper() {
    rule().assert_detects(
        r#"
def main [] {
  print "hello"
}

def helper [] {
  print "unused"
}
"#,
    );
}

#[test]
fn multiple_unused_helpers() {
    rule().assert_violation_count(
        r#"
def main [] {
  print "hello"
}

def helper1 [] {
  print "unused1"
}

def helper2 [] {
  print "unused2"
}
"#,
        2,
    );
}

#[test]
fn unused_exported_function() {
    rule().assert_detects(
        r#"
def main [] {
  print "hello"
}

export def unused-helper [] {
  print "not used"
}
"#,
    );
}

#[test]
fn transitive_unused_helper() {
    rule().assert_detects(
        r#"
def main [] {
  helper1
}

def helper1 [] {
  print "used"
}

def helper2 [] {
  print "not called"
}
"#,
    );
}

#[test]
fn unused_chain() {
    rule().assert_violation_count(
        r#"
def main [] {
  print "main"
}

def helper1 [] {
  helper2
}

def helper2 [] {
  print "nested"
}
"#,
        2,
    );
}
