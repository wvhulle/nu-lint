use super::RULE;

#[test]
fn main_without_external_calls() {
    RULE.assert_ignores(
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
    RULE.assert_ignores(
        r"
def main [] {
  let result = (^powerprofilesctl get | complete)
  print $result
}
",
    );
}

#[test]
fn main_with_string_param_not_used_as_external() {
    RULE.assert_ignores(
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
    RULE.assert_ignores(
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
fn main_calls_defined_function() {
    RULE.assert_ignores(
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
    RULE.assert_ignores(
        r#"
def main [config: string] {
  let cmd = "ls"
  ^$cmd
  print $config
}
"#,
    );
}
