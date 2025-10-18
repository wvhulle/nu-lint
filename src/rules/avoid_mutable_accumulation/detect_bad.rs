use super::rule;
use crate::LintContext;

#[test]
fn test_detect_mutable_list_accumulation() {
    let bad_code = r"
mut results = []
for item in [1 2 3] {
    $results = ($results | append $item)
}
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect mutable list accumulation pattern"
        );
    });
}

#[test]
fn test_detect_conditional_mutable_accumulation() {
    let bad_code = r"
mut filtered = []
for x in $data {
    if $x > 10 {
        $filtered = ($filtered | append $x)
    }
}
";

    LintContext::test_with_parsed_source(bad_code, |context| {
        assert!(
            !(rule().check)(&context).is_empty(),
            "Should detect conditional mutable accumulation pattern"
        );
    });
}
