use super::RULE;

#[test]
fn five_levels_of_nesting() {
    RULE.assert_detects(
        r#"
def deeply-nested [] {
  if true {
    if true {
      if true {
        if true {
          if true {
            print "too deep"
          }
        }
      }
    }
  }
}
"#,
    );
}

#[test]
fn nested_loops_and_conditions() {
    RULE.assert_detects(
        r#"
def process-data [] {
  for item in $items {
    if $item.active {
      for sub in $item.children {
        if $sub.valid {
          match $sub.type {
            "a" => { print "deeply nested" }
          }
        }
      }
    }
  }
}
"#,
    );
}

#[test]
fn nested_try_blocks() {
    RULE.assert_detects(
        r#"
def handle-errors [] {
  try {
    if true {
      try {
        if true {
          try {
            print "too nested"
          }
        }
      }
    }
  }
}
"#,
    );
}

#[test]
fn mixed_control_structures() {
    RULE.assert_detects(
        r#"
def complex-function [] {
  while true {
    if $condition {
      for x in $list {
        match $x {
          1 => {
            if $nested {
              print "excessive nesting"
            }
          }
        }
      }
    }
  }
}
"#,
    );
}
