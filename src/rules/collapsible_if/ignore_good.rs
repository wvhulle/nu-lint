use super::rule;

#[test]
fn test_if_with_else() {
    let good_code = r#"
if $x > 0 {
    if $y > 0 {
        print "both positive"
    } else {
        print "y not positive"
    }
}
"#;
    
    rule().assert_ignores(good_code);
}

#[test]
fn test_outer_if_with_else() {
    let good_code = r#"
if $x > 0 {
    if $y > 0 {
        print "both positive"
    }
} else {
    print "x not positive"
}
"#;
    
    rule().assert_ignores(good_code);
}

#[test]
fn test_single_if() {
    let good_code = r#"
if $x > 0 {
    print "positive"
}
"#;
    
    rule().assert_ignores(good_code);
}

#[test]
fn test_if_with_multiple_statements() {
    let good_code = r#"
if $x > 0 {
    print "x is positive"
    if $y > 0 {
        print "y is also positive"
    }
}
"#;
    
    rule().assert_ignores(good_code);
}

#[test]
fn test_separate_if_statements() {
    let good_code = r#"
if $x > 0 {
    print "x positive"
}
if $y > 0 {
    print "y positive"
}
"#;
    
    rule().assert_ignores(good_code);
}

#[test]
fn test_already_combined_condition() {
    let good_code = r#"
if $x > 0 and $y > 0 {
    print "both positive"
}
"#;
    
    rule().assert_ignores(good_code);
}

#[test]
fn test_nested_if_with_else_if() {
    let good_code = r#"
if $x > 0 {
    if $y > 0 {
        print "both positive"
    } else if $y < 0 {
        print "y negative"
    }
}
"#;
    
    rule().assert_ignores(good_code);
}
