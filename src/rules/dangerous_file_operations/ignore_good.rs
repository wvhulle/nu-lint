use super::rule;

#[test]
fn ignore_safe_specific_file_removal() {
    rule().assert_ignores("rm temp.txt");
}

#[test]
fn ignore_rm_with_validation() {
    let good_code = r"
def cleanup [path] {
    if ($path | path exists) and ($path != '/') {
        rm -rf $path
    }
}";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_safe_directory_operations() {
    rule().assert_ignores("mkdir temp; cd temp; rm file.txt");
}

#[test]
fn ignore_rm_in_safe_directory() {
    rule().assert_ignores("cd /tmp/myproject; rm -rf build");
}

#[test]
fn ignore_validated_variable_usage() {
    let good_code = r"
let path = $env.TEMP_DIR
if ($path | str starts-with '/tmp/') {
    rm -rf $path
}";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_safe_tmp_operations() {
    rule().assert_ignores("rm -rf /tmp/my_temp_dir");
}

#[test]
fn ignore_safe_temp_file_removal() {
    rule().assert_ignores("rm /tmp/cache/*.log");
}

#[test]
fn ignore_safe_project_cleanup() {
    rule().assert_ignores("rm -rf ./target");
}

#[test]
fn ignore_safe_relative_path() {
    rule().assert_ignores("rm -rf build/");
}

#[test]
fn ignore_mv_within_safe_directories() {
    rule().assert_ignores("mv ./old_name.txt ./new_name.txt");
}

#[test]
fn ignore_cp_to_safe_location() {
    rule().assert_ignores("cp source.txt /tmp/backup/");
}

#[test]
fn ignore_rm_with_explicit_filename() {
    rule().assert_ignores("rm package-lock.json");
}

#[test]
fn ignore_safe_glob_in_current_dir() {
    rule().assert_ignores("rm *.log");
}

#[test]
fn ignore_mv_safe_backup() {
    rule().assert_ignores("mv old_config.toml backup/old_config.toml");
}

#[test]
fn ignore_cp_with_safe_paths() {
    rule().assert_ignores("cp data.json backup/data.json");
}

#[test]
fn ignore_rm_in_nested_project_dir() {
    rule().assert_ignores("rm -rf ./node_modules");
}

#[test]
fn ignore_safe_rm_multiple_files() {
    rule().assert_ignores("rm file1.txt file2.txt file3.txt");
}

#[test]
fn ignore_validated_path_check() {
    let good_code = r"
def safe_remove [dir: string] {
    if ($dir | path exists) and ($dir | str starts-with './projects/') {
        rm -rf $dir
    }
}";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_safe_cleanup_script() {
    let good_code = r"
def cleanup_build [] {
    rm -rf target/debug
    rm -rf target/release
}";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_mv_with_relative_paths() {
    rule().assert_ignores("mv src/old.rs src/new.rs");
}

#[test]
fn ignore_cp_recursive_safe() {
    rule().assert_ignores("cp -r templates/ output/");
}

#[test]
fn ignore_safe_cache_cleanup() {
    let good_code = r"
ls ~/.cache/my_app | where modified < (date now | date add -7day) | each { rm $in.name }
";
    rule().assert_ignores(good_code);
}

#[test]
fn ignore_rm_in_workspace_subdir() {
    rule().assert_ignores("cd workspace/project; rm -rf dist");
}

#[test]
fn ignore_safe_log_rotation() {
    let good_code = r"
glob logs/*.log | where size > 100MB | each { rm $in }
";
    rule().assert_ignores(good_code);
}
