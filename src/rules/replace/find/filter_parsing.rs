use crate::rules::replace::find::rule;

// Size filter tests

#[test]
fn converts_size_greater_than() {
    rule().assert_replacement_contains(r"^find . -size +1M", "ls ./**/* | where size > 1mb");
}

#[test]
fn converts_size_less_than() {
    rule().assert_replacement_contains(r"^find . -size -500k", "ls ./**/* | where size < 500kb");
}

#[test]
fn converts_size_exact() {
    rule().assert_replacement_contains(r"^find . -size 1G", "ls ./**/* | where size == 1gb");
}

#[test]
fn converts_size_in_bytes() {
    rule().assert_replacement_contains(r"^find . -size 1024", "ls ./**/* | where size == 1024b");
}

// Time filter tests

#[test]
fn converts_mtime_older_than() {
    rule().assert_replacement_contains(
        r"^find . -mtime +7",
        "ls ./**/* | where modified < ((date now) - 7day)",
    );
}

#[test]
fn converts_mtime_newer_than() {
    rule().assert_replacement_contains(
        r"^find . -mtime -3",
        "ls ./**/* | where modified > ((date now) - 3day)",
    );
}

#[test]
fn converts_mmin_for_minutes() {
    rule().assert_replacement_contains(
        r"^find . -mmin -60",
        "ls ./**/* | where modified > ((date now) - 60day)",
    );
}

// Type filter tests

#[test]
fn converts_type_file() {
    rule().assert_replacement_contains(r"^find . -type f", "ls ./**/* | where type == file");
}

#[test]
fn converts_type_directory() {
    rule().assert_replacement_contains(r"^find . -type d", "ls ./**/* | where type == dir");
}

#[test]
fn converts_type_symlink() {
    rule().assert_replacement_contains(r"^find . -type l", "ls ./**/* | where type == symlink");
}

// Combined filter tests

#[test]
fn combines_multiple_filters_in_pipeline() {
    let source = r#"^find . -name "*.rs" -type f -size +100k -mtime -7"#;

    rule().assert_replacement_contains(
        source,
        "ls ./**/*.rs | where type == file | where size > 100kb | where modified > ((date now) - \
         7day)",
    );
    rule().assert_fix_explanation_contains(source, "type:");
    rule().assert_fix_explanation_contains(source, "size:");
    rule().assert_fix_explanation_contains(source, "time:");
}

#[test]
fn handles_empty_and_type_together() {
    rule().assert_replacement_contains(
        r"^find . -type f -empty",
        "ls ./**/* | where type == file | where size == 0b",
    );
}
