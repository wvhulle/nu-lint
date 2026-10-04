use nu_protocol::{Span, ast::Expression};

use crate::{
    LintLevel,
    ast::external::{CliSpec, ExternalInvocation, Flag},
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

pub struct FixData<'a> {
    span: Span,
    physical: bool,
    target: Option<&'a Expression>,
}

impl<'a> FixData<'a> {
    fn from_invocation(invocation: &ExternalInvocation<'a>) -> Option<Self> {
        let parsed = invocation.parse(&SPEC)?;
        let target = match parsed.operands.as_slice() {
            [] => None,
            [target] => Some(*target),
            _ => return None,
        };
        Some(Self {
            span: invocation.span,
            physical: parsed.has(PHYSICAL),
            target,
        })
    }
}

struct CdToBuiltin;

impl DetectFix for CdToBuiltin {
    type FixInput<'a> = Option<FixData<'a>>;

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
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("external 'cd' has no effect on the script");
                (detection, FixData::from_invocation(invocation))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let fix_data = fix_data.as_ref()?;
        let physical = if fix_data.physical { " --physical" } else { "" };
        let target = fix_data.target.map_or(String::new(), |path| {
            format!(" {}", context.expr_text(path))
        });
        let replacement = format!("cd{physical}{target}");
        Some(Fix {
            explanation: format!("Use the built-in '{replacement}'").into(),
            replacements: vec![Replacement::new(fix_data.span, replacement)],
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
