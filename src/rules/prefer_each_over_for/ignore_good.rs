use super::rule;

#[test]
fn test_ignore_for_loop_with_print() {
    let good_code = r"
for item in $items {
    print $item
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_for_loop_with_external_command() {
    let good_code = r#"
for file in $files {
    ^cp $file $"backup_($file)"
}
"#;
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_for_loop_with_file_operations() {
    let good_code = r"
for name in $file_names {
    save $name
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_for_loop_with_mutation() {
    let good_code = r"
for item in $items {
    mut result = $item + 1
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_for_loop_with_system_commands() {
    let good_code = r"
for dir in $directories {
    cd $dir
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_ignore_for_loop_with_git_commands() {
    let good_code = r"
for branch in $branches {
    git checkout $branch
}
";
    rule().assert_ignores(good_code);
}

#[test]
fn test_accept_each_usage() {
    let good_code = r"
$items | each { |item| $item * 2 }
";
    rule().assert_ignores(good_code);
}
