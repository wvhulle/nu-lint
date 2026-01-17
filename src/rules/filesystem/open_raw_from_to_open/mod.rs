use nu_protocol::{Span, ast::Pipeline};

use super::{find_open_from_patterns, open_from_span};
use crate::{
    LintLevel,
    ast::block::BlockExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

pub struct FixData {
    full_span: Span,
    filename: String,
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    find_open_from_patterns(pipeline, context)
        .into_iter()
        .filter(|pattern| pattern.has_raw_flag)
        .map(|pattern| {
            let full_span = open_from_span(&pattern);
            let format = &pattern.format;
            let filename = &pattern.filename;

            let detected = Detection::from_global_span(
                format!("Redundant 'open --raw | from {format}' - use 'open {filename}' instead"),
                pattern.from_expr.span,
            )
            .with_primary_label("unnecessary explicit parsing")
            .with_extra_label(
                "--raw returns text instead of structured data",
                pattern.open_expr.span,
            );

            let fix_data = FixData {
                full_span,
                filename: filename.clone(),
            };

            (detected, fix_data)
        })
        .collect()
}

/// Detects `open --raw FILE.json | from json` which is redundant because
/// `open FILE.json` (without --raw) already recognizes the format and parses it
/// into structured data automatically.
struct OpenRawFromToOpen;

impl DetectFix for OpenRawFromToOpen {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "open_raw_from_to_open"
    }

    fn short_description(&self) -> &'static str {
        "Simplify `open --raw | from` to `open`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/open.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!(
                "Simplify to 'open {}' - Nu auto-parses this format",
                fix_data.filename
            )
            .into(),
            replacements: vec![Replacement::new(
                fix_data.full_span,
                format!("open {}", fix_data.filename),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &OpenRawFromToOpen;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
