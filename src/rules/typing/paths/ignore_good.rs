use super::RULE;

#[test]
fn test_ignore_path_type_used() {
    let code = r"
def copy-file [source_path: path, dest_path: path] {
    cp $source_path $dest_path
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_string_without_path_name() {
    let code = r"
def greet [name: string, message: string] {
    print $'Hello ($name): ($message)'
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_no_type_annotation() {
    let code = r"
def process [input_data] {
    $input_data | to json
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_non_string_with_path_name() {
    let code = r"
def count-paths [path_count: int] {
    print $path_count
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_record_type_with_path() {
    let code = r"
def process-config [config: record] {
    let path = $config.file_path
    open $path
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_table_type() {
    let code = r"
def filter-paths [paths: table] {
    $paths | where type == 'file'
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_list_of_paths() {
    let code = r"
def backup-files [file_paths: list<path>] {
    for path in $file_paths {
        cp $path backup/
    }
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_closure_parameter() {
    let code = r"
def each-path [callback: closure] {
    ['a.txt', 'b.txt'] | each $callback
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_xpath_string() {
    let code = r"
def query-xml [xpath: string, xml_data: string] {
    # XPath is not a filesystem path
    $xml_data | query web $xpath
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_jsonpath_string() {
    let code = r"
def query-json [jsonpath: string, data: string] {
    # JSONPath is not a filesystem path
    $data | from json | query $jsonpath
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_classpath_string() {
    let code = r"
def run-java [class_path: string, main_class: string] {
    # Java classpath is not a filesystem path
    ^java -cp $class_path $main_class
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_external_command_without_path_name() {
    let code = r"
def compress [input: string, output: string] {
    tar czf $output $input
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_grep_with_pattern() {
    let code = r"
def search [pattern: string, text: string] {
    grep $pattern $text
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_sed_with_replacement() {
    let code = r"
def replace [pattern: string, replacement: string, input: string] {
    sed $pattern $replacement $input
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_docker_with_image_name() {
    let code = r"
def build [image: string, tag: string] {
    docker build -t $image:$tag .
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_git_with_branch() {
    let code = r"
def switch-branch [branch: string] {
    git checkout $branch
}
";
    RULE.assert_ignores(code);
}

#[test]
fn test_ignore_last_profile_false_positive() {
    let code = r#"
def handle_profile_change [last_profile: string] {
    sleep 500ms
    let profile = (get_current_profile)
    if $profile != $last_profile {
        print $"Profile changed from '($last_profile)' to '($profile)'"
        trigger_profile_service $profile
        $profile
    } else {
        $last_profile
    }
}
"#;
    RULE.assert_ignores(code);
}
