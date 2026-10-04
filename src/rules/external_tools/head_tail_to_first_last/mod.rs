use nu_protocol::{Span, ast::Expression};

use crate::{
    LintLevel,
    ast::external::{CliSpec, ExternalInvocation, Flag, FlagValue, LineSource},
    context::LintContext,
    dsl::arguments::tail_start_line,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const LINES: Flag = Flag::switch('n', "lines").with_value();

static SPEC: CliSpec = CliSpec {
    flags: &[LINES],
    numeric_shorthand: Some(LINES),
};

const DEFAULT_LINE_COUNT: u64 = 10;

const NOTE: &str = "'lines | first' and 'lines | last' return a list of strings that the rest of \
                    the pipeline can process without splitting text again.";

enum LineCount<'a> {
    Literal(u64),
    Dynamic(&'a Expression),
}

impl<'a> LineCount<'a> {
    fn parse(count: Option<FlagValue<'a>>) -> Option<Self> {
        let Some(count) = count else {
            return Some(Self::Literal(DEFAULT_LINE_COUNT));
        };
        match (count, count.literal()) {
            (_, Some(literal)) => literal.parse().ok().map(Self::Literal),
            (FlagValue::Argument(expr), None) => Some(Self::Dynamic(expr)),
            (FlagValue::Inline(_), None) => None,
        }
    }

    fn text(&self, context: &LintContext) -> String {
        match self {
            Self::Literal(count) => count.to_string(),
            Self::Dynamic(expr) => context.expr_text(expr).to_string(),
        }
    }
}

fn tail_start(count: Option<FlagValue>) -> Option<u64> {
    tail_start_line(count?.literal()?)
}

enum Selection<'a> {
    First(LineCount<'a>),
    Last(LineCount<'a>),
    SkipLeading(u64),
}

pub struct LineSlice<'a> {
    span: Span,
    source: LineSource<'a>,
    selection: Selection<'a>,
}

impl<'a> LineSlice<'a> {
    fn from_invocation(invocation: &ExternalInvocation<'a>) -> Option<Self> {
        let parsed = invocation.parse(&SPEC)?;
        let source = LineSource::of(invocation, &parsed.operands)?;
        let count = parsed.value(LINES);
        let selection = match (invocation.name, tail_start(count)) {
            ("head", _) => Selection::First(LineCount::parse(count)?),
            (_, Some(start)) => Selection::SkipLeading(start.saturating_sub(1)),
            (_, None) => Selection::Last(LineCount::parse(count)?),
        };
        Some(Self {
            span: invocation.span,
            source,
            selection,
        })
    }

    fn to_fix(&self, context: &LintContext) -> Fix {
        let selection = match &self.selection {
            Selection::First(count) => format!("first {}", count.text(context)),
            Selection::Last(count) => format!("last {}", count.text(context)),
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
                let slice = LineSlice::from_invocation(invocation)?;
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
