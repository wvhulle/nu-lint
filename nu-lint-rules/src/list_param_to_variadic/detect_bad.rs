use super::RULE;

#[test]
fn detect_single_list_param() {
    RULE.assert_detects(
        r#"
def process [items: list<string>] {
    $items | each { print $in }
}
"#,
    );
}

#[test]
fn detect_list_as_last_required() {
    RULE.assert_detects(
        r#"
def cmd [name: string, files: list<path>] {
    $files | each { open $in }
}
"#,
    );
}

#[test]
fn detect_list_as_last_optional() {
    RULE.assert_detects(
        r#"
def cmd [name: string, files?: list<string>] {
    $files | default [] | each { print $in }
}
"#,
    );
}

#[test]
fn detect_untyped_list() {
    RULE.assert_detects(
        r#"
def process [items: list] {
    $items | length
}
"#,
    );
}

#[test]
fn detect_list_with_flags() {
    RULE.assert_detects(
        r#"
def cmd [items: list<string>, --verbose] {
    if $verbose { print "verbose" }
    $items | each { print $in }
}
"#,
    );
}
