use super::rule;

#[test]
fn no_main_function() {
    rule().assert_ignores(
        r#"
def helper1 [] {
  print "no main"
}

def helper2 [] {
  print "also no main"
}
"#,
    );
}

#[test]
fn all_helpers_used() {
    rule().assert_ignores(
        r#"
def main [] {
  helper1
  helper2
}

def helper1 [] {
  print "used"
}

def helper2 [] {
  print "also used"
}
"#,
    );
}

#[test]
fn transitive_call_chain() {
    rule().assert_ignores(
        r#"
def main [] {
  helper1
}

def helper1 [] {
  helper2
}

def helper2 [] {
  print "transitively used"
}
"#,
    );
}

#[test]
fn transitive_call_chain_spaces() {
    rule().assert_ignores(
        r#"
def helper2 [] {
  print "transitively used"
}

def "helper foo" [] {
  helper2
}


def main [] {
  helper foo
}



"#,
    );
}

#[test]
fn helper_called_from_helper() {
    rule().assert_ignores(
        r"
def main [] {
  do-work
}

def do-work [] {
  let result = calculate
  print $result
}

def calculate [] {
  42
}
",
    );
}

#[test]
fn only_main_no_helpers() {
    rule().assert_ignores(
        r#"
def main [] {
  print "hello world"
}
"#,
    );
}

#[test]
fn script_without_functions() {
    rule().assert_ignores(
        r#"
print "just a script"
let x = 42
"#,
    );
}

#[test]
fn main_calls_builtin_commands() {
    rule().assert_ignores(
        r"
def main [] {
  ls | where type == file
}
",
    );
}

#[test]
fn complex_call_chain() {
    rule().assert_ignores(
        r#"
def main [] {
  process-data
}

def process-data [] {
  let data = load-data
  validate-data $data
  save-data $data
}

def load-data [] {
  {a: 1, b: 2}
}

def validate-data [data] {
  if ($data | is-empty) {
    error make {msg: "empty data"}
  }
}

def save-data [data] {
  $data | to json
}
"#,
    );
}
