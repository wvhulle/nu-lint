use std::{borrow::Cow, iter::from_fn, slice::Iter};

use nu_protocol::{
    Span,
    ast::{Expr, Expression, ExternalArgument},
};

use crate::{ast::string::quote_nu_string, context::LintContext};

pub struct ExternalInvocation<'a> {
    pub name: &'a str,
    pub span: Span,
    pub args: &'a [ExternalArgument],
    pub previous: Option<&'a Expression>,
    pub next: Option<&'a Expression>,
}

impl<'a> ExternalInvocation<'a> {
    pub fn parse(&self, spec: &CliSpec) -> Option<ParsedCli<'a>> {
        spec.parse(self.args)
    }

    pub fn next_external_name(&self) -> Option<&'a str> {
        self.next.and_then(external_name)
    }

    pub fn next_command_name(&self, context: &'a LintContext) -> Option<&'a str> {
        command_name(self.next?, context)
    }

    pub fn previous_command_name(&self, context: &'a LintContext) -> Option<&'a str> {
        command_name(self.previous?, context)
    }
}

fn command_name<'a>(expr: &Expression, context: &'a LintContext) -> Option<&'a str> {
    match &expr.expr {
        Expr::Call(call) => Some(context.working_set.get_decl(call.decl_id).name()),
        _ => None,
    }
}

pub fn external_name(expr: &Expression) -> Option<&str> {
    match &expr.expr {
        Expr::ExternalCall(head, _) => literal_content(head),
        _ => None,
    }
}

pub fn literal_content(expr: &Expression) -> Option<&str> {
    match &expr.expr {
        Expr::String(content) | Expr::RawString(content) | Expr::GlobPattern(content, _) => {
            Some(content)
        }
        _ => None,
    }
}

pub fn is_expanded_glob(expr: &Expression) -> bool {
    matches!(&expr.expr, Expr::GlobPattern(content, false) if content.contains(['*', '?', '[']))
}

pub fn value_text<'a>(context: &'a LintContext, expr: &'a Expression) -> Cow<'a, str> {
    match &expr.expr {
        Expr::GlobPattern(bare_word, _) => Cow::Owned(quote_nu_string(bare_word)),
        _ => Cow::Borrowed(context.expr_text(expr)),
    }
}

pub enum LineSource<'a> {
    Piped,
    File(&'a Expression),
}

impl<'a> LineSource<'a> {
    pub fn of(invocation: &ExternalInvocation<'a>, operands: &[&'a Expression]) -> Option<Self> {
        match (invocation.previous, operands) {
            (Some(_), []) => Some(Self::Piped),
            (None, [file]) if !is_expanded_glob(file) => Some(Self::File(file)),
            _ => None,
        }
    }

    pub fn open_prefix(&self, context: &LintContext) -> String {
        match self {
            Self::Piped => String::new(),
            Self::File(file) => format!("open --raw {} | ", context.expr_text(file)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flag {
    pub short: Option<char>,
    pub long: Option<&'static str>,
    pub takes_value: bool,
}

impl Flag {
    pub const fn switch(short: char, long: &'static str) -> Self {
        Self {
            short: Some(short),
            long: Some(long),
            takes_value: false,
        }
    }

    pub const fn short_switch(short: char) -> Self {
        Self {
            short: Some(short),
            long: None,
            takes_value: false,
        }
    }

    pub const fn long_switch(long: &'static str) -> Self {
        Self {
            short: None,
            long: Some(long),
            takes_value: false,
        }
    }

    pub const fn with_value(self) -> Self {
        Self {
            takes_value: true,
            ..self
        }
    }
}

pub struct CliSpec {
    pub flags: &'static [Flag],
    pub numeric_shorthand: Option<Flag>,
}

#[derive(Debug, Clone, Copy)]
pub enum FlagValue<'a> {
    Inline(&'a str),
    Argument(&'a Expression),
}

impl<'a> FlagValue<'a> {
    pub fn literal(&self) -> Option<&'a str> {
        match self {
            Self::Inline(text) => Some(text),
            Self::Argument(expr) => literal_content(expr),
        }
    }

    pub fn nu_text(&self, context: &'a LintContext) -> Cow<'a, str> {
        match self {
            Self::Inline(text) => Cow::Owned(quote_nu_string(text)),
            Self::Argument(expr) => value_text(context, expr),
        }
    }
}

#[derive(Clone, Copy)]
struct ParsedFlag<'a> {
    flag: Flag,
    value: Option<FlagValue<'a>>,
}

#[derive(Clone, Copy)]
enum ParsedArgument<'a> {
    Flag(ParsedFlag<'a>),
    Operand(&'a Expression),
}

impl<'a> ParsedArgument<'a> {
    const fn flag(self) -> Option<ParsedFlag<'a>> {
        match self {
            Self::Flag(flag) => Some(flag),
            Self::Operand(_) => None,
        }
    }

    const fn operand(self) -> Option<&'a Expression> {
        match self {
            Self::Operand(expr) => Some(expr),
            Self::Flag(_) => None,
        }
    }
}

pub struct ParsedCli<'a> {
    flags: Vec<ParsedFlag<'a>>,
    pub operands: Vec<&'a Expression>,
}

impl<'a> ParsedCli<'a> {
    pub fn has(&self, flag: Flag) -> bool {
        self.flags.iter().any(|parsed| parsed.flag == flag)
    }

    pub fn value(&self, flag: Flag) -> Option<FlagValue<'a>> {
        self.values(flag).last()
    }

    pub fn values(&self, flag: Flag) -> impl Iterator<Item = FlagValue<'a>> + '_ {
        self.flags
            .iter()
            .filter(move |parsed| parsed.flag == flag)
            .filter_map(|parsed| parsed.value)
    }
}

