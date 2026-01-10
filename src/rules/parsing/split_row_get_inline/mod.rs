use nu_protocol::{
    Span,
    ast::{Expr, Pipeline},
};

use super::{
    extract_delimiter_from_split_call, extract_index_from_call, generate_parse_replacement,
    is_indexed_access_call, is_split_row_call,
};
use crate::{
    Fix, LintLevel, Replacement,
    ast::block::BlockExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

pub enum FixData {
    WithDelimiter {
        span: Span,
        delimiter: String,
        index: usize,
    },
    NoFix,
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    if pipeline.elements.len() < 2 {
        return vec![];
    }

    pipeline
        .elements
        .windows(2)
        .filter_map(|window| {
            let [current, next] = window else {
                return None;
            };
            let (Expr::Call(split_call), Expr::Call(access_call)) =
                (&current.expr.expr, &next.expr.expr)
            else {
                return None;
            };

            if !is_split_row_call(split_call, context)
                || !is_indexed_access_call(access_call, context)
            {
                return None;
            }

            let index = extract_index_from_call(access_call, context)?;
            let span = Span::new(current.expr.span.start, next.expr.span.end);

            let delimiter = extract_delimiter_from_split_call(split_call, context);

            delimiter.map_or_else(
                || {
                    let violation = Detection::from_global_span(
                        "Use 'parse' instead of chaining 'split row | get' in a pipeline",
                        span,
                    )
                    .with_primary_label("split row followed by indexed get in same pipeline");
                    Some((violation, FixData::NoFix))
                },
                |delim| {
                    let violation = Detection::from_global_span(
                        "Use 'parse' instead of chaining 'split row | get' in a pipeline",
                        span,
                    )
                    .with_primary_label("split row followed by indexed get in same pipeline");
                    Some((
                        violation,
                        FixData::WithDelimiter {
                            span,
                            delimiter: delim,
                            index,
                        },
                    ))
                },
            )
        })
        .collect()
}

struct SplitGetRule;

impl DetectFix for SplitGetRule {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "split_row_get_inline"
    }

    fn short_description(&self) -> &'static str {
        "Replace chained 'split row | get INDEX' pattern in a single pipeline with 'parse' for \
         structured text extraction"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/parse.html")
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            r#"Chaining 'split row' with indexed 'get' access is verbose and error-prone because:
- You need to count field positions manually
- The code doesn't show what each field represents
- Off-by-one errors are common

Instead, use 'parse' for structured text extraction.

The 'parse' command creates records with named fields. Access them by name
(e.g., $result.field_name) instead of by index, making your code more readable
and maintainable.

Example:
  Before: "192.168.1.100:8080" | split row ":" | get 0
  After:  "192.168.1.100:8080" | parse "{ip}:{port}" | get ip"#,
        )
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        match fix_data {
            FixData::WithDelimiter {
                span,
                delimiter,
                index,
            } => {
                let replacement = generate_parse_replacement(delimiter, &[*index]);
                Some(Fix::with_explanation(
                    format!("Replace 'split row | get/skip' with '{replacement}'"),
                    vec![Replacement::new(*span, replacement)],
                ))
            }
            FixData::NoFix => None,
        }
    }
}

pub static RULE: &dyn Rule = &SplitGetRule;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
