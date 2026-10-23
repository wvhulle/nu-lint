use super::RULE;

#[test]
fn single_unused_helper() {
    RULE.assert_detects(
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
    RULE.assert_count(
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
fn transitive_unused_helper() {
    RULE.assert_detects(
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
    RULE.assert_count(
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

#[test]
fn unused_helper_with_subcommands() {
    RULE.assert_detects(
        r#"
def "main build" [] {
  print "building"
}

def "main test" [] {
  print "testing"
}

def unused-helper [] {
  print "never called"
}
"#,
    );
}

#[test]
fn recursive_unused_helper() {
    RULE.assert_detects(
        r#"
def main [] {
  print "main"
}

def recursive-unused [n: int] {
  if $n > 0 {
    recursive-unused ($n - 1)
  }
}
"#,
    );
}

#[test]
fn mutually_recursive_unused_helpers() {
    RULE.assert_count(
        r#"
def main [] {
  print "main"
}

def ping [n: int] {
  if $n > 0 { pong ($n - 1) }
}

def pong [n: int] {
  if $n > 0 { ping ($n - 1) }
}
"#,
        2,
    );
}

#[test]
fn unused_helper_with_exports() {
    RULE.assert_detects(
        r#"
export def public_api [] {
  helper_for_api
}

def helper_for_api [] {
  print "used by export"
}

def truly_unused [] {
  print "not used by anyone"
}
"#,
    );
}
