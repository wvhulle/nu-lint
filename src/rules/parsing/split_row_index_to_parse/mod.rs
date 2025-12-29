use nu_protocol::{
    Span,
    ast::{Block, Expr, Pipeline},
};

use super::{
    extract_delimiter_from_split_call, extract_index_from_call, generate_parse_replacement,
    is_indexed_access_call, is_split_row_call,
};
use crate::{
    Fix, LintLevel, Replacement,
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

fn check_pipeline_for_split_get(
    pipeline: &Pipeline,
    context: &LintContext,
) -> Option<(Detection, FixData)> {
    if pipeline.elements.len() < 2 {
        return None;
    }

    pipeline.elements.windows(2).find_map(|window| {
        let [current, next] = window else {
            return None;
        };
        let (Expr::Call(split_call), Expr::Call(access_call)) =
            (&current.expr.expr, &next.expr.expr)
        else {
            return None;
        };

        if !is_split_row_call(split_call, context) || !is_indexed_access_call(access_call, context)
        {
            return None;
        }

        let index = extract_index_from_call(access_call, context)?;
        let span = Span::new(current.expr.span.start, next.expr.span.end);

        let delimiter = extract_delimiter_from_split_call(split_call, context);

        delimiter.map_or_else(
            || {
                let violation = Detection::from_global_span(
                    "Manual string splitting with indexed access - consider using 'parse'",
                    span,
                )
                .with_primary_label("split + index pattern")
                .with_help(
                    "Use 'parse \"{field0} {field1}\"' for structured text extraction. For \
                     complex delimiters containing regex special characters, use 'parse --regex' \
                     with named capture groups like '(?P<field0>.*)delimiter(?P<field1>.*)'",
                );
                Some((violation, FixData::NoFix))
            },
            |delim| {
                let replacement = generate_parse_replacement(&delim, &[index]);
                let violation = Detection::from_global_span(
                    "Manual string splitting with indexed access - consider using 'parse'",
                    span,
                )
                .with_primary_label("split + index pattern")
                .with_help(format!(
                    "Use '{replacement}' for structured text extraction. Access fields by name \
                     (e.g., $result.field{index}) instead of index."
                ));
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
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<(Detection, FixData)>) {
    for pipeline in &block.pipelines {
        if let Some(violation) = check_pipeline_for_split_get(pipeline, context) {
            violations.push(violation);
        }
    }
}

struct SplitGetRule;

impl DetectFix for SplitGetRule {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "split_row_get_to_parse"
    }

    fn explanation(&self) -> &'static str {
        "Prefer 'parse' command over 'split row | get' pattern"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/parse.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = Vec::new();

        check_block(context.ast, context, &mut violations);

        violations.extend(context.detect_with_fix_data(|expr, ctx| {
            let mut expr_violations = Vec::new();

            if let Expr::Closure(block_id) | Expr::Block(block_id) = &expr.expr {
                let block = ctx.working_set.get_block(*block_id);
                check_block(block, ctx, &mut expr_violations);
            }

            expr_violations
        }));

        violations
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
