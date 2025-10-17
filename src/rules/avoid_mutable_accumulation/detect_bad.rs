#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::avoid_mutable_accumulation::AvoidMutableAccumulation;
    use crate::context::LintContext;
    use crate::rule::Rule;

    #[test]
    fn test_detect_mutable_list_accumulation() {
        let rule = AvoidMutableAccumulation::default();

        let bad_code = r"
mut results = []
for item in [1 2 3] {
    $results = ($results | append $item)
}
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect mutable list accumulation pattern"
        );
    }

    #[test]
    fn test_detect_conditional_mutable_accumulation() {
        let rule = AvoidMutableAccumulation::default();

        let bad_code = r"
mut filtered = []
for x in $data {
    if $x > 10 {
        $filtered = ($filtered | append $x)
    }
}
";
        let context = LintContext::test_from_source(bad_code);
        assert!(
            !rule.check(&context).is_empty(),
            "Should detect conditional mutable accumulation pattern"
        );
    }
}