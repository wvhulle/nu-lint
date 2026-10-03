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

const NOTE: &str = "'lines | reverse' returns the reversed lines as a list of strings that the \
                    rest of the pipeline can process without splitting text again.";

struct TacToReverse;

impl DetectFix for TacToReverse {
    type FixInput<'a> = Replacement;

    fn id(&self) -> &'static str {
        "tac_to_reverse"
    }

    fn short_description(&self) -> &'static str {
        "`tac` replaceable with `lines | reverse`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/reverse.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["tac"])
            .iter()
            .filter_map(|invocation| {
                let parsed = invocation.parse(&SPEC)?;
                let source = LineSource::of(invocation, &parsed.operands)?;
                let replacement = Replacement::new(
                    invocation.span,
                    format!("{}lines | reverse", source.open_prefix(context)),
                );
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("'tac' reversing lines");
                Some((detection, replacement))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, replacement: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!("Reverse lines with '{}'", replacement.replacement_text).into(),
            replacements: vec![replacement.clone()],
        })
    }
}

pub static RULE: &dyn Rule = &TacToReverse;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
