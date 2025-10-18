use super::rule;
use crate::LintContext;

#[test]
fn test_ignore_where_usage() {
    let good_code = r"
ls | where size > 10kb
";

    LintContext::test_with_parsed_source(good_code, |context| {
        let violations = (rule().check)(&context);
        assert!(violations.is_empty(), "Should not flag proper where usage");
    });
}

#[test]
fn test_ignore_each_with_side_effects() {
    let good_code = r"
ls | each { |f| if $f.size > 100kb { print $f.name } }
";

    LintContext::test_with_parsed_source(good_code, |context| {
        let violations = (rule().check)(&context);
        assert!(
            violations.is_empty(),
            "Should not flag each with side effects like print"
        );
    });
}

#[test]
fn test_ignore_each_without_if() {
    let good_code = r"
seq 1 10 | each { |x| $x * 2 }
";

    LintContext::test_with_parsed_source(good_code, |context| {
        let violations = (rule().check)(&context);
        assert!(violations.is_empty(), "Should not flag each without if");
    });
}

#[test]
fn test_ignore_each_if_with_mutation() {
    let good_code = r"
ls | each { |f| if $f.size > 100kb { mut name = $f.name; $name } }
";

    LintContext::test_with_parsed_source(good_code, |context| {
        let violations = (rule().check)(&context);
        assert!(violations.is_empty(), "Should not flag each with mutations");
    });
}
