#[cfg(test)]
mod tests {
    use crate::{
        context::LintContext, rule::RegexRule,
        rules::prefer_where_over_each_if::PreferWhereOverEachIf,
    };

    #[test]
    fn test_detect_each_if_simple_filter() {
        let rule = PreferWhereOverEachIf;

        let bad_code = r"
ls | each { |f| if $f.size > 100kb { $f } }
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect each with if for simple filtering"
            );
        });
    }

    #[test]
    fn test_detect_each_if_complex_condition() {
        let rule = PreferWhereOverEachIf;

        let bad_code = r"
open data.json | get items | each { |item| if ($item.status == 'active' and $item.count > 0) { $item } }
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect each with if for complex filtering"
            );
        });
    }

    #[test]
    fn test_detect_each_if_with_property_access() {
        let rule = PreferWhereOverEachIf;

        let bad_code = r"
open users.json | each { |u| if $u.age >= 18 { $u } }
";

        LintContext::test_with_parsed_source(bad_code, |context| {
            let violations = rule.check(&context);
            assert!(
                !violations.is_empty(),
                "Should detect each with if checking properties"
            );
        });
    }
}