impl CliSpec {
    pub fn parse<'a>(&self, args: &'a [ExternalArgument]) -> Option<ParsedCli<'a>> {
        let mut remaining = args.iter();
        let groups: Option<Vec<Vec<ParsedArgument<'a>>>> =
            from_fn(|| Some(self.parse_argument(remaining.next()?, &mut remaining))).collect();
        let arguments: Vec<ParsedArgument<'a>> = groups?.into_iter().flatten().collect();
        Some(ParsedCli {
            flags: arguments
                .iter()
                .copied()
                .filter_map(ParsedArgument::flag)
                .collect(),
            operands: arguments
                .iter()
                .copied()
                .filter_map(ParsedArgument::operand)
                .collect(),
        })
    }

    fn parse_argument<'a>(
        &self,
        argument: &'a ExternalArgument,
        remaining: &mut Iter<'a, ExternalArgument>,
    ) -> Option<Vec<ParsedArgument<'a>>> {
        let ExternalArgument::Regular(expr) = argument else {
            return None;
        };
        match literal_content(expr) {
            Some("--") => remaining.map(operand).collect(),
            Some(word) if word.starts_with("--") => Some(vec![ParsedArgument::Flag(
                self.parse_long(&word[2..], remaining)?,
            )]),
            Some(word) if word.len() > 1 && word.starts_with('-') => {
                self.parse_short_cluster(&word[1..], remaining)
            }
            _ => Some(vec![ParsedArgument::Operand(expr)]),
        }
    }

    fn parse_long<'a>(
        &self,
        word: &'a str,
        remaining: &mut Iter<'a, ExternalArgument>,
    ) -> Option<ParsedFlag<'a>> {
        let (name, attached) = match word.split_once('=') {
            Some((name, value)) => (name, Some(value)),
            None => (word, None),
        };
        let flag = self
            .flags
            .iter()
            .copied()
            .find(|flag| flag.long == Some(name))?;
        attach_value(flag, attached, remaining)
    }

    fn parse_short_cluster<'a>(
        &self,
        cluster: &'a str,
        remaining: &mut Iter<'a, ExternalArgument>,
    ) -> Option<Vec<ParsedArgument<'a>>> {
        if let Some(shorthand) = self.numeric_shorthand
            && cluster.chars().all(|c| c.is_ascii_digit())
        {
            let value = Some(FlagValue::Inline(cluster));
            return Some(vec![ParsedArgument::Flag(ParsedFlag {
                flag: shorthand,
                value,
            })]);
        }

        let value_flag_start = cluster
            .find(|letter| self.short_flag(letter).is_some_and(|flag| flag.takes_value))
            .unwrap_or(cluster.len());
        let (switches, value_flag) = cluster.split_at(value_flag_start);
        let switch_flags: Option<Vec<ParsedArgument<'a>>> = switches
            .chars()
            .map(|letter| {
                let flag = self.short_flag(letter)?;
                Some(ParsedArgument::Flag(ParsedFlag { flag, value: None }))
            })
            .collect();
        let mut parsed = switch_flags?;
        if let Some(letter) = value_flag.chars().next() {
            let flag = self.short_flag(letter)?;
            let attached = Some(&value_flag[letter.len_utf8()..]).filter(|rest| !rest.is_empty());
            parsed.push(ParsedArgument::Flag(attach_value(
                flag, attached, remaining,
            )?));
        }
        Some(parsed)
    }

    fn short_flag(&self, letter: char) -> Option<Flag> {
        self.flags
            .iter()
            .copied()
            .find(|flag| flag.short == Some(letter))
    }
}

