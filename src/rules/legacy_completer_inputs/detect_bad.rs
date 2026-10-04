use super::RULE;

#[test]
fn detect_parameter_completer_with_context_and_offset() {
    let bad_code = r#"
def complete-branch [context: string, offset: int] {
    [main develop]
}

def checkout [branch: string@complete-branch] {
    print $branch
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_parameter_completer_with_only_context() {
    let bad_code = r#"
def complete-profile [context: string] {
    [debug release]
}

def build [profile: string@complete-profile] {
    print $profile
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_flag_completer() {
    let bad_code = r#"
def complete-profile [context: string, offset: int] {
    [debug release]
}

def build [--profile: string@complete-profile] {
    print $profile
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_new_input_mixed_with_legacy_input() {
    let bad_code = r#"
def complete-branch [token: record, offset: int] {
    [main]
}

def checkout [branch: string@complete-branch] {
    print $branch
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_command_wide_completer_with_spans() {
    let bad_code = r#"
def complete-git [spans: list<string>] {
    [status log]
}

@complete complete-git
def --wrapped git-wrapper [...args] {
    ^git ...$args
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_external_completer_closure_with_spans() {
    let bad_code = r#"
$env.config.completions.external.completer = {|spans|
    ^carapace $spans.0 nushell ...$spans | from json
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_external_completer_in_config_record() {
    let bad_code = r#"
$env.config.completions = {
    external: {
        enable: true
        completer: {|spans| ^carapace $spans.0 nushell ...$spans | from json }
    }
}
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn detect_external_completer_bound_to_variable() {
    let bad_code = r#"
let carapace_completer = {|spans|
    ^carapace $spans.0 nushell ...$spans | from json
}
$env.config.completions.external.completer = $carapace_completer
"#;
    RULE.assert_detects(bad_code);
}

#[test]
fn label_parameter_offset_with_place_cursor() {
    let bad_code = r#"
def complete-branch [context: string, offset: int] {
    [main]
}

def checkout [branch: string@complete-branch] { print $branch }
"#;
    RULE.assert_labels_contain(bad_code, "`$place.cursor`");
}

#[test]
fn label_external_spans_with_place_command() {
    let bad_code = r#"
$env.config.completions.external.completer = {|spans| $spans }
"#;
    RULE.assert_labels_contain(bad_code, "`$place.command`");
}

#[test]
fn detect_each_legacy_completer_once() {
    let bad_code = r#"
def complete-branch [context: string, offset: int] {
    [main]
}

def checkout [branch: string@complete-branch] { print $branch }
def merge [branch: string@complete-branch] { print $branch }
"#;
    RULE.assert_count(bad_code, 1);
}
