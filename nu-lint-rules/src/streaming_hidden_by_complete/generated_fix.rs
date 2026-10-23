use super::RULE;

#[test]
fn fix_cargo_build() {
    RULE.assert_fixed_is("^cargo build | complete", "^cargo build");
}

#[test]
fn fix_cargo_test_release() {
    RULE.assert_fixed_is("^cargo test --release | complete", "^cargo test --release");
}

#[test]
fn fix_git_clone() {
    RULE.assert_fixed_is(
        "^git clone https://github.com/example/repo | complete",
        "^git clone https://github.com/example/repo",
    );
}

#[test]
fn fix_docker_build() {
    RULE.assert_fixed_is("^docker build . | complete", "^docker build .");
}

#[test]
fn fix_npm_install() {
    RULE.assert_fixed_is("^npm install | complete", "^npm install");
}

#[test]
fn fix_make() {
    RULE.assert_fixed_is("^make | complete", "^make");
}

#[test]
fn fix_wget() {
    RULE.assert_fixed_is(
        "^wget https://example.com/file.tar.gz | complete",
        "^wget https://example.com/file.tar.gz",
    );
}
