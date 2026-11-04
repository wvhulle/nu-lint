use nu_lint_analysis::{
    LintLevel,
    ast::block::BlockExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{Span, ast::Pipeline};

use super::{find_open_from_patterns, open_from_span};

pub struct FixData {
    full_span: Span,
    from_decl_id: nu_protocol::DeclId,
    filename_span: Span,
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
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
            );

            let fix_data = FixData {
                full_span,
                from_decl_id: pattern.from_decl_id,
                filename_span: pattern.filename_span,
            };

            (detected, fix_data)
        })
        .collect()
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

    fn short_description(&self) -> &'static str {
        "`open` already parses known formats into structured data"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/open.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Error
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let from_command = context.working_set.get_decl(fix_data.from_decl_id).name();
        Some(Fix {
            explanation: format!(
                "Remove '| {from_command}' - open already parses this file format"
            )
            .into(),
            replacements: vec![Replacement::new(
                fix_data.full_span,
                format!("open {}", context.span_text(fix_data.filename_span)),
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
