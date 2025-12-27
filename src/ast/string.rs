use std::str::from_utf8;

use nu_protocol::{
    Span,
    ast::{Call, Expr, Expression},
};

use crate::context::LintContext;

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
