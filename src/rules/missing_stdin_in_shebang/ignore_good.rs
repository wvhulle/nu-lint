use super::RULE;
use crate::log::init_env_log;

#[test]
fn ignore_main_with_stdin_flag_standard() {
    let source = r"#!/usr/bin/env -S nu --stdin

def main []: string -> string {
    $in | str upcase
}
";
    RULE.assert_count(source, 0);
}

#[test]
fn ignore_main_with_stdin_flag_direct() {
    let source = r"#!/usr/bin/env nu --stdin

def main []: string -> string {
    $in | str upcase
}
";
    RULE.assert_count(source, 0);
}

#[test]
fn ignore_main_without_pipeline_input() {
    let source = r#"#!/usr/bin/env nu

def main [] {
    print "Hello"
}
"#;
    RULE.assert_count(source, 0);
}

#[test]
fn ignore_main_with_nothing_input_type() {
    let source = r#"#!/usr/bin/env nu

def main []: nothing -> string {
    "Hello"
}
"#;
    RULE.assert_count(source, 0);
}

#[test]
fn ignore_main_with_positional_args_only() {
    let source = r#"#!/usr/bin/env nu

def main [name: string] {
    print $"Hello ($name)"
}
"#;
    RULE.assert_count(source, 0);
}

#[test]
fn ignore_helper_function_uses_in() {
    let source = r#"#!/usr/bin/env nu

def helper [] {
    let data = $in
    print $data
}

def main [] {
    "test" | helper
}
"#;
    RULE.assert_count(source, 0);
}

#[test]
fn ignore_script_without_shebang() {
    let source = r"
def main []: string -> string {
    $in | str upcase
}
";
    RULE.assert_count(source, 0);
}

#[test]
fn ignore_main_with_stdin_and_uses_in() {
    let source = r"#!/usr/bin/env -S nu --stdin

def main [] {
    let data = $in
    print $data
}
";
    RULE.assert_count(source, 0);
}

#[test]
fn ignore_main_with_nothing_input_and_nothing_output_and_flags() {
    init_env_log();
    let source = r#"#!/usr/bin/env nu

def main [
  --mcp-config: string # JSON string containing MCP configuration
]: nothing -> nothing {
  let json_path = $env.HOME | path join '.claude.json'

  ensure-config-exists $json_path

  let new_servers = $mcp_config | from json | get mcpServers
  let existing_config = open $json_path
  let existing_servers = $existing_config | get -i mcpServers | default {}

  if (has-changes $new_servers $existing_servers) {
    update-and-report $json_path $existing_config $existing_servers $new_servers
  } else {
    print $"<notice>MCP servers already up to date in ($json_path)"
  }
}
"#;
    RULE.assert_count(source, 0);
}
