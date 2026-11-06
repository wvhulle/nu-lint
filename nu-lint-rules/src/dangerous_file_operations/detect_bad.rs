use super::RULE;

#[test]
fn detect_rm_rf_root() {
    RULE.assert_detects("rm -rf /");
}

#[test]
fn detect_rm_rf_wildcard() {
    RULE.assert_detects("rm -rf /*");
}

#[test]
fn detect_dangerous_parent_directory() {
    RULE.assert_detects("rm -rf ../");
}

#[test]
fn detect_mv_to_dangerous_path() {
    RULE.assert_detects("mv important_file /");
}

#[test]
fn detect_rm_system_directories() {
    for dir in [
        "/home", "/etc", "/usr", "/var", "/sys", "/proc", "/boot", "/lib", "/bin", "/sbin", "/dev",
    ] {
        RULE.assert_detects(&format!("rm -rf {dir}"));
    }
}

#[test]
fn detect_rm_home_variations() {
    RULE.assert_detects("rm -rf ~");
    RULE.assert_detects("rm -rf ~/");
    RULE.assert_detects("rm ~/.bashrc");
    RULE.assert_detects("rm -rf ~/.ssh");
}

#[test]
fn detect_file_operations_to_dev_null() {
    RULE.assert_detects("cp important.txt /dev/null");
    RULE.assert_detects("mv config.toml /dev/null");
    RULE.assert_detects("mv ~/.bashrc /dev/null");
}

#[test]
fn detect_wildcard_deletions() {
    RULE.assert_detects("rm -rf /home/*");
    RULE.assert_detects("rm -rf /var/*");
    RULE.assert_detects("rm /etc/*");
}

#[test]
fn detect_operations_on_critical_files() {
    RULE.assert_detects("rm /etc/passwd");
    RULE.assert_detects("cp random.txt /etc/hosts");
}

#[test]
fn detect_subdirectory_operations() {
    RULE.assert_detects("rm -rf /usr/lib");
    RULE.assert_detects("rm -rf /usr/bin");
    RULE.assert_detects("rm -rf /etc/systemd");
    RULE.assert_detects("rm -rf /usr/local");
}

#[test]
fn detect_copy_move_to_system_dirs() {
    RULE.assert_detects("cp malicious.txt /sys/module");
    RULE.assert_detects("mv script.sh /usr/bin/important");
    RULE.assert_detects("mv /home /tmp");
    RULE.assert_detects("cp kernel.img /boot/vmlinuz");
    RULE.assert_detects("mv myconfig /etc/config");
}

#[test]
fn detect_quoted_external_command() {
    RULE.assert_detects(r#"^"rm" -rf /"#);
}

#[test]
fn detect_quoted_recursive_flag() {
    RULE.assert_detects(r#"^rm "-rf" /"#);
}

#[test]
fn detect_recursive_rm_as_critical() {
    use nu_lint_analysis::context::LintContext;

    for code in ["rm -rf /", "rm --recursive ~", "^rm -rf /", "^rm -R /etc"] {
        let violations = LintContext::test_get_violations(code, |context| RULE.check(context));
        let labels: Vec<_> = violations
            .iter()
            .filter_map(|violation| violation.primary_label.as_deref())
            .collect();
        assert_eq!(labels, ["recursive operation on dangerous path"], "{code}");
    }
}
