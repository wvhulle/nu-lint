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

impl DetectFix for WcToLength {
    type FixInput<'a> = Replacement;

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
                let replacement = Replacement::new(
                    invocation.span,
                    format!("{}lines | length", source.open_prefix(context)),
                );
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("'wc -l' counting lines");
                Some((detection, replacement))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, replacement: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!("Count lines with '{}'", replacement.replacement_text).into(),
            replacements: vec![replacement.clone()],
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
