use nu_protocol::{
    Span,
    ast::{Expr, Expression, Pipeline},
};

use crate::{
    ast::{call::CallExt, pipeline::PipelineExt},
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
    pipeline
        .find_command_pairs(
            context,
            |call, ctx| call.is_call_to_command("open", ctx),
            |call, ctx| call.get_call_name(ctx).starts_with("from "),
        )
        .into_iter()
        .filter_map(|pair| {
            let from_name = pair.second.get_call_name(context);
            let format = from_name.strip_prefix("from ")?;

            let filename_arg = pair.first.get_first_positional_arg()?;
            let filename = context.plain_text(filename_arg.span);

            let filename_content = match &filename_arg.expr {
                Expr::String(s) | Expr::RawString(s) | Expr::GlobPattern(s, _) => s.as_str(),
                _ => filename,
            };

            let file_format = context.format_for_extension(filename_content)?;

            if file_format != format {
                return None;
            }

            let open_text = context.plain_text(pair.span);
            let has_raw_flag = open_text.contains("--raw") || open_text.contains("-r ");

            Some(OpenFromPattern {
                open_expr: &pipeline.elements[pair.first_index].expr,
                from_expr: &pipeline.elements[pair.second_index].expr,
                filename: filename.to_string(),
                format: format.to_string(),
                has_raw_flag,
            })
        })
        .collect()
}

/// Create a span covering from open to from (the entire `open FILE | from
/// FORMAT` expression)
pub fn open_from_span(pattern: &OpenFromPattern) -> Span {
    Span::new(pattern.open_expr.span.start, pattern.from_expr.span.end)
}
