use nu_protocol::{
    Span,
    ast::{Call, Pipeline},
};

use super::{extract_delimiter_from_split_call, is_split_row_call};
use crate::{
    Fix, LintLevel, Replacement,
    ast::{block::BlockExt, call::CallExt, pipeline::PipelineExt},
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
    access_type: AccessType,
}

fn is_first_call(call: &Call, ctx: &LintContext) -> bool {
    call.is_call_to_command("first", ctx)
        && call
            .get_first_positional_arg()
            .is_none_or(|arg| ctx.expr_text(arg).parse::<usize>().is_ok_and(|n| n == 1))
}

fn is_last_call(call: &Call, ctx: &LintContext) -> bool {
    call.is_call_to_command("last", ctx)
        && call
            .get_first_positional_arg()
            .is_none_or(|arg| ctx.expr_text(arg).parse::<usize>().is_ok_and(|n| n == 1))
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

            // Only handle space delimiter
            if delimiter != " " {
                return None;
            }

            let accessor = match access_type {
                AccessType::First => "first",
                AccessType::Last => "last",
            };

            let violation = Detection::from_global_span(
                format!("Use 'split words | {accessor}' for whitespace splitting"),
                pair.span,
            )
            .with_primary_label("can be simplified")
            .with_extra_label("splits on space character", pair.first.span())
            .with_extra_label(format!("takes {accessor} element"), pair.second.span());

            Some((
                violation,
                FixData {
                    span: pair.span,
                    access_type,
                },
            ))
        })
        .collect()
}

const fn generate_replacement(access_type: &AccessType) -> &'static str {
    match access_type {
        AccessType::First => "split words | first",
        AccessType::Last => "split words | last",
    }
}

struct SplitRowSpaceToSplitWords;

impl DetectFix for SplitRowSpaceToSplitWords {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "split_row_space_to_split_words"
    }

    fn short_description(&self) -> &'static str {
        "Use 'split words' for whitespace splitting"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "The command 'split words' is optimized for splitting on whitespace. It handles \
             multiple spaces and different whitespace characters better than 'split row \" \"', \
             which only splits on single space characters.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/split_words.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Hint) // TODO: may have false positives
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = generate_replacement(&fix_data.access_type);
        Some(Fix {
            explanation: "replace".into(),
            replacements: vec![Replacement::new(fix_data.span, replacement.to_string())],
        })
    }
}

pub static RULE: &dyn Rule = &SplitRowSpaceToSplitWords;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
