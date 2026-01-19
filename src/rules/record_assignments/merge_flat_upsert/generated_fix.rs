use super::RULE;

#[test]
fn test_fix_two_flat() {
    RULE.assert_fixed_is(
        r#"mut config = {}
$config.debug = true
$config.verbose = false"#,
        r#"mut config = {}
$config = ($config | upsert debug true | upsert verbose false)"#,
    );
}

#[test]
fn test_fix_three_flat() {
    RULE.assert_fixed_is(
        r#"mut s = {}
$s.a = 1
$s.b = 2
$s.c = 3"#,
        r#"mut s = {}
$s = ($s | upsert a 1 | upsert b 2 | upsert c 3)"#,
    );
}

#[test]
fn test_fix_mixed_value_types() {
    RULE.assert_fixed_is(
        r#"mut opts = {}
$opts.enabled = true
$opts.count = 42
$opts.name = "test""#,
        r#"mut opts = {}
$opts = ($opts | upsert enabled true | upsert count 42 | upsert name "test")"#,
    );
}

#[test]
fn test_fix_with_expressions() {
    RULE.assert_fixed_is(
        r#"mut data = {}
$data.items = [1 2 3]
$data.total = ($items | math sum)"#,
        r#"mut data = {}
$data = ($data | upsert items [1 2 3] | upsert total ($items | math sum))"#,
    );
}

#[test]
fn test_fix_hyphenated_field() {
    RULE.assert_fixed_is(
        r#"mut cfg = {}
$cfg.log-level = "info"
$cfg.max-retries = 3"#,
        r#"mut cfg = {}
$cfg = ($cfg | upsert log-level "info" | upsert max-retries 3)"#,
    );
}
