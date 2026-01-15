use nu_protocol::{
    Span,
    ast::{Call, Pipeline},
};

use super::{extract_delimiter_from_split_call, is_split_row_call};
use crate::{
    Fix, LintLevel, Replacement,
    ast::{block::BlockExt, call::CallExt, pipeline::PipelineExt, regex::escape_regex},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

enum AccessType {
    First,
    Last,
}

struct FixData {
    span: Span,
    delimiter: String,
    access_type: AccessType,
}

fn is_first_call(call: &Call, ctx: &LintContext) -> bool {
    call.is_call_to_command("first", ctx)
        && call.get_first_positional_arg().is_none_or(|arg| {
            ctx.plain_text(arg.span)
                .parse::<usize>()
                .is_ok_and(|n| n == 1)
        })
}

fn is_last_call(call: &Call, ctx: &LintContext) -> bool {
    call.is_call_to_command("last", ctx)
        && call.get_first_positional_arg().is_none_or(|arg| {
            ctx.plain_text(arg.span)
                .parse::<usize>()
                .is_ok_and(|n| n == 1)
        })
}

fn is_first_or_last(call: &Call, ctx: &LintContext) -> bool {
    is_first_call(call, ctx) || is_last_call(call, ctx)
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    pipeline
        .find_command_pairs(context, is_split_row_call, is_first_or_last)
        .into_iter()
        .filter_map(|pair| {
            let access_type = if is_first_call(pair.second, context) {
                AccessType::First
            } else {
                AccessType::Last
            };

            let delimiter = extract_delimiter_from_split_call(pair.first, context)?;

            let (message, label) = match access_type {
                AccessType::First => (
                    "Use 'parse' instead of 'split row | first'",
                    "split + first pattern",
                ),
                AccessType::Last => (
                    "Use 'parse' instead of 'split row | last'",
                    "split + last pattern",
                ),
            };

            let violation = Detection::from_global_span(message, pair.span)
                .with_primary_label(label)
                .with_extra_label("split row call", pair.first.span())
                .with_extra_label("access call", pair.second.span());

            Some((
                violation,
                FixData {
                    span: pair.span,
                    delimiter,
                    access_type,
                },
            ))
        })
        .collect()
}

fn generate_replacement(delimiter: &str, access_type: &AccessType) -> String {
    let escaped = escape_regex(delimiter);
    match access_type {
        AccessType::First => {
            format!("parse --regex '(?P<first>[^{escaped}]*).*' | get first")
        }
        AccessType::Last => {
            format!("parse --regex '.*{escaped}(?P<last>.*)' | get last")
        }
    }
}

struct SplitFirstLastRule;

impl DetectFix for SplitFirstLastRule {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "split_row_first_last"
    }

    fn short_description(&self) -> &'static str {
        "Use 'parse' instead of 'split row | first' or 'split row | last'"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/parse.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = generate_replacement(&fix_data.delimiter, &fix_data.access_type);
        Some(Fix::with_explanation(
            format!("Replace with '{replacement}'"),
            vec![Replacement::new(fix_data.span, replacement)],
        ))
    }
}

pub static RULE: &dyn Rule = &SplitFirstLastRule;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
