use nu_protocol::Span;

use crate::{
    LintLevel,
    ast::external::{CliSpec, Flag, LineSource},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const LINES: Flag = Flag::switch('l', "lines");

static SPEC: CliSpec = CliSpec {
    flags: &[LINES],
    numeric_shorthand: None,
};

const NOTE: &str = "'lines | length' returns the line count as an integer instead of text. It \
                    also counts a final line without trailing newline, which 'wc -l' skips.";

struct WcToLength;

pub struct FixData<'a> {
    span: Span,
    source: LineSource<'a>,
}

impl DetectFix for WcToLength {
    type FixInput<'a> = FixData<'a>;

    fn id(&self) -> &'static str {
        "wc_to_length"
    }

    fn short_description(&self) -> &'static str {
        "`wc -l` replaceable with `lines | length`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/length.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["wc"])
            .iter()
            .filter_map(|invocation| {
                let parsed = invocation.parse(&SPEC)?;
                if !parsed.has(LINES) {
                    return None;
                }
                let source = LineSource::of(invocation, &parsed.operands)?;
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("'wc -l' counting lines");
                Some((
                    detection,
                    FixData {
                        span: invocation.span,
                        source,
                    },
                ))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = format!("{}lines | length", fix_data.source.open_prefix(context));
        Some(Fix {
            explanation: format!("Count lines with '{replacement}'").into(),
            replacements: vec![Replacement::new(fix_data.span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &WcToLength;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
