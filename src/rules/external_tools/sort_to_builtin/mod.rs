use nu_protocol::{
    Span,
    ast::{Expr, Expression},
};

use crate::{
    LintLevel,
    ast::external::{CliSpec, ExternalInvocation, Flag, LineSource, external_name},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const REVERSE: Flag = Flag::switch('r', "reverse");
const IGNORE_CASE: Flag = Flag::switch('f', "ignore-case");
const UNIQUE: Flag = Flag::switch('u', "unique");

static SPEC: CliSpec = CliSpec {
    flags: &[REVERSE, IGNORE_CASE, UNIQUE],
    numeric_shorthand: None,
};

const NOTE: &str = "'lines | sort' returns a sorted list of strings. Nu compares by code point \
                    like 'LC_ALL=C sort', so the order no longer depends on the locale of the \
                    machine running the script.";

enum Deduplication {
    None,
    CaseSensitive,
    CaseInsensitive,
}

fn is_plain_uniq(expr: &Expression) -> bool {
    external_name(expr) == Some("uniq")
        && matches!(&expr.expr, Expr::ExternalCall(_, args) if args.is_empty())
}

pub struct LineSort<'a> {
    span: Span,
    source: LineSource<'a>,
    reverse: bool,
    ignore_case: bool,
    deduplication: Deduplication,
}

impl<'a> LineSort<'a> {
    fn from_invocation(invocation: &ExternalInvocation<'a>) -> Option<Self> {
        let parsed = invocation.parse(&SPEC)?;
        let source = LineSource::of(invocation, &parsed.operands)?;
        let ignore_case = parsed.has(IGNORE_CASE);
        let piped_uniq = invocation.next.filter(|next| is_plain_uniq(next));
        let deduplication = match (parsed.has(UNIQUE), ignore_case, piped_uniq) {
            (true, true, _) => Deduplication::CaseInsensitive,
            (true, false, _) | (false, _, Some(_)) => Deduplication::CaseSensitive,
            (false, _, None) => Deduplication::None,
        };
        let end = piped_uniq.map_or(invocation.span.end, |uniq| uniq.span.end);
        Some(Self {
            span: Span::new(invocation.span.start, end),
            source,
            reverse: parsed.has(REVERSE),
            ignore_case,
            deduplication,
        })
    }

    fn to_fix(&self, context: &LintContext) -> Fix {
        let reverse = if self.reverse { " --reverse" } else { "" };
        let ignore_case = if self.ignore_case {
            " --ignore-case"
        } else {
            ""
        };
        let deduplication = match self.deduplication {
            Deduplication::None => "",
            Deduplication::CaseSensitive => " | uniq",
            Deduplication::CaseInsensitive => " | uniq --ignore-case",
        };
        let replacement = format!(
            "{}lines | sort{reverse}{ignore_case}{deduplication}",
            self.source.open_prefix(context)
        );
        Fix {
            explanation: format!("Sort lines with '{replacement}'").into(),
            replacements: vec![Replacement::new(self.span, replacement)],
        }
    }
}

struct SortToBuiltin;

impl DetectFix for SortToBuiltin {
    type FixInput<'a> = LineSort<'a>;

    fn id(&self) -> &'static str {
        "sort_to_builtin"
    }

    fn short_description(&self) -> &'static str {
        "External `sort` of lines replaceable with built-in `sort`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/sort.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["sort"])
            .iter()
            .filter_map(|invocation| {
                let sort = LineSort::from_invocation(invocation)?;
                let detection = Detection::from_global_span(NOTE, sort.span)
                    .with_primary_label("external 'sort' of lines");
                Some((detection, sort))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, sort: &Self::FixInput<'_>) -> Option<Fix> {
        Some(sort.to_fix(context))
    }
}

pub static RULE: &dyn Rule = &SortToBuiltin;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
