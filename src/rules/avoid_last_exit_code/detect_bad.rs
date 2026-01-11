use super::RULE;

#[test]
fn detects_basic_not_equal_check() {
    RULE.assert_detects(
        r#"
^git clone https://example.com/repo
if $env.LAST_EXIT_CODE != 0 {
    error make { msg: "clone failed" }
}
"#,
    );
}

#[test]
fn detects_equality_check() {
    RULE.assert_detects(
        r#"
^curl https://example.com
if $env.LAST_EXIT_CODE == 0 {
    print "success"
}
"#,
    );
}

#[test]
fn detects_in_function() {
    RULE.assert_detects(
        r#"
def clone-repo [] {
    ^git clone repo
    if $env.LAST_EXIT_CODE != 0 {
        return
    }
}
"#,
    );
}

#[test]
fn detects_multiple_instances() {
    RULE.assert_count(
        r#"
^git push
if $env.LAST_EXIT_CODE != 0 { return }
^git pull
if $env.LAST_EXIT_CODE != 0 { return }
"#,
        2,
    );
}

#[test]
fn detects_with_comparison_operators() {
    RULE.assert_detects(
        r#"
^git status
if $env.LAST_EXIT_CODE > 0 {
    print "failed"
}
"#,
    );
}

#[test]
fn detects_in_nested_closure() {
    RULE.assert_detects(
        r#"
def deploy [] {
    [1 2 3] | each {|x|
        ^git push
        if $env.LAST_EXIT_CODE != 0 {
            return
        }
    }
}
"#,
    );
}

#[test]
fn detects_with_sed() {
    RULE.assert_detects(
        r#"
^sed -i 's/foo/bar/' file.txt
if $env.LAST_EXIT_CODE != 0 {
    print "sed failed"
}
"#,
    );
}

#[test]
fn detects_with_find() {
    RULE.assert_detects(
        r#"
^find . -name "*.txt"
if $env.LAST_EXIT_CODE != 0 {
    error make { msg: "find failed" }
}
"#,
    );
}
