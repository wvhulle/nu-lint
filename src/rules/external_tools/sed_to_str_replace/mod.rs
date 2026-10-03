use nu_protocol::Span;

use crate::{
    LintLevel,
    ast::{
        external::{CliSpec, ExternalInvocation, LineSource, literal_content},
        string::quote_nu_string,
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

static SPEC: CliSpec = CliSpec {
    flags: &[],
    numeric_shorthand: None,
};

const BASIC_REGEX_SPECIAL_CHARS: &[char] = &['.', '*', '[', ']', '^', '$', '\\', '\n'];
const REPLACEMENT_SPECIAL_CHARS: &[char] = &['&', '\\', '\n'];

const NOTE: &str = "'str replace --all' substitutes literal text without regex escaping and \
                    without starting an external process.";

struct Substitution<'a> {
    find: &'a str,
    replace: &'a str,
}

impl<'a> Substitution<'a> {
    fn parse_global_literal(script: &'a str) -> Option<Self> {
        let body = script.strip_prefix('s')?;
        let delimiter = body.chars().next().filter(|c| !c.is_alphanumeric())?;
        let mut parts = body[delimiter.len_utf8()..].split(delimiter);
        let (find, replace, flags) = (parts.next()?, parts.next()?, parts.next()?);
        let is_literal = !find.is_empty()
            && !find.contains(BASIC_REGEX_SPECIAL_CHARS)
            && !replace.contains(REPLACEMENT_SPECIAL_CHARS);
        (parts.next().is_none() && flags == "g" && is_literal).then_some(Self { find, replace })
    }
}

pub struct TextSubstitution<'a> {
    span: Span,
    source: LineSource<'a>,
    substitution: Substitution<'a>,
}

impl<'a> TextSubstitution<'a> {
    fn from_invocation(invocation: &ExternalInvocation<'a>) -> Option<Self> {
        let parsed = invocation.parse(&SPEC)?;
        let (script, inputs) = parsed.operands.split_first()?;
        Some(Self {
            span: invocation.span,
            source: LineSource::of(invocation, inputs)?,
            substitution: Substitution::parse_global_literal(literal_content(script)?)?,
        })
    }

    fn to_fix(&self, context: &LintContext) -> Fix {
        let replacement = format!(
            "{}str replace --all {} {}",
            self.source.open_prefix(context),
            quote_nu_string(self.substitution.find),
            quote_nu_string(self.substitution.replace)
        );
        Fix {
            explanation: format!("Substitute text with '{replacement}'").into(),
            replacements: vec![Replacement::new(self.span, replacement)],
        }
    }
}

struct SedToStrReplace;

impl DetectFix for SedToStrReplace {
    type FixInput<'a> = TextSubstitution<'a>;

    fn id(&self) -> &'static str {
        "sed_to_str_replace"
    }

    fn short_description(&self) -> &'static str {
        "`sed` literal substitution replaceable with `str replace`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/str_replace.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["sed"])
            .iter()
            .filter_map(|invocation| {
                let substitution = TextSubstitution::from_invocation(invocation)?;
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("'sed' replacing literal text");
                Some((detection, substitution))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, substitution: &Self::FixInput<'_>) -> Option<Fix> {
        Some(substitution.to_fix(context))
    }
}

pub static RULE: &dyn Rule = &SedToStrReplace;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
