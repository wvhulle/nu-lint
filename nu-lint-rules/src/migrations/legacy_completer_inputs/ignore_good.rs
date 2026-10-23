use super::RULE;

#[test]
fn ignore_parameter_completer_with_token() {
    let good_code = r#"
def complete-branch [token: record] {
    [main develop] | where $it starts-with $token.text
}

def checkout [branch: string@complete-branch] {
    print $branch
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_parameter_completer_with_buffer_and_place() {
    let good_code = r#"
def complete-profile [buffer: string, place: record] {
    [debug release]
}

def build [--profile: string@complete-profile] {
    print $profile
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_completer_without_inputs() {
    let good_code = r#"
def complete-profile [] {
    [debug release]
}

def build [profile: string@complete-profile] {
    print $profile
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_list_completion() {
    let good_code = r#"
def build [profile: string@[debug release]] {
    print $profile
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_command_not_used_as_completer() {
    let good_code = r#"
def find-at [context: string, offset: int] {
    $context | str substring $offset..
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_command_wide_completer_with_place() {
    let good_code = r#"
def complete-git [place: record] {
    [status log]
}

@complete complete-git
def --wrapped git-wrapper [...args] {
    ^git ...$args
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_external_completer_closure_with_place() {
    let good_code = r#"
$env.config.completions.external.completer = {|place: record|
    ^carapace ($place.command | first) nushell ...$place.command | from json
}
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_unrelated_closure_with_spans_parameter() {
    let good_code = r#"
let render = {|spans| $spans | str join ' ' }
$env.config.completions.external.enable = true
"#;
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_other_config_closures() {
    let good_code = r#"
$env.config.hooks.display_output = {|value| $value | table }
"#;
    RULE.assert_ignores(good_code);
}
