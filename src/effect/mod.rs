pub mod builtin;
pub mod external;

/// Things that may happen at runtime for both built-in Nu and external (bash)
/// commands. These need preknowledge or familiarity with the command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommonEffect {
    /// Likely to fail under normal circumstances. Probably needs error
    /// handling.
    LikelyErrors,
    /// Dangerous command that may cause data loss.
    Dangerous,
}

pub fn is_dangerous_path(path_str: &str) -> bool {
    EXACT_DANGEROUS_PATHS.contains(&path_str)
        || path_str.starts_with("/..")
        || matches!(
            path_str,
            "/*" | "~/*"
                | "/home/*"
                | "/usr/*"
                | "/etc/*"
                | "/var/*"
                | "/sys/*"
                | "/proc/*"
                | "/dev/*"
                | "/boot/*"
                | "/lib/*"
                | "/bin/*"
                | "/sbin/*"
        )
        || SYSTEM_DIRECTORIES.contains(&path_str)
        || path_str == "/dev/null"
        || (!path_str.contains("/tmp/")
            && SYSTEM_DIRECTORIES
                .iter()
                .any(|dir| path_str.starts_with(&format!("{dir}/"))))
        || ((path_str.starts_with("~.") || path_str.starts_with("~/"))
            && path_str[1..].matches('/').count() <= 1)
}

const SYSTEM_DIRECTORIES: &[&str] = &[
    "/home", "/usr", "/etc", "/var", "/sys", "/proc", "/dev", "/boot", "/lib", "/bin", "/sbin",
];

const EXACT_DANGEROUS_PATHS: &[&str] = &["/", "~", "../", ".."];
