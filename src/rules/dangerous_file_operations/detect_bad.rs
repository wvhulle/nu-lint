use super::rule;

#[test]
fn detect_rm_rf_root() {
    rule().assert_detects("rm -rf /");
}

#[test]
fn detect_rm_rf_wildcard() {
    rule().assert_detects("rm -rf /*");
}

#[test]
fn detect_rm_with_unchecked_variable() {
    rule().assert_detects("let path = $env.HOME; rm -rf $path");
}

#[test]
fn detect_dangerous_parent_directory() {
    rule().assert_detects("rm -rf ../");
}

#[test]
fn detect_mv_to_dangerous_path() {
    rule().assert_detects("mv important_file /");
}

#[test]
fn detect_rm_variable_without_validation() {
    let bad_code = r"
def cleanup [path] {
    rm -rf $path
}";
    rule().assert_detects(bad_code);
}

#[test]
fn detect_rm_system_directories() {
    for dir in [
        "/home", "/etc", "/usr", "/var", "/sys", "/proc", "/boot", "/lib", "/bin", "/sbin", "/dev",
    ] {
        rule().assert_detects(&format!("rm -rf {dir}"));
    }
}

#[test]
fn detect_rm_home_variations() {
    rule().assert_detects("rm -rf ~");
    rule().assert_detects("rm -rf ~/");
    rule().assert_detects("rm ~/.bashrc");
    rule().assert_detects("rm -rf ~/.ssh");
}

#[test]
fn detect_file_operations_to_dev_null() {
    rule().assert_detects("cp important.txt /dev/null");
    rule().assert_detects("mv config.toml /dev/null");
    rule().assert_detects("mv ~/.bashrc /dev/null");
}

#[test]
fn detect_wildcard_deletions() {
    rule().assert_detects("rm -rf /home/*");
    rule().assert_detects("rm -rf /var/*");
    rule().assert_detects("rm /etc/*");
}

#[test]
fn detect_operations_on_critical_files() {
    rule().assert_detects("rm /etc/passwd");
    rule().assert_detects("cp random.txt /etc/hosts");
}

#[test]
fn detect_subdirectory_operations() {
    rule().assert_detects("rm -rf /usr/lib");
    rule().assert_detects("rm -rf /usr/bin");
    rule().assert_detects("rm -rf /etc/systemd");
    rule().assert_detects("rm -rf /usr/local");
}

#[test]
fn detect_copy_move_to_system_dirs() {
    rule().assert_detects("cp malicious.txt /sys/module");
    rule().assert_detects("mv script.sh /usr/bin/important");
    rule().assert_detects("mv /home /tmp");
    rule().assert_detects("cp kernel.img /boot/vmlinuz");
    rule().assert_detects("mv myconfig /etc/config");
}

#[test]
fn detect_variable_home_path() {
    let bad_code = r"
let home = '/home/user'
rm -rf $home";
    rule().assert_detects(bad_code);
}
