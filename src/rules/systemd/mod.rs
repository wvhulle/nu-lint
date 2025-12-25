//! Systemd-related lint rules and shared utilities.
//!
//! This module contains rules for ensuring proper systemd compatibility,
//! particularly for journal log level prefixes.

mod journal_prefix;

pub mod add_journal_prefix;
pub mod mnemonic_log_level;

use nu_protocol::ast::{self, Expr, Expression};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
};

/// Systemd journal log levels (RFC 5424 severity levels).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Emergency = 0,
    Alert = 1,
    Critical = 2,
    Error = 3,
    Warning = 4,
    Notice = 5,
    Info = 6,
    Debug = 7,
}

impl LogLevel {
    /// All recognized keyword aliases that map to log levels.
    pub const KEYWORD_ALIASES: &[(&str, Self)] = &[
        // Emergency
        ("emergency", Self::Emergency),
        ("emerg", Self::Emergency),
        ("panic", Self::Emergency),
        // Alert
        ("alert", Self::Alert),
        // Critical
        ("critical", Self::Critical),
        ("crit", Self::Critical),
        ("fatal", Self::Critical),
        // Error
        ("error", Self::Error),
        ("err", Self::Error),
        ("fail", Self::Error),
        ("failed", Self::Error),
        // Warning
        ("warning", Self::Warning),
        ("warn", Self::Warning),
        ("caution", Self::Warning),
        // Notice
        ("notice", Self::Notice),
        // Info
        ("info", Self::Info),
        ("information", Self::Info),
        // Debug
        ("debug", Self::Debug),
        ("trace", Self::Debug),
    ];

    /// Returns the standard keyword representation for this level.
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Emergency => "emerg",
            Self::Alert => "alert",
            Self::Critical => "crit",
            Self::Error => "err",
            Self::Warning => "warning",
            Self::Notice => "notice",
            Self::Info => "info",
            Self::Debug => "debug",
        }
    }

    /// Returns the numeric string representation for this level.
    pub const fn numeric_str(self) -> &'static str {
        match self {
            Self::Emergency => "0",
            Self::Alert => "1",
            Self::Critical => "2",
            Self::Error => "3",
            Self::Warning => "4",
            Self::Notice => "5",
            Self::Info => "6",
            Self::Debug => "7",
        }
    }

    /// Parses a string into a log level (supports numeric and keyword formats).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "0" => Some(Self::Emergency),
            "1" => Some(Self::Alert),
            "2" => Some(Self::Critical),
            "3" => Some(Self::Error),
            "4" => Some(Self::Warning),
            "5" => Some(Self::Notice),
            "6" => Some(Self::Info),
            "7" => Some(Self::Debug),
            _ => Self::KEYWORD_ALIASES
                .iter()
                .find(|(kw, _)| *kw == s)
                .map(|(_, level)| *level),
        }
    }

    /// Returns true if the string is a numeric prefix (0-7).
    pub fn is_numeric(s: &str) -> bool {
        matches!(s, "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7")
    }

    /// Detects an appropriate log level based on message content.
    pub fn detect_from_message(text: &str) -> Self {
        let lower = text.to_lowercase();
        let first_word = lower
            .split(|c: char| c.is_whitespace() || c == ':')
            .find(|w| !w.is_empty())
            .unwrap_or("");

        Self::KEYWORD_ALIASES
            .iter()
            .find(|(kw, _)| *kw == first_word)
            .map_or_else(|| Self::detect_from_content(&lower), |(_, level)| *level)
    }

    fn detect_from_content(lower_text: &str) -> Self {
        if lower_text.contains("error") || lower_text.contains("fail") {
            Self::Error
        } else if lower_text.contains("warn") {
            Self::Warning
        } else if lower_text.contains("debug") {
            Self::Debug
        } else {
            Self::Info
        }
    }
}

/// Result of checking for a journal prefix in a message.
#[derive(Debug)]
pub enum PrefixStatus {
    /// No prefix found - needs to be added.
    Missing,
    /// Numeric prefix found (e.g., `<3>`) - should use keyword instead.
    Numeric(LogLevel),
    /// Valid keyword prefix found - no action needed.
    Valid,
}

impl PrefixStatus {
    /// Checks a message for systemd journal prefix status.
    pub fn check(text: &str) -> Self {
        let Some(prefix) = text
            .trim_start()
            .strip_prefix('<')
            .and_then(|s| s.split_once('>'))
            .map(|(p, _)| p)
        else {
            return Self::Missing;
        };

        match LogLevel::parse(prefix) {
            Some(level) if LogLevel::is_numeric(prefix) => Self::Numeric(level),
            Some(_) => Self::Valid,
            None => Self::Missing,
        }
    }
}

/// Strips redundant severity keywords from the beginning of a message.
pub fn strip_keyword_prefix(text: &str) -> &str {
    let trimmed = text.trim_start();
    let lower = trimmed.to_lowercase();

    for (keyword, _) in LogLevel::KEYWORD_ALIASES {
        let pattern = format!("{keyword}:");
        if lower.starts_with(&pattern) {
            return trimmed[pattern.len()..].trim_start();
        }
    }
    text
}

/// Extracts the first string part from an expression for prefix checking.
/// Returns `None` for multiline strings (which are skipped).
pub fn extract_first_string_part(expr: &Expression, ctx: &LintContext) -> Option<String> {
    match &expr.expr {
        Expr::String(s) | Expr::RawString(s) if !s.contains('\n') => Some(s.clone()),
        Expr::StringInterpolation(parts) => parts.first().and_then(|first| match &first.expr {
            Expr::String(s) if !s.contains('\n') => Some(s.clone()),
            _ => None,
        }),
        _ => {
            let text = expr.span_text(ctx);
            (!text.contains('\n')).then(|| text.to_string())
        }
    }
}

/// Checks if an expression is a print or echo call.
pub fn is_print_or_echo(expr: &Expression, ctx: &LintContext) -> bool {
    let Expr::Call(call) = &expr.expr else {
        return false;
    };
    let name = call.get_call_name(ctx);
    matches!(name.as_str(), "print" | "echo")
}

/// Checks if a pipeline contains a print or echo call.
pub fn pipeline_contains_print(pipeline: &ast::Pipeline, ctx: &LintContext) -> bool {
    pipeline
        .elements
        .iter()
        .any(|e| is_print_or_echo(&e.expr, ctx))
}
