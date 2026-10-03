use nu_protocol::{Span, ast::Expression};

use crate::{
    LintLevel,
    ast::{
        external::{CliSpec, ExternalInvocation, Flag, LineSource, literal_content, value_text},
        regex::basic_regex_agrees_with_rust_regex,
        string::quote_nu_string,
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const IGNORE_CASE: Flag = Flag::switch('i', "ignore-case");
const INVERT_MATCH: Flag = Flag::switch('v', "invert-match");
const COUNT: Flag = Flag::switch('c', "count");
const FIXED_STRINGS: Flag = Flag::switch('F', "fixed-strings");
const EXTENDED_REGEXP: Flag = Flag::switch('E', "extended-regexp");
const CASE_SENSITIVE: Flag = Flag::switch('s', "case-sensitive");

static GREP_SPEC: CliSpec = CliSpec {
    flags: &[
        IGNORE_CASE,
        INVERT_MATCH,
        COUNT,
        FIXED_STRINGS,
        EXTENDED_REGEXP,
    ],
    numeric_shorthand: None,
};

static RIPGREP_SPEC: CliSpec = CliSpec {
    flags: &[
        IGNORE_CASE,
        INVERT_MATCH,
        COUNT,
        FIXED_STRINGS,
        CASE_SENSITIVE,
    ],
    numeric_shorthand: None,
};

const NOTE: &str = "Filtering lines with 'where' keeps the result as a list of strings, uses the \
                    same regex dialect as '=~' elsewhere in the script and returns an empty list \
                    when nothing matches, while grep exits with code 1 which Nu reports as an \
                    error.";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dialect {
    Basic,
    Extended,
}

enum Matching {
    Literal,
    Regex(Dialect),
}

pub struct LineFilter<'a> {
    span: Span,
    source: LineSource<'a>,
    pattern: &'a Expression,
    matching: Matching,
    ignore_case: bool,
    invert: bool,
    count: bool,
}

impl<'a> LineFilter<'a> {
    fn from_invocation(invocation: &ExternalInvocation<'a>) -> Option<Self> {
        let (spec, default_dialect) = match invocation.name {
            "grep" => (&GREP_SPEC, Dialect::Basic),
            _ => (&RIPGREP_SPEC, Dialect::Extended),
        };
        let parsed = invocation.parse(spec)?;
        let (pattern, inputs) = parsed.operands.split_first()?;
        let source = LineSource::of(invocation, inputs)
            .filter(|source| matches!(source, LineSource::Piped) || invocation.name == "grep")?;
        let matching = if parsed.has(FIXED_STRINGS) {
            Matching::Literal
        } else if parsed.has(EXTENDED_REGEXP) {
            Matching::Regex(Dialect::Extended)
        } else {
            Matching::Regex(default_dialect)
        };

        Some(Self {
            span: invocation.span,
            source,
            pattern,
            matching,
            ignore_case: parsed.has(IGNORE_CASE),
            invert: parsed.has(INVERT_MATCH),
            count: parsed.has(COUNT),
        })
    }

    fn regex_is_portable(&self) -> bool {
        match self.matching {
            Matching::Literal | Matching::Regex(Dialect::Extended) => true,
            Matching::Regex(Dialect::Basic) => {
                literal_content(self.pattern).is_some_and(basic_regex_agrees_with_rust_regex)
            }
        }
    }

    fn condition(&self, context: &LintContext) -> String {
        let pattern = value_text(context, self.pattern);
        match (&self.matching, self.invert) {
            (Matching::Literal, invert) => {
                let flag = if self.ignore_case {
                    " --ignore-case"
                } else {
                    ""
                };
                let negation = if invert { "not " } else { "" };
                format!("{negation}($it | str contains{flag} {pattern})")
            }
            (Matching::Regex(_), invert) => {
                let operator = if invert { "!~" } else { "=~" };
                let pattern = if self.ignore_case {
                    self.case_insensitive_regex(context)
                } else {
                    pattern.into_owned()
                };
                format!("$it {operator} {pattern}")
            }
        }
    }

    fn case_insensitive_regex(&self, context: &LintContext) -> String {
        literal_content(self.pattern).map_or_else(
            || format!("('(?i)' + {})", context.expr_text(self.pattern)),
            |content| quote_nu_string(&format!("(?i){content}")),
        )
    }

    fn to_fix(&self, context: &LintContext) -> Fix {
        let counting = if self.count { " | length" } else { "" };
        let replacement = format!(
            "{}lines | where {}{counting}",
            self.source.open_prefix(context),
            self.condition(context)
        );
        Fix {
            explanation: format!("Filter lines with '{replacement}'").into(),
            replacements: vec![Replacement::new(self.span, replacement)],
        }
    }
}

struct GrepToWhere;

impl DetectFix for GrepToWhere {
    type FixInput<'a> = LineFilter<'a>;

    fn id(&self) -> &'static str {
        "grep_to_where"
    }

    fn short_description(&self) -> &'static str {
        "`grep` filtering lines replaceable with `where`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/where.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["grep", "rg"])
            .iter()
            .filter_map(|invocation| {
                let filter = LineFilter::from_invocation(invocation)?;
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label(format!("'{}' used as a line filter", invocation.name));
                Some((detection, filter))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, filter: &Self::FixInput<'_>) -> Option<Fix> {
        filter.regex_is_portable().then(|| filter.to_fix(context))
    }
}

pub static RULE: &dyn Rule = &GrepToWhere;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
