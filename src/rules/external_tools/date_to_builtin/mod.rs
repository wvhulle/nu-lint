use crate::{
    LintLevel,
    ast::{
        external::{CliSpec, ExternalInvocation, Flag, literal_content},
        string::quote_nu_string,
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const UTC: Flag = Flag::switch('u', "utc");

static SPEC: CliSpec = CliSpec {
    flags: &[UTC],
    numeric_shorthand: None,
};

const SHARED_CONVERSIONS: &str = "YmdHMSybBaAjeIpzsFTDRuwUWVGghklPnt%";
const PADDING_MODIFIERS: &str = "-_0:";

const NOTE: &str = "'date now' returns a datetime value that can be compared, shifted and \
                    formatted later, instead of text in the format of the current locale.";

fn formats_identically_in_chrono(format: &str) -> bool {
    let mut characters = format.chars();
    while let Some(character) = characters.next() {
        if character != '%' {
            continue;
        }
        let conversion = characters.find(|next| !PADDING_MODIFIERS.contains(*next));
        if !conversion.is_some_and(|letter| SHARED_CONVERSIONS.contains(letter)) {
            return false;
        }
    }
    true
}

fn builtin_equivalent(invocation: &ExternalInvocation) -> Option<String> {
    let parsed = invocation.parse(&SPEC)?;
    let timezone = if parsed.has(UTC) {
        " | date to-timezone UTC"
    } else {
        ""
    };
    let formatting = match parsed.operands.as_slice() {
        [] => String::new(),
        [format] => {
            let format = literal_content(format)?.strip_prefix('+')?;
            if !formats_identically_in_chrono(format) {
                return None;
            }
            format!(" | format date {}", quote_nu_string(format))
        }
        _ => return None,
    };
    Some(format!("date now{timezone}{formatting}"))
}

struct DateToBuiltin;

impl DetectFix for DateToBuiltin {
    type FixInput<'a> = Replacement;

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
                let equivalent = builtin_equivalent(invocation)?;
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("external 'date' prints text");
                Some((detection, Replacement::new(invocation.span, equivalent)))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, replacement: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!("Use '{}'", replacement.replacement_text).into(),
            replacements: vec![replacement.clone()],
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
