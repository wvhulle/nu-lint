use super::rule;

#[test]
fn test_ignore_path_type_used() {
    let code = r"
def copy-file [source_path: path, dest_path: path] {
    cp $source_path $dest_path
}
";
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_string_without_path_name() {
    let code = r"
def greet [name: string, message: string] {
    print $'Hello ($name): ($message)'
}
";
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_no_type_annotation() {
    let code = r"
def process [input_data] {
    $input_data | to json
}
";
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_non_string_with_path_name() {
    let code = r"
def count-paths [path_count: int] {
    print $path_count
}
";
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_record_type_with_path() {
    let code = r"
def process-config [config: record] {
    let path = $config.file_path
    open $path
}
";
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_table_type() {
    let code = r"
def filter-paths [paths: table] {
    $paths | where type == 'file'
}
";
    rule().assert_ignores(code);
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
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_closure_parameter() {
    let code = r"
def each-path [callback: closure] {
    ['a.txt', 'b.txt'] | each $callback
}
";
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_xpath_string() {
    let code = r"
def query-xml [xpath: string, xml_data: string] {
    # XPath is not a filesystem path
    $xml_data | query web $xpath
}
";
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_jsonpath_string() {
    let code = r"
def query-json [jsonpath: string, data: string] {
    # JSONPath is not a filesystem path
    $data | from json | query $jsonpath
}
";
    rule().assert_ignores(code);
}

#[test]
fn test_ignore_classpath_string() {
    let code = r"
def run-java [class_path: string, main_class: string] {
    # Java classpath is not a filesystem path
    ^java -cp $class_path $main_class
}
";
    rule().assert_ignores(code);
}
