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

struct FixData {
    span: Span,
    delimiter: String,
}

fn is_first_call(call: &Call, ctx: &LintContext) -> bool {
    call.is_call_to_command("first", ctx)
        && call
            .get_first_positional_arg()
            .is_none_or(|arg| ctx.expr_text(arg).parse::<usize>().is_ok_and(|n| n == 1))
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    pipeline
        .find_command_pairs(context, is_split_row_call, is_first_call)
        .into_iter()
        .filter_map(|pair| {
            let delimiter = extract_delimiter_from_split_call(pair.first, context)?;

            // Skip space delimiter - handled by split_row_space_to_split_words
            if delimiter == " " {
                return None;
            }

            let violation = Detection::from_global_span(
                "Extract first field with 'parse' instead of 'split row | first'",
                pair.span,
            )
            .with_primary_label("can be simplified")
            .with_extra_label("splits into list", pair.first.span())
            .with_extra_label("takes first element", pair.second.span());

            Some((
                violation,
                FixData {
                    span: pair.span,
                    delimiter,
                },
            ))
        })
        .collect()
}

fn generate_replacement(delimiter: &str) -> String {
    format!("parse \"{{first}}{delimiter}{{_}}\" | get first")
}

struct SplitFirstToParse;

impl DetectFix for SplitFirstToParse {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "split_first_to_parse"
    }

    fn short_description(&self) -> &'static str {
        "Extract first field with 'parse' pattern"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "The pattern 'split row <delim> | first' creates an intermediate list just to get \
             the first element. Use 'parse \"{first}<delim>{_}\" | get first' instead, which \
             directly extracts the first field without allocating a list.",
        )
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
        let replacement = generate_replacement(&fix_data.delimiter);
        Some(Fix::with_explanation(
            format!("Replace with '{replacement}'"),
            vec![Replacement::new(fix_data.span, replacement)],
        ))
    }
}

pub static RULE: &dyn Rule = &SplitFirstToParse;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
