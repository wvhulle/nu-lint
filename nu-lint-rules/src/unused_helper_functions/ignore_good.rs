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

#[test]
fn exported_function_not_called_from_main() {
    // Exported functions are part of the module's public API and may be used
    // externally
    RULE.assert_ignores(
        r#"
def main [] {
  print "main"
}

export def helper [] {
  print "exported helper"
}
"#,
    );
}

#[test]
fn completer_function_not_called_from_main() {
    // Functions used as argument completers are referenced via @completer
    // annotation
    RULE.assert_ignores(
        r#"
def main [arg: string@my-completer] {
  print $arg
}

def my-completer [] {
  ["option1", "option2"]
}
"#,
    );
}

#[test]
fn completer_for_flag() {
    // Completer for a flag parameter
    RULE.assert_ignores(
        r#"
def main [--format: string@format-completer] {
  print $format
}

def format-completer [] {
  ["json", "yaml", "toml"]
}
"#,
    );
}

#[test]
fn helper_used_by_exported_function() {
    RULE.assert_ignores(
        r#"
export def public_api [] {
  helper
}

def helper [] {
  print "used by exported function"
}
"#,
    );
}

#[test]
fn transitive_chain_from_exported_function() {
    RULE.assert_ignores(
        r#"
export def public_api [] {
  helper_a
}

def helper_a [] {
  helper_b
}

def helper_b [] {
  print "transitively used by export"
}
"#,
    );
}

#[test]
fn multiple_exports_sharing_helper() {
    RULE.assert_ignores(
        r#"
export def export_a [] {
  shared_helper
}

export def export_b [] {
  shared_helper
}

def shared_helper [] {
  print "shared by multiple exports"
}
"#,
    );
}

#[test]
fn module_without_main_only_exports() {
    RULE.assert_ignores(
        r#"
export def api_one [] {
  helper_one
}

export def api_two [] {
  helper_two
}

def helper_one [] {
  print "used by api_one"
}

def helper_two [] {
  print "used by api_two"
}
"#,
    );
}

#[test]
fn mixed_entry_points_main_and_export() {
    RULE.assert_ignores(
        r#"
def main [] {
  print "main entry"
}

export def exported_api [] {
  helper_for_export
}

def helper_for_export [] {
  print "used only by exported function"
}
"#,
    );
}

#[test]
fn export_with_completer_calling_helper() {
    RULE.assert_ignores(
        r#"
export def cmd [arg: string@my_completer] {
  print $arg
}

def my_completer [] {
  helper_for_completion
}

def helper_for_completion [] {
  ["option1", "option2", "option3"]
}
"#,
    );
}

#[test]
fn main_and_export_both_calling_same_helper() {
    RULE.assert_ignores(
        r#"
def main [] {
  shared_helper
}

export def exported_func [] {
  shared_helper
}

def shared_helper [] {
  print "shared by both main and export"
}
"#,
    );
}

#[test]
fn deeply_nested_call_chain_from_export() {
    RULE.assert_ignores(
        r#"
export def level1 [] {
  level2
}

def level2 [] {
  level3
}

def level3 [] {
  level4
}

def level4 [] {
  print "deeply nested"
}
"#,
    );
}
