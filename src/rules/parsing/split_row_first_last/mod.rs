use nu_protocol::{
    Span,
    ast::{Block, Call, Expr, Pipeline},
};

use super::{extract_delimiter_from_split_call, is_split_row_call};
use crate::{
    Fix, LintLevel, Replacement,
    ast::{call::CallExt, regex::escape_regex},
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

fn check_pipeline_for_split_first_last(
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

        if !is_split_row_call(split_call, context) {
            return None;
        }

        let access_type = if is_first_call(access_call, context) {
            AccessType::First
        } else if is_last_call(access_call, context) {
            AccessType::Last
        } else {
            return None;
        };

        let delimiter = extract_delimiter_from_split_call(split_call, context)?;
        let span = Span::new(current.expr.span.start, next.expr.span.end);

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

        let violation = Detection::from_global_span(message, span)
            .with_primary_label(label)
            .with_extra_label("split row call", current.expr.span)
            .with_extra_label("access call", next.expr.span);

        Some((
            violation,
            FixData {
                span,
                delimiter,
                access_type,
            },
        ))
    })
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<(Detection, FixData)>) {
    for pipeline in &block.pipelines {
        if let Some(violation) = check_pipeline_for_split_first_last(pipeline, context) {
            violations.push(violation);
        }
    }
}

fn generate_replacement(delimiter: &str, access_type: &AccessType) -> String {
    let escaped = escape_regex(delimiter);
    match access_type {
        // For first: capture non-greedy up to first delimiter
        // "a:b:c" | parse --regex "(?P<first>[^:]*):.*" → first = "a"
        AccessType::First => {
            format!("parse --regex '(?P<first>[^{escaped}]*).*' | get first")
        }
        // For last: greedy capture before last delimiter
        // "a:b:c" | parse --regex ".*:(?P<last>.*)" → last = "c"
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

    fn explanation(&self) -> &'static str {
        "Use 'parse' instead of 'split row | first' or 'split row | last'"
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
