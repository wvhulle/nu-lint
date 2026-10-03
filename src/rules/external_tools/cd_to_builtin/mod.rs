use crate::{
    LintLevel,
    ast::external::{CliSpec, Flag},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const PHYSICAL: Flag = Flag::switch('P', "physical");
const LOGICAL: Flag = Flag::switch('L', "logical");

static SPEC: CliSpec = CliSpec {
    flags: &[PHYSICAL, LOGICAL],
    numeric_shorthand: None,
};

const NOTE: &str = "An external 'cd' runs in a child process and cannot change the directory of \
                    the script. Most systems do not even have a 'cd' binary.";

struct CdToBuiltin;

impl DetectFix for CdToBuiltin {
    type FixInput<'a> = Option<Replacement>;

    fn id(&self) -> &'static str {
        "cd_to_builtin"
    }

    fn short_description(&self) -> &'static str {
        "External `cd` replaceable with built-in `cd`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/cd.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["cd"])
            .iter()
            .map(|invocation| {
                let replacement = invocation
                    .parse(&SPEC)
                    .filter(|parsed| parsed.operands.len() <= 1)
                    .map(|parsed| {
                        let physical = if parsed.has(PHYSICAL) {
                            " --physical"
                        } else {
                            ""
                        };
                        let target = parsed.operands.first().map_or(String::new(), |path| {
                            format!(" {}", context.expr_text(path))
                        });
                        Replacement::new(invocation.span, format!("cd{physical}{target}"))
                    });
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("external 'cd' has no effect on the script");
                (detection, replacement)
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, replacement: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = replacement.as_ref()?;
        Some(Fix {
            explanation: format!("Use the built-in '{}'", replacement.replacement_text).into(),
            replacements: vec![replacement.clone()],
        })
    }
}

pub static RULE: &dyn Rule = &CdToBuiltin;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
