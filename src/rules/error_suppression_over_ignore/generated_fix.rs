use super::rule;

// Note: Automatic fixes for this rule would require significant code
// restructuring (from `rm file | ignore` to `do -i { rm file }`), so we only
// provide detection and suggestions. These tests verify the rule correctly
// identifies the patterns.

#[test]
fn test_detect_rm_with_ignore() {
    let bad_code = "rm /tmp/file.txt | ignore";

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "do -i");
    rule().assert_suggestion_contains(bad_code, "rm /tmp/file.txt");
    rule().assert_suggestion_contains(bad_code, "do -i { rm /tmp/file.txt }");
    rule().assert_suggestion_contains(bad_code, "try { rm /tmp/file.txt } catch");
}

#[test]
fn test_detect_mv_with_ignore() {
    let bad_code = "mv old.txt new.txt | ignore";

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "mv old.txt new.txt");
    rule().assert_suggestion_contains(bad_code, "do -i { mv old.txt new.txt }");
    rule().assert_suggestion_contains(bad_code, "Instead of:");
    rule().assert_suggestion_contains(bad_code, "Use:");
}

#[test]
fn test_detect_cp_with_ignore() {
    let bad_code = "cp source.txt dest.txt | ignore";

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "cp source.txt dest.txt");
    rule().assert_suggestion_contains(bad_code, "do -i { cp source.txt dest.txt }");
}

#[test]
fn test_detect_mkdir_with_ignore() {
    let bad_code = "mkdir /tmp/newdir | ignore";

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "mkdir /tmp/newdir");
    rule().assert_suggestion_contains(bad_code, "do -i { mkdir /tmp/newdir }");
}

#[test]
fn test_detect_multiple_file_operations() {
    let bad_code = r"
rm file1.txt | ignore
mv old.txt new.txt | ignore
cp a.txt b.txt | ignore
";

    rule().assert_violation_count_exact(bad_code, 3);
}

#[test]
fn test_suggestion_quality() {
    let bad_code = "rm /tmp/file.txt | ignore";

    rule().assert_detects(bad_code);
    rule().assert_suggestion_contains(bad_code, "'| ignore' only discards output, not errors");
    rule().assert_suggestion_contains(bad_code, "Instead of:");
    rule().assert_suggestion_contains(bad_code, "Use:");
    rule().assert_suggestion_contains(bad_code, "Or use try-catch");
    rule().assert_suggestion_contains(bad_code, "rm /tmp/file.txt | ignore");
    rule().assert_suggestion_contains(bad_code, "do -i { rm /tmp/file.txt }");
    rule().assert_suggestion_contains(
        bad_code,
        "try { rm /tmp/file.txt } catch { print 'failed' }",
    );
}
