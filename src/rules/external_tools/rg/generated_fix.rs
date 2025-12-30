use super::RULE;
use crate::log::instrument;

#[test]
fn fix_simple_rg_to_find() {
    instrument();
    let source = r#"^rg \"todo\""#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"find \"todo\""#);
    RULE.assert_fix_explanation_contains(source, "case-insensitive");
}

#[test]
fn fix_rg_with_file_to_where() {
    instrument();
    let source = r#"^rg \"error\" logs.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"open logs.txt | lines | where $it =~ \"error\""#);
}

#[test]
fn fix_rg_case_insensitive_flag() {
    instrument();
    let source = r#"^rg -i \"warning\" logs.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(
        source,
        r#"open logs.txt | lines | where $it =~ \"warning\""#,
    );
    RULE.assert_fix_explanation_contains(source, "filter for 'warning'");
}

#[test]
fn fix_rg_invert_match() {
    instrument();
    let source = r#"^rg -v \"debug\" app.log"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"open app.log | lines | where $it !~ \"debug\""#);
    RULE.assert_fix_explanation_contains(source, "invert matches");
}

#[test]
fn fix_rg_line_numbers() {
    instrument();
    let source = r#"^rg -n \"TODO\" source.rs"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(
        source,
        r#"open source.rs | lines | enumerate | where $it.item =~ \"TODO\""#,
    );
    RULE.assert_fixed_contains(source, "enumerate");
}

#[test]
fn fix_rg_count() {
    instrument();
    let source = r#"^rg -c \"error\" logs.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(
        source,
        r#"open logs.txt | lines | where $it =~ \"error\" | length"#,
    );
    RULE.assert_fix_explanation_contains(source, "length");
}

#[test]
fn fix_rg_line_numbers_and_count() {
    instrument();
    let source = r#"^rg -nc \"panic\" src/main.rs"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, "enumerate");
    RULE.assert_fixed_contains(source, "length");
    RULE.assert_fixed_contains(source, r#"$it.item =~ \"panic\""#);
}

#[test]
fn fix_rg_fixed_strings() {
    instrument();
    let source = r#"^rg -F \"literal\" README.md"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(
        source,
        r#"open README.md | lines | where $it | str contains \"literal\""#,
    );
    RULE.assert_fix_explanation_contains(source, "str contains");
}

#[test]
fn fix_rg_multiple_files() {
    instrument();
    let source = r#"^rg \"expr\" file1.nu file2.nu"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(
        source,
        r#"open file1.nu file2.nu | lines | where $it =~ \"expr\""#,
    );
}

#[test]
fn fix_explanation_mentions_find() {
    instrument();
    let source = r#"^rg \"pattern\""#;
    RULE.assert_count(source, 1);
    RULE.assert_fix_explanation_contains(source, "find");
    RULE.assert_fix_explanation_contains(source, "case-insensitive");
}

#[test]
fn fix_explanation_mentions_where() {
    instrument();
    let source = r#"^rg \"pattern\" file.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fix_explanation_contains(source, "where");
}

#[test]
fn fix_explanation_mentions_structured_data() {
    instrument();
    let source = r#"^rg \"error\" logs.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fix_explanation_contains(source, "structured");
}

#[test]
fn fix_no_file_uses_find() {
    instrument();
    let source = r#"^rg \"test\""#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"find \"test\""#);
    RULE.assert_fix_explanation_contains(source, "case-insensitive");
}

#[test]
fn fix_preserves_pattern() {
    instrument();
    let source = r#"^rg \"complex[0-9]+\" file.txt"#;
    RULE.assert_count(source, 1);
    RULE.assert_fixed_contains(source, r#"where $it =~ \"complex[0-9]+\""#);
}
