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
fn detect_rm_home_directory() {
    rule().assert_detects("rm -rf /home");
}

#[test]
fn detect_rm_home_user_directory() {
    rule().assert_detects("rm -rf /home/user");
}

#[test]
fn detect_rm_home_wildcard() {
    rule().assert_detects("rm -rf /home/*");
}

#[test]
fn detect_cp_to_dev_null() {
    rule().assert_detects("cp important.txt /dev/null");
}

#[test]
fn detect_mv_to_dev_null() {
    rule().assert_detects("mv config.toml /dev/null");
}

#[test]
fn detect_rm_etc_directory() {
    rule().assert_detects("rm -rf /etc");
}

#[test]
fn detect_rm_usr_directory() {
    rule().assert_detects("rm -rf /usr");
}

#[test]
fn detect_rm_var_directory() {
    rule().assert_detects("rm -rf /var");
}

#[test]
fn detect_rm_sys_directory() {
    rule().assert_detects("rm -rf /sys");
}

#[test]
fn detect_rm_proc_directory() {
    rule().assert_detects("rm -rf /proc");
}

#[test]
fn detect_rm_tilde_home() {
    rule().assert_detects("rm -rf ~");
}

#[test]
fn detect_mv_home_config_to_dev_null() {
    rule().assert_detects("mv ~/.bashrc /dev/null");
}

#[test]
fn detect_rm_etc_passwd() {
    rule().assert_detects("rm /etc/passwd");
}

#[test]
fn detect_rm_var_with_wildcard() {
    rule().assert_detects("rm -rf /var/*");
}

#[test]
fn detect_cp_to_sys_directory() {
    rule().assert_detects("cp malicious.txt /sys/module");
}

#[test]
fn detect_mv_to_usr_bin() {
    rule().assert_detects("mv script.sh /usr/bin/important");
}

#[test]
fn detect_rm_etc_wildcard() {
    rule().assert_detects("rm /etc/*");
}

#[test]
fn detect_rm_boot_directory() {
    rule().assert_detects("rm -rf /boot");
}

#[test]
fn detect_rm_lib_directory() {
    rule().assert_detects("rm -rf /lib");
}

#[test]
fn detect_rm_bin_directory() {
    rule().assert_detects("rm -rf /bin");
}

#[test]
fn detect_rm_sbin_directory() {
    rule().assert_detects("rm -rf /sbin");
}

#[test]
fn detect_mv_entire_home_dir() {
    rule().assert_detects("mv /home /tmp");
}

#[test]
fn detect_cp_over_critical_file() {
    rule().assert_detects("cp random.txt /etc/hosts");
}

#[test]
fn detect_rm_dev_directory() {
    rule().assert_detects("rm -rf /dev");
}

#[test]
fn detect_rm_usr_lib() {
    rule().assert_detects("rm -rf /usr/lib");
}

#[test]
fn detect_rm_usr_bin() {
    rule().assert_detects("rm -rf /usr/bin");
}

#[test]
fn detect_rm_etc_systemd() {
    rule().assert_detects("rm -rf /etc/systemd");
}

#[test]
fn detect_variable_home_path() {
    let bad_code = r"
let home = '/home/user'
rm -rf $home";
    rule().assert_detects(bad_code);
}

#[test]
fn detect_rm_home_tilde_slash() {
    rule().assert_detects("rm -rf ~/");
}

#[test]
fn detect_rm_home_bashrc() {
    rule().assert_detects("rm ~/.bashrc");
}

#[test]
fn detect_rm_home_ssh() {
    rule().assert_detects("rm -rf ~/.ssh");
}

#[test]
fn detect_cp_to_boot() {
    rule().assert_detects("cp kernel.img /boot/vmlinuz");
}

#[test]
fn detect_mv_to_etc() {
    rule().assert_detects("mv myconfig /etc/config");
}

#[test]
fn detect_rm_var_log_wildcard() {
    rule().assert_detects("rm -rf /var/log/*");
}

#[test]
fn detect_rm_usr_local() {
    rule().assert_detects("rm -rf /usr/local");
}