fn attach_value<'a>(
    flag: Flag,
    attached: Option<&'a str>,
    remaining: &mut Iter<'a, ExternalArgument>,
) -> Option<ParsedFlag<'a>> {
    let value = match (flag.takes_value, attached) {
        (true, Some(value)) => Some(FlagValue::Inline(value)),
        (true, None) => Some(FlagValue::Argument(regular(remaining.next()?)?)),
        (false, Some(_)) => return None,
        (false, None) => None,
    };
    Some(ParsedFlag { flag, value })
}

const fn regular(argument: &ExternalArgument) -> Option<&Expression> {
    match argument {
        ExternalArgument::Regular(expr) => Some(expr),
        ExternalArgument::Spread(_) => None,
    }
}

fn operand(argument: &ExternalArgument) -> Option<ParsedArgument<'_>> {
    regular(argument).map(ParsedArgument::Operand)
}

#[cfg(test)]
mod tests {
    use super::*;

    const IGNORE_CASE: Flag = Flag::switch('i', "ignore-case");
    const COUNT: Flag = Flag::switch('c', "count");
    const LINES: Flag = Flag::switch('n', "lines").with_value();

    static SPEC: CliSpec = CliSpec {
        flags: &[IGNORE_CASE, COUNT, LINES],
        numeric_shorthand: Some(LINES),
    };

    fn operand_texts<'a>(parsed: &ParsedCli<'a>, context: &'a LintContext) -> Vec<&'a str> {
        parsed
            .operands
            .iter()
            .map(|operand| context.expr_text(operand))
            .collect()
    }

    fn with_parsed(source: &str, check: impl Fn(Option<ParsedCli>, &LintContext)) {
        LintContext::test_with_parsed_source(source, |context| {
            let invocations = context.external_invocations(&["tool"]);
            let invocation = invocations.first().expect("external call");
            check(invocation.parse(&SPEC), &context);
        });
    }

    #[test]
    fn combined_short_switches() {
        with_parsed("^tool -ic pattern", |parsed, context| {
            let parsed = parsed.expect("known flags");
            assert!(parsed.has(IGNORE_CASE));
            assert!(parsed.has(COUNT));
            assert_eq!(operand_texts(&parsed, context), ["pattern"]);
        });
    }

    #[test]
    fn value_forms() {
        for source in [
            "^tool -n 5",
            "^tool -n5",
            "^tool -in5",
            "^tool --lines=5",
            "^tool --lines 5",
            "^tool -5",
        ] {
            with_parsed(source, |parsed, _| {
                let parsed = parsed.expect("known flags");
                assert_eq!(
                    parsed.value(LINES).and_then(|value| value.literal()),
                    Some("5")
                );
                assert_eq!(parsed.operands.len(), 0);
            });
        }
    }

    #[test]
    fn unknown_flags_reject() {
        for source in [
            "^tool -x p",
            "^tool -ix p",
            "^tool --other p",
            "^tool --count=3",
            "^tool -n",
        ] {
            with_parsed(source, |parsed, _| assert!(parsed.is_none(), "{source}"));
        }
    }

    #[test]
    fn spread_rejects() {
        with_parsed("let xs = []; ^tool ...$xs", |parsed, _| {
            assert!(parsed.is_none());
        });
    }

    #[test]
    fn variables_and_quotes_stay_operands() {
        with_parsed(
            r#"let p = "x"; ^tool $p "my file" -- -c"#,
            |parsed, context| {
                let parsed = parsed.expect("known flags");
                assert!(!parsed.has(COUNT));
                assert_eq!(
                    operand_texts(&parsed, context),
                    ["$p", r#""my file""#, "-c"]
                );
            },
        );
    }

    #[test]
    fn pipeline_neighbours() {
        LintContext::test_with_parsed_source(
            "^tool a | ^tool b | lines; ls | each { ^tool c }",
            |context| {
                let invocations = context.external_invocations(&["tool"]);
                let piped: Vec<_> = invocations
                    .iter()
                    .map(|invocation| invocation.previous.is_some())
                    .collect();
                assert_eq!(piped, [false, true, false]);
                assert_eq!(invocations[0].next_external_name(), Some("tool"));
                assert_eq!(invocations[1].next_command_name(&context), Some("lines"));
            },
        );
    }
}
