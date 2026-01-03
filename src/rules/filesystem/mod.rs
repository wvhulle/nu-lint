use nu_protocol::{
    Span,
    ast::{Expr, Expression, Pipeline},
};

use crate::{
    ast::{call::CallExt, string::strip_quotes},
    context::LintContext,
};

pub mod from_after_parsed_open;
pub mod open_raw_from_to_open;
pub mod string_as_path;

/// Data extracted from an `open FILE | from FORMAT` pipeline pattern
pub struct OpenFromPattern<'a> {
    pub open_expr: &'a Expression,
    pub from_expr: &'a Expression,
    /// Original filename text from source (including quotes)
    pub filename: String,
    /// The format name (e.g., "json", "toml")
    pub format: String,
    pub has_raw_flag: bool,
}

/// Find `open FILE | from FORMAT` patterns in a pipeline where the format
/// matches the file extension.
///
/// Returns patterns where the `from` format matches the file's extension
/// according to `context.format_for_extension()`.
pub fn find_open_from_patterns<'a>(
    pipeline: &'a Pipeline,
    context: &'a LintContext,
) -> Vec<OpenFromPattern<'a>> {
    let elements = &pipeline.elements;
    if elements.len() < 2 {
        return vec![];
    }

    let mut patterns = Vec::new();

    for i in 0..elements.len() - 1 {
        let open_expr = &elements[i].expr;
        let from_expr = &elements[i + 1].expr;

        let Expr::Call(open_call) = &open_expr.expr else {
            continue;
        };

        if !open_call.is_call_to_command("open", context) {
            continue;
        }

        let Expr::Call(from_call) = &from_expr.expr else {
            continue;
        };

        let from_name = from_call.get_call_name(context);
        if !from_name.starts_with("from ") {
            continue;
        }

        let format = from_name.strip_prefix("from ").unwrap_or_default();

        let Some(filename_arg) = open_call.get_first_positional_arg() else {
            continue;
        };

        let filename = context.get_span_text(filename_arg.span);

        // Extract actual filename content for extension detection, handling all string
        // formats
        let filename_content = match &filename_arg.expr {
            Expr::String(s) | Expr::RawString(s) | Expr::GlobPattern(s, _) => s.as_str(),
            _ => strip_quotes(filename),
        };

        let Some(file_format) = context.format_for_extension(filename_content) else {
            continue;
        };

        if file_format != format {
            continue;
        }

        let open_text = context.get_span_text(open_expr.span);
        let has_raw_flag = open_text.contains("--raw") || open_text.contains("-r ");

        // Store original filename text (with quotes) for use in fixes
        patterns.push(OpenFromPattern {
            open_expr,
            from_expr,
            filename: filename.to_string(),
            format: format.to_string(),
            has_raw_flag,
        });
    }

    patterns
}

/// Create a span covering from open to from (the entire `open FILE | from
/// FORMAT` expression)
pub fn open_from_span(pattern: &OpenFromPattern) -> Span {
    Span::new(pattern.open_expr.span.start, pattern.from_expr.span.end)
}
