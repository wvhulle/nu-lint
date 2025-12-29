/// Regex special characters that need escaping in literal patterns.
/// These characters have special meaning in regex syntax.
pub const REGEX_SPECIAL_CHARS: &[char] = &[
    '\\', '.', '+', '*', '?', '(', ')', '[', ']', '{', '}', '|', '^', '$',
];

/// Checks if a string contains any regex special characters.
pub fn contains_regex_special_chars(text: &str) -> bool {
    text.chars().any(|c| REGEX_SPECIAL_CHARS.contains(&c))
}

/// Escapes regex special characters in a string for literal matching.
pub fn escape_regex(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        if REGEX_SPECIAL_CHARS.contains(&c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}
