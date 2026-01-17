use super::RULE;
use crate::log::init_env_log;

#[test]
fn test_detect_various_path_parameter_names() {
    init_env_log();

    let test_cases = [
        ("source_path", "cp $source_path $dest"),
        ("config_path", "open $config_path | from toml"),
        ("filepath", "open $filepath"),
        ("input_file", "open $input_file"),
        ("target_dir", "ls $target_dir"),
        ("source_directory", "ls $source_directory"),
        ("target_location", "save $target_location"),
    ];

    for (param_name, body) in test_cases {
        let code = format!(r"def test-fn [{param_name}: string] {{ {body} }}");
        RULE.assert_detects(&code);
    }
}

#[test]
fn test_detect_path_parameters_without_type_annotation() {
    init_env_log();

    for param_name in [
        "file_path",
        "config_path",
        "target_directory",
        "backup_location",
    ] {
        let code = format!(r"def test-fn [{param_name}] {{ open ${param_name} }}");
        RULE.assert_detects(&code);
    }
}

#[test]
fn test_detect_multiple_path_parameters() {
    init_env_log();
    let code = r"
def sync-files [source_path: string, target_path: string, backup_path: string] {
    cp $source_path $target_path
    cp $target_path $backup_path
}
";
    RULE.assert_count(code, 3);
}

#[test]
fn test_detect_optional_path_parameter() {
    init_env_log();
    let code = r"
def read-file [file_path?: string] {
    if ($file_path != null) {
        open $file_path
    }
}
";
    RULE.assert_detects(code);
}

#[test]
fn test_detect_exported_function() {
    init_env_log();
    let code = r"
export def save-data [output_path: string, data] {
    $data | save $output_path
}
";
    RULE.assert_detects(code);
}

#[test]
fn test_detect_different_case_variations() {
    init_env_log();
    for (param, body) in [
        ("filePath", "open $filePath"),
        ("SOURCE_PATH", "cp $SOURCE_PATH dest"),
    ] {
        let code = format!(r"def test-fn [{param}: string] {{ {body} }}");
        RULE.assert_detects(&code);
    }
}
