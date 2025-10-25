use super::rule;

#[test]
fn test_good_builtin_env_access() {
    let good = "$env.HOME";
    rule().assert_ignores(good);
}

#[test]
fn test_good_env_with_fallback() {
    let good = "$env.EDITOR? | default 'vim'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_env_in_pipeline() {
    let good = "echo $env.PATH | split row (char esep)";
    rule().assert_ignores(good);
}

#[test]
fn test_good_builtin_date() {
    let good = "date now";
    rule().assert_ignores(good);
}

#[test]
fn test_good_date_formatting() {
    let good = "date now | format date '%Y-%m-%d'";
    rule().assert_ignores(good);
}

#[test]
fn test_good_date_into_conversion() {
    let good = "'2024-01-01' | into datetime";
    rule().assert_ignores(good);
}

#[test]
fn test_good_builtin_whoami() {
    let good = "whoami";
    rule().assert_ignores(good);
}

#[test]
fn test_good_builtin_sys_host() {
    let good = "(sys host).hostname";
    rule().assert_ignores(good);
}

#[test]
fn test_good_sys_full_info() {
    let good = "sys host | select name kernel_version";
    rule().assert_ignores(good);
}

#[test]
fn test_good_sys_net_for_ip() {
    let good = "sys net | where name == 'eth0' | get ip.0";
    rule().assert_ignores(good);
}

#[test]
fn test_good_builtin_help() {
    let good = "help ls";
    rule().assert_ignores(good);
}

#[test]
fn test_good_builtin_which() {
    let good = "which nu";
    rule().assert_ignores(good);
}

#[test]
fn test_good_builtin_input() {
    let good = "let name = input 'Enter your name: '";
    rule().assert_ignores(good);
}

#[test]
fn test_good_input_secure() {
    let good = "let password = input -s 'Password: '";
    rule().assert_ignores(good);
}

#[test]
fn test_good_cd_with_tilde() {
    let good = "cd ~/projects";
    rule().assert_ignores(good);
}
