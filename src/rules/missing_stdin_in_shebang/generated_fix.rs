use super::RULE;

#[test]
fn test_fix_basic_env_nu_shebang() {
    let source = r"#!/usr/bin/env nu

def main []: string -> string {
    $in | str upcase
}
";

    RULE.assert_replacement_contains(source, "--stdin");
    RULE.assert_replacement_contains(source, "#!/usr/bin/env -S nu --stdin");
}

#[test]
fn test_fix_env_with_s_flag() {
    let source = r"#!/usr/bin/env -S nu

def main [] {
    $in | str upcase
}
";

    RULE.assert_replacement_contains(source, "--stdin");
    RULE.assert_replacement_contains(source, "#!/usr/bin/env -S nu --stdin");
}

#[test]
fn test_fix_direct_nu_path() {
    let source = r"#!/usr/bin/nu

def main []: string -> string {
    $in | lines | length
}
";

    RULE.assert_replacement_contains(source, "#!/usr/bin/nu --stdin");
}

#[test]
fn test_fix_with_uses_in_variable() {
    let source = r"#!/usr/bin/env nu

def main [] {
    let input = $in
    print $input
}
";

    RULE.assert_replacement_contains(source, "#!/usr/bin/env -S nu --stdin");
}

#[test]
fn test_fix_with_pipeline_type_annotation() {
    let source = r"#!/usr/bin/env nu

def main []: list<string> -> string {
    lines | first
}
";

    RULE.assert_replacement_contains(source, "#!/usr/bin/env -S nu --stdin");
}

#[test]
fn test_no_fix_when_stdin_already_present() {
    let source = r"#!/usr/bin/env -S nu --stdin

def main []: string -> string {
    $in | str upcase
}
";

    RULE.assert_count(source, 0);
}

#[test]
fn test_fix_with_env_s_and_other_flags() {
    let source = r"#!/usr/bin/env -S nu --log-level debug

def main [] {
    $in | str trim
}
";

    RULE.assert_replacement_contains(source, "#!/usr/bin/env -S nu --stdin --log-level debug");
}

#[test]
fn test_fix_explanation() {
    let source = r"#!/usr/bin/env nu

def main [] {
    $in | str upcase
}
";

    RULE.assert_fix_explanation_contains(source, "Add --stdin flag");
}

#[test]
fn test_help_message() {
    let source = r"#!/usr/bin/env nu

def main []: string -> string {
    $in | str upcase
}
";

    RULE.assert_help_contains(source, "--stdin");
    RULE.assert_help_contains(source, "shebang");
}

#[test]
fn test_fix_produces_valid_code() {
    // Test that the fix only replaces the shebang line, not the entire file
    let source = r"#!/usr/bin/env nu

def main [] {
    let input = $in
    print $input
}
";

    // Verify the fix contains the corrected shebang
    RULE.assert_replacement_contains(source, "#!/usr/bin/env -S nu --stdin");

    // Verify there's exactly one violation
    RULE.assert_count(source, 1);
}

#[test]
fn test_fix_preserves_rest_of_file() {
    // Ensure the fix doesn't mangle the code after the shebang
    let source = "#!/usr/bin/env nu
def helper [] { \"test\" }
def main [] { $in | str upcase }
";

    RULE.assert_replacement_contains(source, "#!/usr/bin/env -S nu --stdin");
    RULE.assert_count(source, 1);
}
