use std::str::from_utf8;

use nu_protocol::{
    Span,
    ast::{Call, Expr, Expression},
};

use crate::context::LintContext;

/// Characters that have special meaning at the start of a token in Nushell.
/// These would cause the parser to interpret the token differently.
const SPECIAL_START_CHARS: &[char] = &[
    '-',  // Flag/option prefix
    '$',  // Variable reference
    '(',  // Subexpression/closure start
    '[',  // List start
    '{',  // Record/closure start
    '`',  // Backtick string start
    '\'', // Single quote string start
    '"',  // Double quote string start
    '#',  // Comment start
];

/// Characters that cannot appear in bare words (they have syntactic meaning).
const BARE_WORD_FORBIDDEN: &[char] = &[
    ' ', '\t', '\n', '\r', // Whitespace separates tokens
    '|',  // Pipeline separator
    ';',  // Statement separator
    '(',  // Subexpression
    ')',  // Subexpression end
    '[',  // List/cell path
    ']',  // List end
    '{',  // Record/closure
    '}',  // Record/closure end
    '`',  // Backtick string delimiter
    '\'', // Single quote delimiter
    '"',  // Double quote delimiter
];

/// Reserved words that would be parsed as different types or cause errors if
/// unquoted.
const RESERVED_LITERALS: &[&str] = &[
    "true", "false", "null", // Parsed as different types
    "&&",   // Rejected by parser (suggests using ; or and)
];

/// Checks if a string can be represented as a bare word in Nushell.
///
/// A bare word is a string without quotes that Nushell interprets literally.
/// This function returns `false` if the string can safely be a bare word,
/// and `true` if quotes are needed.
pub fn bare_word_needs_quotes(content: &str) -> bool {
    if content.is_empty() {
        return true;
    }

    // Check for reserved literals that would change meaning
    if RESERVED_LITERALS.contains(&content) {
        return true;
    }

    // Check if it would be parsed as a number
    if looks_like_number(content) {
        return true;
    }

    // Check first character for special meaning
    let first_char = content.chars().next().unwrap();
    if SPECIAL_START_CHARS.contains(&first_char) {
        return true;
    }

    // Check for forbidden characters anywhere in the string
    for ch in content.chars() {
        if BARE_WORD_FORBIDDEN.contains(&ch) {
            return true;
        }
    }

    false
}

/// Checks if content looks like a number literal (int, float, hex, binary,
/// octal, filesize, or duration).
fn looks_like_number(content: &str) -> bool {
    // Integer
    if content.parse::<i64>().is_ok() {
        return true;
    }

    // Float (including those starting with .)
    if content.parse::<f64>().is_ok() {
        return true;
    }

    // Hex (0x...), binary (0b...), octal (0o...)
    if content.starts_with("0x") || content.starts_with("0b") || content.starts_with("0o") {
        return true;
    }

    // Filesize suffixes
    let filesize_suffixes = [
        "b", "kb", "mb", "gb", "tb", "pb", "kib", "mib", "gib", "tib", "pib",
    ];
    for suffix in filesize_suffixes {
        if content.to_lowercase().ends_with(suffix) {
            let prefix = &content[..content.len() - suffix.len()];
            if prefix.parse::<f64>().is_ok() {
                return true;
            }
        }
    }

    // Duration suffixes
    let duration_suffixes = ["ns", "us", "µs", "ms", "sec", "min", "hr", "day", "wk"];
    for suffix in duration_suffixes {
        if content.ends_with(suffix) {
            let prefix = &content[..content.len() - suffix.len()];
            if prefix.parse::<f64>().is_ok() {
                return true;
            }
        }
    }

    false
}

/// Checks if a string needs quotes when used as a cell path member (record
/// field access).
///
/// Cell path members have stricter requirements than general bare words:
/// - They appear after a dot in expressions like `$record.field`
/// - Numeric strings would be interpreted as list indices
/// - Spaces require quotes for proper parsing
pub fn cell_path_member_needs_quotes(content: &str) -> bool {
    if content.is_empty() {
        return true;
    }

    // Numeric strings would be interpreted as list indices
    if content.parse::<i64>().is_ok() {
        return true;
    }

    // Spaces always need quotes in cell paths
    if content.contains(' ') {
        return true;
    }

    // Check for characters that would break cell path parsing
    for ch in content.chars() {
        if matches!(ch, '.' | '[' | ']' | '(' | ')' | '"' | '\'' | '`') {
            return true;
        }
    }

    false
}

