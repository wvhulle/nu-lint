use super::rule;
use crate::log::instrument;

#[test]

fn test_detect_source_path_parameter() {
    instrument();

    let code = r"
def copy-file [source_path: string, dest: string] {
    cp $source_path $dest
}
";
    rule().assert_detects(code);
}

#[test]

fn test_detect_config_path_parameter() {
    instrument();
    let code = r"
def load-config [config_path: string] {
    open $config_path | from toml
}
";
    rule().assert_detects(code);
}

#[test]

fn test_detect_filepath_parameter() {
    instrument();
    let code = r"
def process-file [filepath: string] {
    open $filepath
}
";
    rule().assert_detects(code);
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
    rule().assert_violation_count_exact(code, 3);
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

fn test_detect_mixed_case_path() {
    instrument();
    let code = r"
def open-file [filePath: string] {
    open $filePath
}
";
    rule().assert_detects(code);
}

#[test]

fn test_detect_uppercase_path() {
    instrument();
    let code = r"
def backup [SOURCE_PATH: string, DEST_PATH: string] {
    cp $SOURCE_PATH $DEST_PATH
}
";
    rule().assert_violation_count_exact(code, 2);
}

#[test]

fn test_detect_file_parameter() {
    instrument();
    let code = r"
def process [input_file: string] {
    open $input_file
}
";
    rule().assert_detects(code);
}

#[test]

fn test_detect_dir_parameter() {
    instrument();
    let code = r"
def list-contents [target_dir: string] {
    ls $target_dir
}
";
    rule().assert_detects(code);
}

#[test]

fn test_detect_directory_parameter() {
    instrument();
    let code = r"
def scan [source_directory: string] {
    ls $source_directory
}
";
    rule().assert_detects(code);
}

#[test]

fn test_detect_location_parameter() {
    instrument();
    let code = r"
def save-to [target_location: string] {
    save $target_location
}
";
    rule().assert_detects(code);
}

#[test]

fn test_detect_file_path_no_type_annotation() {
    instrument();
    let code = r"
def main [file_path] {
    open $file_path
}
";
    rule().assert_detects(code);
}

#[test]

fn test_detect_config_path_no_type_annotation() {
    instrument();
    let code = r"
def load-settings [config_path] {
    open $config_path | from toml
}
";
    rule().assert_detects(code);
}

#[test]

fn test_detect_directory_no_type_annotation() {
    instrument();
    let code = r"
def list-dir [target_directory] {
    ls $target_directory
}
";
    rule().assert_detects(code);
}

#[test]

fn test_detect_location_no_type_annotation() {
    instrument();
    let code = r"
def backup-to [backup_location] {
    cp data.txt $backup_location
}
";
    rule().assert_detects(code);
}

#[test]
fn test_detect_tar_with_path_name() {
    instrument();
    let code = r"
def archive [backup_folder: string] {
    tar czf backup.tar.gz $backup_folder
}
";
    rule().assert_detects(code);
}

#[test]
fn test_detect_rsync_with_directory() {
    instrument();
    let code = r"
def sync-dir [source_dir: string, target_dir: string] {
    rsync -av $source_dir $target_dir
}
";
    rule().assert_violation_count_exact(code, 2);
}

#[test]
fn test_detect_scp_with_file_path() {
    instrument();
    let code = r"
def upload [file_path: string, remote: string] {
    scp $file_path $remote
}
";
    rule().assert_detects(code);
}

#[test]
fn test_detect_wget_with_location() {
    instrument();
    let code = r"
def download [url: string, output_location: string] {
    wget -O $output_location $url
}
";
    rule().assert_detects(code);
}

#[test]
fn test_detect_curl_with_output_path() {
    instrument();
    let code = r"
def fetch [url: string, output_path: string] {
    curl -o $output_path $url
}
";
    rule().assert_detects(code);
}

#[test]
fn test_detect_zip_with_directory() {
    instrument();
    let code = r"
def compress [source_directory: string] {
    ^zip -r archive.zip $source_directory
}
";
    rule().assert_detects(code);
}

#[test]
fn test_detect_unzip_with_target_dir() {
    instrument();
    let code = r"
def extract [archive: string, target_dir: string] {
    unzip $archive -d $target_dir
}
";
    rule().assert_detects(code);
}
