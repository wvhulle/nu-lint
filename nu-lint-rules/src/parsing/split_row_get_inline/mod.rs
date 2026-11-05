use nu_lint_analysis::{
    Fix, LintLevel, Replacement,
    ast::{block::BlockExt, pipeline::PipelineExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
use nu_protocol::{
    Span,
    ast::{Call, Pipeline},
};

use super::{
    extract_delimiter_from_split_call, extract_index_from_call, generate_parse_replacement,
    is_indexed_access_call, is_split_row_call,
};

pub enum FixData {
    WithDelimiter {
        span: Span,
        split_call: Call,
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
            let has_literal_delimiter =
                extract_delimiter_from_split_call(pair.first, context).is_some();

            let violation = Detection::from_global_span(
                "Extract field by name with 'parse' instead of 'split row | get INDEX'",
                pair.span,
            )
            .with_primary_label("index-based access")
            .with_extra_label("splits into list", pair.first.span())
            .with_extra_label("accesses by numeric index", pair.second.span());

            let fix_data = if has_literal_delimiter {
                FixData::WithDelimiter {
                    span: pair.span,
                    split_call: pair.first.clone(),
                    index,
                }
            } else {
                FixData::NoFix
            };

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
        "Extract field by name with `parse` pattern"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Chaining `split row` with indexed `get` requires counting field positions manually \
             and doesn't show what each field represents. Use `parse` to create records with \
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

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        match fix_data {
            FixData::WithDelimiter {
                span,
                split_call,
                index,
            } => {
                let delimiter = extract_delimiter_from_split_call(split_call, context)?;
                let replacement = generate_parse_replacement(&delimiter, &[*index]);
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