/// The type and content of a string in Nushell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringFormat {
    /// Double-quoted string literal: `"text"`
    Double(String),
    /// Single-quoted string literal: `'text'`
    Single(String),
    /// Raw string: `r#'text'#`
    Raw(String),
    /// Bare word string: `text` (no quotes)
    BareWord(String),
    /// String interpolation with double quotes: `$"text ($var)"`
    InterpolationDouble(String),
    /// String interpolation with single quotes: `$'text ($var)'`
    InterpolationSingle(String),
    /// Backtick string: `` `text` `` (used for paths/commands, has different
    /// semantics)
    Backtick(String),
}

impl StringFormat {
    /// Returns the string content regardless of format.
    pub fn content(&self) -> &str {
        match self {
            Self::Double(s)
            | Self::Single(s)
            | Self::Raw(s)
            | Self::BareWord(s)
            | Self::InterpolationDouble(s)
            | Self::InterpolationSingle(s)
            | Self::Backtick(s) => s,
        }
    }

    /// Checks if two string formats are compatible for merging.
    pub const fn is_compatible(&self, other: &Self) -> bool {
        use StringFormat::{
            BareWord, Double, InterpolationDouble, InterpolationSingle, Raw, Single,
        };
        matches!(
            (self, other),
            // Plain strings (Double, Single, Raw, BareWord) can all be merged together
            (Double(_) | Single(_) | Raw(_) | BareWord(_), Double(_) | Single(_) | Raw(_) | BareWord(_))
                // Interpolations must match quote style
                | (InterpolationDouble(_), InterpolationDouble(_))
                | (InterpolationSingle(_), InterpolationSingle(_)) /* Backtick strings are never
                                                                    * compatible for merging
                                                                    * as they have different
                                                                    * semantics */
        )
    }

    /// Reconstructs the original source text with the given content.
    pub fn reconstruct(&self, new_content: &str) -> String {
        match self {
            Self::Double(_) => format!("\"{new_content}\""),
            Self::Single(_) => format!("'{new_content}'"),
            Self::Raw(_) => format!("r#'{new_content}'#"),
            Self::BareWord(_) => new_content.to_string(),
            Self::InterpolationDouble(_) => format!("$\"{new_content}\""),
            Self::InterpolationSingle(_) => format!("$'{new_content}'"),
            Self::Backtick(_) => format!("`{new_content}`"),
        }
    }

    /// Extracts string format and content from an expression using AST
    /// analysis.
    pub fn from_expression(expr: &Expression, ctx: &LintContext) -> Option<Self> {
        match &expr.expr {
            Expr::String(s) => Some(Self::detect_string_format(s, expr.span, ctx)),
            Expr::RawString(s) => Some(Self::Raw(s.clone())),
            Expr::StringInterpolation(_) => Self::detect_interpolation_format(expr.span, ctx),
            _ => None,
        }
    }

    /// Extracts string format from a Call's first positional argument.
    pub fn from_call_arg(call: &Call, ctx: &LintContext) -> Option<Self> {
        use nu_protocol::ast::Argument;

        let expr = call.arguments.iter().find_map(|arg| match arg {
            Argument::Positional(e) | Argument::Unknown(e) => Some(e),
            _ => None,
        })?;

        Self::from_expression(expr, ctx)
    }

    fn detect_string_format(content: &str, span: Span, ctx: &LintContext) -> Self {
        let bytes = ctx.working_set.get_span_contents(span);
        let source = from_utf8(bytes).unwrap_or("");

        if source.starts_with('`') {
            Self::Backtick(content.to_string())
        } else if source.starts_with('"') {
            Self::Double(content.to_string())
        } else if source.starts_with('\'') {
            Self::Single(content.to_string())
        } else if source.starts_with("r#") || source.starts_with("r'") {
            Self::Raw(content.to_string())
        } else {
            Self::BareWord(content.to_string())
        }
    }

    fn detect_interpolation_format(span: Span, ctx: &LintContext) -> Option<Self> {
        let bytes = ctx.working_set.get_span_contents(span);
        let source = from_utf8(bytes).unwrap_or("");

        let single_quote = source
            .strip_prefix("$'")
            .and_then(|s| s.strip_suffix('\''))
            .map(|stripped| Self::InterpolationSingle(stripped.to_string()));

        source
            .strip_prefix("$\"")
            .and_then(|s| s.strip_suffix('"'))
            .map(|stripped| Self::InterpolationDouble(stripped.to_string()))
            .or(single_quote)
    }
}
