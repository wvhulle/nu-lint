use nu_protocol::Span;

use crate::{
    LintLevel,
    ast::{
        external::{CliSpec, ExternalInvocation, Flag, literal_content},
        string::quote_nu_string,
    },
    context::LintContext,
    dsl::date::chrono_compatible_format,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const UTC: Flag = Flag::switch('u', "utc");

static SPEC: CliSpec = CliSpec {
    flags: &[UTC],
    numeric_shorthand: None,
};

const NOTE: &str = "'date now' returns a datetime value that can be compared, shifted and \
                    formatted later, instead of text in the format of the current locale.";

pub struct CurrentDate<'a> {
    span: Span,
    utc: bool,
    format: Option<&'a str>,
}

impl<'a> CurrentDate<'a> {
    fn from_invocation(invocation: &ExternalInvocation<'a>) -> Option<Self> {
        let parsed = invocation.parse(&SPEC)?;
        let format = match parsed.operands.as_slice() {
            [] => None,
            [operand] => Some(chrono_compatible_format(literal_content(operand)?)?),
            _ => return None,
        };
        Some(Self {
            span: invocation.span,
            utc: parsed.has(UTC),
            format,
        })
    }

    fn to_nu(&self) -> String {
        let timezone = if self.utc {
            " | date to-timezone UTC"
        } else {
            ""
        };
        let formatting = self.format.map_or(String::new(), |format| {
            format!(" | format date {}", quote_nu_string(format))
        });
        format!("date now{timezone}{formatting}")
    }
}

struct DateToBuiltin;

impl DetectFix for DateToBuiltin {
    type FixInput<'a> = CurrentDate<'a>;

    fn id(&self) -> &'static str {
        "date_to_builtin"
    }

    fn short_description(&self) -> &'static str {
        "External `date` replaceable with `date now` and `format date`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/date_now.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["date"])
            .iter()
            .filter_map(|invocation| {
                let date = CurrentDate::from_invocation(invocation)?;
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("external 'date' prints text");
                Some((detection, date))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, date: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = date.to_nu();
        Some(Fix {
            explanation: format!("Use '{replacement}'").into(),
            replacements: vec![Replacement::new(date.span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &DateToBuiltin;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
