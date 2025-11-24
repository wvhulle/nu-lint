use super::rule;
use crate::log::instrument;

#[test]
fn test_detect_various_path_parameter_names() {
    instrument();

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
        rule().assert_detects(&code);
    }
}

#[test]
fn test_detect_path_parameters_without_type_annotation() {
    instrument();

    for param_name in [
        "file_path",
        "config_path",
        "target_directory",
        "backup_location",
    ] {
        let code = format!(r"def test-fn [{param_name}] {{ open ${param_name} }}");
        rule().assert_detects(&code);
    }
}

#[test]
fn test_detect_multiple_path_parameters() {
    instrument();
    let code = r"
def sync-files [source_path: string, target_path: string, backup_path: string] {
    cp $source_path $target_path
    cp $target_path $backup_path
}
";
    rule().assert_count(code, 3);
}

#[test]
fn test_detect_optional_path_parameter() {
    instrument();
    let code = r"
def read-file [file_path?: string] {
    if ($file_path != null) {
        open $file_path
    }
}
";
    rule().assert_detects(code);
}

#[test]
fn test_detect_exported_function() {
    instrument();
    let code = r"
export def save-data [output_path: string, data] {
    $data | save $output_path
}
";
    rule().assert_detects(code);
}

#[test]
fn test_detect_different_case_variations() {
    instrument();
    for (param, body) in [
        ("filePath", "open $filePath"),
        ("SOURCE_PATH", "cp $SOURCE_PATH dest"),
    ] {
        let code = format!(r"def test-fn [{param}: string] {{ {body} }}");
        rule().assert_detects(&code);
    }
}

#[test]
fn test_detect_path_params_with_external_commands() {
    instrument();

    let test_cases = [
        ("backup_folder", "tar czf backup.tar.gz $backup_folder"),
        ("source_directory", "^zip -r archive.zip $source_directory"),
    ];

    for (param, body) in test_cases {
        let code = format!(r"def test-fn [{param}: string] {{ {body} }}");
        rule().assert_detects(&code);
    }
}

#[test]
fn test_detect_multipath_external_commands() {
    instrument();

    let test_cases = [
        (
            "source_dir",
            "target_dir",
            "rsync -av $source_dir $target_dir",
            2,
        ),
        ("file_path", "remote", "scp $file_path $remote", 1),
        ("url", "output_location", "wget -O $output_location $url", 1),
        ("url", "output_path", "curl -o $output_path $url", 1),
        ("archive", "target_dir", "unzip $archive -d $target_dir", 1),
    ];

    for (param1, param2, body, expected_count) in test_cases {
        let code = format!(r"def test-fn [{param1}: string, {param2}: string] {{ {body} }}");
        rule().assert_count(&code, expected_count);
    }
}
