use super::rule;

#[test]
fn main_without_external_calls() {
    rule().assert_ignores(
        r"
def main [config_path: string] {
  let config = (open $config_path)
  print $config
}
",
    );
}

#[test]
fn main_with_external_call_but_hardcoded_command() {
    rule().assert_ignores(
        r"
def main [] {
  let result = (^powerprofilesctl get | complete)
  print $result
}
",
    );
}

#[test]
fn helper_function_with_script_parameter() {
    rule().assert_ignores(
        r"
def run-script [script_path: string] {
  ^$script_path
}
",
    );
}

#[test]
fn main_with_string_param_not_used_as_external() {
    rule().assert_ignores(
        r#"
def main [message: string] {
  print $message
  echo "Done"
}
"#,
    );
}

#[test]
fn main_with_number_parameter() {
    rule().assert_ignores(
        r"
def main [count: int] {
  for i in 1..$count {
    print $i
  }
}
",
    );
}

#[test]
fn no_main_function() {
    rule().assert_ignores(
        r#"
def helper [script: string] {
  ^$script
}

print "Hello"
"#,
    );
}

#[test]
fn main_calls_defined_function() {
    rule().assert_ignores(
        r#"
def handle-profile [profile: string] {
  print $"Handling profile: ($profile)"
}

def main [] {
  let profile = "balanced"
  handle-profile $profile
}
"#,
    );
}

#[test]
fn main_with_external_call_different_variable() {
    rule().assert_ignores(
        r#"
def main [config: string] {
  let cmd = "ls"
  ^$cmd
  print $config
}
"#,
    );
}
