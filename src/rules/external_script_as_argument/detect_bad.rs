use super::rule;

#[test]
fn test_detect_main_with_script_path_parameter() {
    rule().assert_detects(
        r#"
def main [handler_path: string] {
  let result = (^$handler_path "arg1" | complete)
  print $result
}
"#,
    );
}

#[test]
fn test_detect_main_with_any_type_used_as_command() {
    rule().assert_detects(
        r"
def main [cmd] {
  let output = (^$cmd --version | complete)
  print $output
}
",
    );
}

#[test]
fn test_detect_main_with_multiple_params_external_usage() {
    rule().assert_detects(
        r#"
def main [
  handler_path: string
  profile: string
] {
  print $"<6>Profile changed to '($profile)'"
  
  try {
    let result = (^$handler_path $profile | complete)
    if $result.exit_code != 0 {
      print $"<3>Error: Handler exited with code ($result.exit_code)"
    }
  } catch {|err|
    print $"<3>Error calling handler: ($err.msg)"
  }
}
"#,
    );
}

#[test]
fn test_detect_main_with_optional_script_parameter() {
    rule().assert_detects(
        r"
def main [handler?: string] {
  if ($handler != null) {
    ^$handler
  }
}
",
    );
}

#[test]
fn test_detect_helper_function_with_script_parameter() {
    rule().assert_detects(
        r"
def run-script [script_path: string] {
  ^$script_path
}
",
    );
}

#[test]
fn test_detect_multiple_custom_commands_with_script_params() {
    rule().assert_detects(
        r#"
def run-backup [backup_script: string] {
  ^$backup_script --verbose
}

def cleanup [cleanup_tool: path] {
  ^$cleanup_tool /tmp
}

def main [] {
  print "Running tasks..."
}
"#,
    );
}
