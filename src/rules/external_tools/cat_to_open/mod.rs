use nu_protocol::{Span, ast::Expression};

use crate::{
    LintLevel,
    ast::external::{CliSpec, LineSource},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

static SPEC: CliSpec = CliSpec {
    flags: &[],
    numeric_shorthand: None,
};

const NOTE: &str = "'open --raw' reads the file without starting an external process and works \
                    the same on every platform.";

pub struct FixData<'a> {
    span: Span,
    file: &'a Expression,
}

struct CatToOpen;

impl DetectFix for CatToOpen {
    type FixInput<'a> = FixData<'a>;

    fn id(&self) -> &'static str {
        "cat_to_open"
    }

    fn short_description(&self) -> &'static str {
        "`cat` of a single file replaceable with `open --raw`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/open.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["cat"])
            .iter()
            .filter_map(|invocation| {
                let parsed = invocation.parse(&SPEC)?;
                let LineSource::File(file) = LineSource::of(invocation, &parsed.operands)? else {
                    return None;
                };
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("'cat' reading one file");
                Some((
                    detection,
                    FixData {
                        span: invocation.span,
                        file,
                    },
                ))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = format!("open --raw {}", context.expr_text(fix_data.file));
        Some(Fix {
            explanation: format!("Read the file with '{replacement}'").into(),
            replacements: vec![Replacement::new(fix_data.span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &CatToOpen;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
