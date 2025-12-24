use super::RULE;

#[test]
fn no_main_function() {
    RULE.assert_ignores(
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
fn transitive_call_chain() {
    RULE.assert_ignores(
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
fn helper_called_from_helper() {
    RULE.assert_ignores(
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
fn script_without_functions() {
    RULE.assert_ignores(
        r#"
print "just a script"
let x = 42
"#,
    );
}

#[test]
fn complex_call_chain() {
    RULE.assert_ignores(
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

#[test]
fn helper_called_from_main_subcommand() {
    RULE.assert_ignores(
        r#"
def "main subcommand" [] {
  helper
}

def helper [] {
  print "used by subcommand"
}
"#,
    );
}

#[test]
fn helper_called_from_multiple_main_subcommands() {
    RULE.assert_ignores(
        r#"
def main [] {
  print "main entry"
}

def "main build" [] {
  helper1
}

def "main test" [] {
  helper2
}

def helper1 [] {
  print "used by build"
}

def helper2 [] {
  print "used by test"
}
"#,
    );
}

#[test]
fn transitive_call_from_subcommand() {
    RULE.assert_ignores(
        r#"
def "main run" [] {
  process
}

def process [] {
  validate
}

def validate [] {
  print "transitively used from subcommand"
}
"#,
    );
}

#[test]
fn recursive_main() {
    RULE.assert_ignores(
        r#"
def main [n: int] {
  if $n > 0 {
    main ($n - 1)
  }
}
"#,
    );
}
