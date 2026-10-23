use super::RULE;

#[test]
fn ignore_safe_specific_file_removal() {
    RULE.assert_ignores("rm temp.txt");
}

#[test]
fn ignore_safe_directory_operations() {
    RULE.assert_ignores("mkdir temp; cd temp; rm file.txt");
}

#[test]
fn ignore_rm_in_safe_directory() {
    RULE.assert_ignores("cd /tmp/myproject; rm -rf build");
}

#[test]
fn ignore_safe_tmp_operations() {
    RULE.assert_ignores("rm -rf /tmp/my_temp_dir");
}

#[test]
fn ignore_safe_temp_file_removal() {
    RULE.assert_ignores("rm /tmp/cache/*.log");
}

#[test]
fn ignore_safe_project_cleanup() {
    RULE.assert_ignores("rm -rf ./target");
}

#[test]
fn ignore_safe_relative_path() {
    RULE.assert_ignores("rm -rf build/");
}

#[test]
fn ignore_mv_within_safe_directories() {
    RULE.assert_ignores("mv ./old_name.txt ./new_name.txt");
}

#[test]
fn ignore_cp_to_safe_location() {
    RULE.assert_ignores("cp source.txt /tmp/backup/");
}

#[test]
fn ignore_rm_with_explicit_filename() {
    RULE.assert_ignores("rm package-lock.json");
}

#[test]
fn ignore_safe_glob_in_current_dir() {
    RULE.assert_ignores("rm *.log");
}

#[test]
fn ignore_mv_safe_backup() {
    RULE.assert_ignores("mv old_config.toml backup/old_config.toml");
}

#[test]
fn ignore_cp_with_safe_paths() {
    RULE.assert_ignores("cp data.json backup/data.json");
}

#[test]
fn ignore_rm_in_nested_project_dir() {
    RULE.assert_ignores("rm -rf ./node_modules");
}

#[test]
fn ignore_safe_rm_multiple_files() {
    RULE.assert_ignores("rm file1.txt file2.txt file3.txt");
}

#[test]
fn ignore_safe_cleanup_script() {
    let good_code = r"
def cleanup_build [] {
    rm -rf target/debug
    rm -rf target/release
}";
    RULE.assert_ignores(good_code);
}

#[test]
fn ignore_mv_with_relative_paths() {
    RULE.assert_ignores("mv src/old.rs src/new.rs");
}

#[test]
fn ignore_cp_recursive_safe() {
    RULE.assert_ignores("cp -r templates/ output/");
}

#[test]
fn ignore_rm_in_workspace_subdir() {
    RULE.assert_ignores("cd workspace/project; rm -rf dist");
}

// Variables are now allowed - no data flow analysis for validation
#[test]
fn ignore_variable_in_file_operation() {
    RULE.assert_ignores("rm -rf $path");
}

#[test]
fn ignore_glob_each_pattern() {
    let code = r#"glob "*.tar.gz" | each {|f| cp $f dist/ }"#;
    RULE.assert_ignores(code);
}

#[test]
fn ignore_ls_each_pattern() {
    let code = r"
ls ~/.cache/my_app | where modified < (date now | date add -7day) | each { rm $in.name }
";
    RULE.assert_ignores(code);
}
