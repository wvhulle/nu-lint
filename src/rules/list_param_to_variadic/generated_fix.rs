use super::RULE;

#[test]
fn fix_single_list_param() {
    RULE.assert_fixed_contains(
        r#"
def process [items: list<string>] {
    $items | each { print $in }
}
"#,
        "[...items: string]",
    );
}

#[test]
fn fix_list_as_last_required() {
    RULE.assert_fixed_contains(
        r#"
def cmd [name: string, files: list<path>] {
    $files | each { open $in }
}
"#,
        "[name: string, ...files: path]",
    );
}

#[test]
fn fix_list_as_last_optional() {
    RULE.assert_fixed_contains(
        r#"
def cmd [name: string, files?: list<string>] {
    $files | default [] | each { print $in }
}
"#,
        "[name: string, ...files: string]",
    );
}

#[test]
fn fix_untyped_list() {
    RULE.assert_fixed_contains(
        r#"
def process [items: list] {
    $items | length
}
"#,
        "[...items]",
    );
}

#[test]
fn fix_preserves_flags() {
    RULE.assert_fixed_contains(
        r#"
def cmd [items: list<string>, --verbose] {
    if $verbose { print "verbose" }
    $items | each { print $in }
}
"#,
        "[...items: string, --verbose]",
    );
}

#[test]
fn fix_preserves_flag_with_type() {
    RULE.assert_fixed_contains(
        r#"
def cmd [items: list<int>, --count: int] {
    $items | take $count
}
"#,
        "[...items: int, --count: int]",
    );
}
