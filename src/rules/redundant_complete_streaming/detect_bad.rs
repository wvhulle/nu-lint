use super::RULE;

#[test]
fn detect_cargo_build_complete() {
    RULE.assert_detects("^cargo build | complete");
}

#[test]
fn detect_cargo_test_complete() {
    RULE.assert_detects("^cargo test | complete");
}

#[test]
fn detect_git_clone_complete() {
    RULE.assert_detects("^git clone https://github.com/example/repo | complete");
}

#[test]
fn detect_git_pull_complete() {
    RULE.assert_detects("^git pull | complete");
}

#[test]
fn detect_docker_build_complete() {
    RULE.assert_detects("^docker build . | complete");
}

#[test]
fn detect_npm_install_complete() {
    RULE.assert_detects("^npm install | complete");
}

#[test]
fn detect_make_complete() {
    RULE.assert_detects("^make | complete");
}

#[test]
fn detect_wget_complete() {
    RULE.assert_detects("^wget https://example.com/file.tar.gz | complete");
}

#[test]
fn detect_in_function() {
    RULE.assert_detects(
        r#"
        def build [] {
            ^cargo build --release | complete
        }
    "#,
    );
}
