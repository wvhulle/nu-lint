use super::rule;
use crate::LintContext;

#[test]
fn test_ignore_for_loop_with_print() {
    let good_code = r"
for item in $items {
    print $item
}
";
    LintContext::test_with_parsed_source(good_code, |context| {
        assert!(
            (rule().check)(&context).is_empty(),
            "Should ignore for loop with print (side effect)"
        );
    });
}

#[test]
fn test_ignore_for_loop_with_external_command() {
    let good_code = r#"
for file in $files {
    ^cp $file $"backup_($file)"
}
"#;
    LintContext::test_with_parsed_source(good_code, |context| {
        assert!(
            (rule().check)(&context).is_empty(),
            "Should ignore for loop with external commands"
        );
    });
}

#[test]
fn test_ignore_for_loop_with_file_operations() {
    let good_code = r"
for name in $file_names {
    save $name
}
";
    LintContext::test_with_parsed_source(good_code, |context| {
        assert!(
            (rule().check)(&context).is_empty(),
            "Should ignore for loop with file operations"
        );
    });
}

#[test]
fn test_ignore_for_loop_with_mutation() {
    let good_code = r"
for item in $items {
    mut result = $item + 1
}
";
    LintContext::test_with_parsed_source(good_code, |context| {
        assert!(
            (rule().check)(&context).is_empty(),
            "Should ignore for loop with mutation"
        );
    });
}

#[test]
fn test_ignore_for_loop_with_system_commands() {
    let good_code = r"
for dir in $directories {
    cd $dir
}
";
    LintContext::test_with_parsed_source(good_code, |context| {
        assert!(
            (rule().check)(&context).is_empty(),
            "Should ignore for loop with system commands like cd"
        );
    });
}

#[test]
fn test_ignore_for_loop_with_git_commands() {
    let good_code = r"
for branch in $branches {
    git checkout $branch
}
";
    LintContext::test_with_parsed_source(good_code, |context| {
        assert!(
            (rule().check)(&context).is_empty(),
            "Should ignore for loop with git commands"
        );
    });
}

#[test]
fn test_accept_each_usage() {
    let good_code = r"
$items | each { |item| $item * 2 }
";
    LintContext::test_with_parsed_source(good_code, |context| {
        assert!(
            (rule().check)(&context).is_empty(),
            "Should accept proper each usage"
        );
    });
}
