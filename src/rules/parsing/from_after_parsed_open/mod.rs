use nu_protocol::{Span, ast::{Block, Expr, Pipeline}};

use crate::{
    LintLevel,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

use super::{find_open_from_patterns, open_from_span};

pub struct FixData {
    full_span: Span,
    format: String,
    filename: String,
}

fn check_pipeline(
    pipeline: &Pipeline,
    context: &LintContext,
) -> Vec<(Detection, FixData)> {
    find_open_from_patterns(pipeline, context)
        .into_iter()
        .filter(|pattern| !pattern.has_raw_flag)
        .map(|pattern| {
            let full_span = open_from_span(&pattern);
            let format = &pattern.format;
            let filename = &pattern.filename;

            let detected = Detection::from_global_span(
                format!(
                    "'from {format}' expects text input but 'open {filename}' returns structured \
                     data"
                ),
                pattern.from_expr.span,
            )
            .with_primary_label("'from' expects text, not structured data")
            .with_extra_label(
                "Nu recognizes this format and parses it automatically",
                pattern.open_expr.span,
            )
            .with_help(format!(
                "Remove '| from {format}' - Nu already parses .{format} files into structured data"
            ));

            let fix_data = FixData {
                full_span,
                format: format.clone(),
                filename: filename.clone(),
            };

            (detected, fix_data)
        })
        .collect()
}

fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<(Detection, FixData)>) {
    for pipeline in &block.pipelines {
        violations.extend(check_pipeline(pipeline, context));
    }
}

/// Detects `open FILE.json | from json` which is an error because `open`
/// already recognizes the file format and parses it into structured data. The
/// `from` command expects text input, not structured data.
struct FromAfterParsedOpen;

impl DetectFix for FromAfterParsedOpen {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "from_after_parsed_open"
    }

    fn explanation(&self) -> &'static str {
        "Nu's 'open' already parses known formats into structured data"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/open.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Error
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
        Some(Fix {
            explanation: format!(
                "Remove '| from {}' - open already parses this file format",
                fix_data.format
            )
            .into(),
            replacements: vec![Replacement::new(
                fix_data.full_span,
                format!("open {}", fix_data.filename),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &FromAfterParsedOpen;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
