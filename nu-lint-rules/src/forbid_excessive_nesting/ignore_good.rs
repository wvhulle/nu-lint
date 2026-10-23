use super::RULE;

#[test]
fn no_nesting() {
    RULE.assert_ignores(
        r#"
def simple [] {
  print "hello"
  42
}
"#,
    );
}

#[test]
fn single_level_nesting() {
    RULE.assert_ignores(
        r#"
def with-if [] {
  if true {
    print "ok"
  }
}
"#,
    );
}

#[test]
fn two_levels_nesting() {
    RULE.assert_ignores(
        r"
def nested-twice [] {
  if true {
    for item in $items {
      print $item
    }
  }
}
",
    );
}

#[test]
fn three_levels_nesting() {
    RULE.assert_ignores(
        r"
def nested-three-times [] {
  if true {
    for item in $items {
      if $item.valid {
        print $item
      }
    }
  }
}
",
    );
}

#[test]
fn four_levels_is_max_allowed() {
    RULE.assert_ignores(
        r#"
def at-max-nesting [] {
  if true {
    for item in $items {
      if $item.valid {
        print "exactly 4 levels"
      }
    }
  }
}
"#,
    );
}

#[test]
fn empty_lines_ignored() {
    RULE.assert_ignores(
        r#"
def with-empty-lines [] {
  if true {

    print "empty lines don't count"

  }
}
"#,
    );
}

#[test]
fn script_level_code() {
    RULE.assert_ignores(
        r#"
print "hello"
let x = 42
print $x
"#,
    );
}
