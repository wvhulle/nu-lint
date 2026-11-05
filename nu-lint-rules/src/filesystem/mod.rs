use nu_lint_analysis::{
    ast::{call::CallExt, external::literal_content, pipeline::PipelineExt},
    context::LintContext,
};
use nu_protocol::{
    Span,
    ast::{Expression, Pipeline},
};

pub mod from_after_parsed_open;
pub mod open_raw_from_to_open;
pub mod string_param_as_path;

/// Data extracted from an `open FILE | from FORMAT` pipeline pattern
pub struct OpenFromPattern<'a> {
    pub open_expr: &'a Expression,
    pub from_expr: &'a Expression,
    /// Original filename text from source (including quotes)
    pub filename: String,
    pub filename_span: Span,
    /// The format name (e.g., "json", "toml")
    pub format: String,
    pub from_decl_id: nu_protocol::DeclId,
    pub has_raw_flag: bool,
}

/// Find `open FILE | from FORMAT` patterns in a pipeline where the format
/// matches the file extension.
///
/// Returns patterns where the `from` format matches the file's extension
/// according to `context.format_for_extension()`.
#[must_use]
pub fn find_open_from_patterns<'a>(
    pipeline: &'a Pipeline,
    context: &'a LintContext,
) -> Vec<OpenFromPattern<'a>> {
    pipeline
        .find_command_pairs(
            context,
            |call, ctx| call.builtin_name(ctx) == Some("open"),
            |call, ctx| {
                call.builtin_name(ctx)
                    .is_some_and(|builtin_name| builtin_name.starts_with("from "))
            },
        )
        .into_iter()
        .filter_map(|pair| {
            let format = pair.second.builtin_name(context)?.strip_prefix("from ")?;

            let filename_arg = pair.first.get_first_positional_arg()?;
            let filename = context.expr_text(filename_arg);

            let file_format = context.format_for_extension(literal_content(filename_arg)?)?;

            if file_format != format {
                return None;
            }

            let has_raw_flag = pair.first.has_named_flag("raw");

            Some(OpenFromPattern {
                open_expr: &pipeline.elements[pair.first_index].expr,
                from_expr: &pipeline.elements[pair.second_index].expr,
                filename: filename.to_string(),
                filename_span: filename_arg.span,
                from_decl_id: pair.second.decl_id,
                format: format.to_string(),
                has_raw_flag,
            })
        })
        .collect()
}

/// Create a span covering from open to from (the entire `open FILE | from
/// FORMAT` expression)
#[must_use]
pub fn open_from_span(pattern: &OpenFromPattern) -> Span {
    Span::new(pattern.open_expr.span.start, pattern.from_expr.span.end)
}
