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

// Tests for call site fixing

#[test]
fn fix_call_site_list_literal() {
    RULE.assert_fixed_is(
        r#"def process [items: list<string>] { $items }
process ["a" "b" "c"]"#,
        r#"def process [...items: string] { $items }
process "a" "b" "c""#,
    );
}

#[test]
fn fix_call_site_variable() {
    RULE.assert_fixed_is(
        r#"def process [items: list<string>] { $items }
let files = ["a" "b"]
process $files"#,
        r#"def process [...items: string] { $items }
let files = ["a" "b"]
process ...$files"#,
    );
}

#[test]
fn fix_call_site_expression() {
    RULE.assert_fixed_is(
        r#"def process [items: list<string>] { $items }
process (["a" "b"])"#,
        r#"def process [...items: string] { $items }
process ...(["a" "b"])"#,
    );
}

#[test]
fn fix_multiple_call_sites() {
    RULE.assert_fixed_is(
        r#"def process [items: list<string>] { $items }
process ["a" "b"]
process ["x" "y" "z"]"#,
        r#"def process [...items: string] { $items }
process "a" "b"
process "x" "y" "z""#,
    );
}

#[test]
fn fix_preserves_other_positional_args() {
    RULE.assert_fixed_is(
        r#"def cmd [name: string, items: list<string>] { $items }
cmd "test" ["a" "b"]"#,
        r#"def cmd [name: string, ...items: string] { $items }
cmd "test" "a" "b""#,
    );
}

#[test]
fn fix_empty_list_call_site() {
    RULE.assert_fixed_is(
        r#"def process [items: list<string>] { $items }
process []"#,
        r#"def process [...items: string] { $items }
process "#,
    );
}
