use std::borrow::Cow;

use nu_protocol::Span;

use crate::{
    LintLevel,
    ast::external::{CliSpec, ExternalInvocation, Flag, FlagValue, LineSource},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const LINES: Flag = Flag::switch('n', "lines").with_value();

static SPEC: CliSpec = CliSpec {
    flags: &[LINES],
    numeric_shorthand: Some(LINES),
};

const DEFAULT_LINE_COUNT: &str = "10";

const NOTE: &str = "'lines | first' and 'lines | last' return a list of strings that the rest of \
                    the pipeline can process without splitting text again.";

enum Selection<'a> {
    First(Cow<'a, str>),
    Last(Cow<'a, str>),
    SkipLeading(u64),
}

pub struct LineSlice<'a> {
    span: Span,
    source: LineSource<'a>,
    selection: Selection<'a>,
}

impl<'a> LineSlice<'a> {
    fn from_invocation(
        invocation: &ExternalInvocation<'a>,
        context: &'a LintContext,
    ) -> Option<Self> {
        let parsed = invocation.parse(&SPEC)?;
        let source = LineSource::of(invocation, &parsed.operands)?;
        let count = parsed.value(LINES);
        let selection = match invocation.name {
            "head" => Selection::First(Self::positive_count(count, context)?),
            _ => match count
                .and_then(|value| value.literal())
                .and_then(|text| text.strip_prefix('+'))
            {
                Some(start) => Selection::SkipLeading(start.parse::<u64>().ok()?.saturating_sub(1)),
                None => Selection::Last(Self::positive_count(count, context)?),
            },
        };
        Some(Self {
            span: invocation.span,
            source,
            selection,
        })
    }

    fn positive_count(
        count: Option<FlagValue<'a>>,
        context: &'a LintContext,
    ) -> Option<Cow<'a, str>> {
        let Some(count) = count else {
            return Some(Cow::Borrowed(DEFAULT_LINE_COUNT));
        };
        match count.literal() {
            Some(literal) => literal.parse::<u64>().ok().map(|_| Cow::Borrowed(literal)),
            None => Some(count.text(context)),
        }
    }

    fn to_fix(&self, context: &LintContext) -> Fix {
        let selection = match &self.selection {
            Selection::First(count) => format!("first {count}"),
            Selection::Last(count) => format!("last {count}"),
            Selection::SkipLeading(skipped) => format!("skip {skipped}"),
        };
        let replacement = format!("{}lines | {selection}", self.source.open_prefix(context));
        Fix {
            explanation: format!("Select lines with '{replacement}'").into(),
            replacements: vec![Replacement::new(self.span, replacement)],
        }
    }
}

struct HeadTailToFirstLast;

impl DetectFix for HeadTailToFirstLast {
    type FixInput<'a> = LineSlice<'a>;

    fn id(&self) -> &'static str {
        "head_tail_to_first_last"
    }

    fn short_description(&self) -> &'static str {
        "`head` or `tail` on lines replaceable with `first` or `last`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/first.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["head", "tail"])
            .iter()
            .filter_map(|invocation| {
                let slice = LineSlice::from_invocation(invocation, context)?;
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label(format!("'{}' selecting lines", invocation.name));
                Some((detection, slice))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, slice: &Self::FixInput<'_>) -> Option<Fix> {
        Some(slice.to_fix(context))
    }
}

pub static RULE: &dyn Rule = &HeadTailToFirstLast;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
