use super::RULE;

#[test]
fn ignores_in_var_inside_main() {
    RULE.assert_ignores(
        r#"
def main [] {
    print $in
}
"#,
    );
}

#[test]
fn ignores_in_var_inside_other_function_with_main() {
    RULE.assert_ignores(
        r#"
def helper [] {
    $in | lines
}

def main [] {
    "test" | helper
}
"#,
    );
}

#[test]
fn ignores_script_without_in_var() {
    RULE.assert_ignores(
        r#"
print "hello"
let x = 5
print ($x * 2)
"#,
    );
}

#[test]
fn ignores_function_without_main() {
    RULE.assert_ignores(
        r#"
def helper [x: int] {
    $x * 2
}

print (helper 5)
"#,
    );
}

#[test]
fn ignores_in_var_in_closure() {
    RULE.assert_ignores(
        r#"
def main [] {
    [1 2 3] | each { |x| $in }
}
"#,
    );
}

#[test]
fn ignores_in_var_in_closure_assignment() {
    RULE.assert_ignores(
        r#"
$env.config.color_config.string = {
  if $in =~ '^#[a-fA-F\d]{6}' {
    $in
  } else {
    'default'
  }
}
"#,
    );
}

#[test]
fn ignores_in_var_in_nested_closure() {
    RULE.assert_ignores(
        r#"
let formatter = {|| $in | str upcase }
"#,
    );
}
