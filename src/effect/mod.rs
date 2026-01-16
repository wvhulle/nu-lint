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

pub fn matches_short_flag(arg_text: &str, flag_char: char) -> bool {
    arg_text
        .strip_prefix('-')
        .filter(|rest| !rest.starts_with('-'))
        .is_some_and(|rest| rest.contains(flag_char))
        || is_dashless_flags(arg_text) && arg_text.contains(flag_char)
}

pub fn matches_long_flag(arg_text: &str, pattern: &str) -> bool {
    arg_text == pattern || arg_text.starts_with(&format!("{pattern}="))
}

fn is_dashless_flags(text: &str) -> bool {
    !text.starts_with('-')
        && !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
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
