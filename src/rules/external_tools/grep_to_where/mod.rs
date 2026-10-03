use nu_protocol::{Span, ast::Expression};

use crate::{
    LintLevel,
    ast::{
        external::{
            CliSpec, ExternalInvocation, Flag, InputSource, ParsedCli, is_expanded_glob,
            literal_content, value_text,
        },
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
    file: Option<&'a Expression>,
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
        let (pattern, file) = Self::pattern_and_file(invocation, &parsed)?;
        let matching = if parsed.has(FIXED_STRINGS) {
            Matching::Literal
        } else if parsed.has(EXTENDED_REGEXP) {
            Matching::Regex(Dialect::Extended)
        } else {
            Matching::Regex(default_dialect)
        };

        Some(Self {
            span: invocation.span,
            file,
            pattern,
            matching,
            ignore_case: parsed.has(IGNORE_CASE),
            invert: parsed.has(INVERT_MATCH),
            count: parsed.has(COUNT),
        })
    }

    fn pattern_and_file(
        invocation: &ExternalInvocation<'a>,
        parsed: &ParsedCli<'a>,
    ) -> Option<(&'a Expression, Option<&'a Expression>)> {
        match (invocation.input, parsed.operands.as_slice()) {
            (InputSource::Piped, [pattern]) => Some((pattern, None)),
            (InputSource::Leading, [pattern, file])
                if invocation.name == "grep" && !is_expanded_glob(file) =>
            {
                Some((pattern, Some(file)))
            }
            _ => None,
        }
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
        let source = self.file.map_or_else(String::new, |file| {
            format!("open --raw {} | ", context.expr_text(file))
        });
        let counting = if self.count { " | length" } else { "" };
        let replacement = format!(
            "{source}lines | where {}{counting}",
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
