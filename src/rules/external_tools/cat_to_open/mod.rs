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

struct CatToOpen;

impl DetectFix for CatToOpen {
    type FixInput<'a> = Replacement;

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
                let replacement = Replacement::new(
                    invocation.span,
                    format!("open --raw {}", context.expr_text(file)),
                );
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("'cat' reading one file");
                Some((detection, replacement))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, replacement: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!("Read the file with '{}'", replacement.replacement_text).into(),
            replacements: vec![replacement.clone()],
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
