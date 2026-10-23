use nu_protocol::{Span, ast::Pipeline};

use super::{
    extract_delimiter_from_split_call, extract_index_from_call, generate_parse_replacement,
    is_indexed_access_call, is_split_row_call,
};
use crate::{
    Fix, LintLevel, Replacement,
    ast::{block::BlockExt, pipeline::PipelineExt},
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
    pipeline
        .find_command_pairs(context, is_split_row_call, is_indexed_access_call)
        .into_iter()
        .filter_map(|pair| {
            let index = extract_index_from_call(pair.second, context)?;
            let delimiter = extract_delimiter_from_split_call(pair.first, context);

            let violation = Detection::from_global_span(
                "Extract field by name with 'parse' instead of 'split row | get INDEX'",
                pair.span,
            )
            .with_primary_label("index-based access")
            .with_extra_label("splits into list", pair.first.span())
            .with_extra_label("accesses by numeric index", pair.second.span());

            let fix_data = delimiter.map_or(FixData::NoFix, |delim| FixData::WithDelimiter {
                span: pair.span,
                delimiter: delim,
                index,
            });

            Some((violation, fix_data))
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
        "Extract field by name with 'parse' pattern"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Chaining 'split row' with indexed 'get' requires counting field positions manually \
             and doesn't show what each field represents. Use 'parse' to create records with \
             named fields that you can access by name instead of index.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/parse.html")
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
                Some(Fix {
                    explanation: "replace".into(),
                    replacements: vec![Replacement::new(*span, replacement)],
                })
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
