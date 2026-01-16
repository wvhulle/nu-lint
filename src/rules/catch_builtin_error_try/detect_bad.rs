use super::RULE;

// HTTP commands - network I/O can fail
#[test]
fn detect_bare_http_get() {
    RULE.assert_detects("http get https://example.com");
}

#[test]
fn detect_bare_http_post() {
    RULE.assert_detects("http post https://example.com");
}

#[test]
fn detect_http_in_pipeline() {
    RULE.assert_detects("http get https://api.example.com | get data");
}

#[test]
fn detect_http_with_headers() {
    RULE.assert_detects("http get https://api.example.com --headers [Accept application/json]");
}

#[test]
fn detect_http_in_function() {
    RULE.assert_detects(
        r#"
        def fetch-data [] {
            http get https://api.example.com
        }
    "#,
    );
}

#[test]
fn detect_http_with_variable_url() {
    RULE.assert_detects("http get $url");
}

#[test]
fn detect_http_put() {
    RULE.assert_detects(r#"http put https://api.example.com { data: "test" }"#);
}

#[test]
fn detect_http_delete() {
    RULE.assert_detects("http delete https://api.example.com/resource/1");
}

// File system commands - can fail due to permissions, missing files
#[test]
fn detect_open_file() {
    RULE.assert_detects("open data.json");
}

#[test]
fn detect_save_file() {
    RULE.assert_detects(r#"{ data: 1 } | save output.json"#);
}

#[test]
fn detect_rm_file() {
    RULE.assert_detects("rm temp.txt");
}

#[test]
fn detect_mv_file() {
    RULE.assert_detects("mv old.txt new.txt");
}

#[test]
fn detect_cp_file() {
    RULE.assert_detects("cp source.txt dest.txt");
}

#[test]
fn detect_mkdir() {
    RULE.assert_detects("mkdir new_dir");
}

#[test]
fn detect_cd_with_path() {
    // cd with a path argument can fail if path doesn't exist
    RULE.assert_detects("cd /some/path");
}

#[test]
fn detect_touch() {
    RULE.assert_detects("touch new_file.txt");
}

// Parsing commands - can fail with malformed input
#[test]
fn detect_from_json() {
    // from json can fail with malformed JSON like "{invalid"
    RULE.assert_detects(r#"$input | from json"#);
}

#[test]
fn detect_from_yaml() {
    RULE.assert_detects(r#"$input | from yaml"#);
}

#[test]
fn detect_from_toml() {
    RULE.assert_detects(r#"$input | from toml"#);
}

// NOTE: `error make` is intentionally not detected here.
// It's designed to throw errors, not accidentally failing.
// See ignore_good.rs for the test that confirms this.

// ls with path argument - can fail if path doesn't exist
#[test]
fn detect_ls_with_path() {
    RULE.assert_detects("ls /some/path");
}
