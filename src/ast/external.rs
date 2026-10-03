use std::{borrow::Cow, iter::once, slice::Iter};

use nu_protocol::{
    Span,
    ast::{Block, Expr, Expression, ExternalArgument, Traverse},
};

use crate::{
    ast::{expression::ExpressionExt, string::quote_nu_string},
    context::LintContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSource {
    Piped,
    Leading,
}

pub struct ExternalInvocation<'a> {
    pub name: &'a str,
    pub span: Span,
    pub args: &'a [ExternalArgument],
    pub input: InputSource,
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
        match &self.next?.expr {
            Expr::Call(call) => Some(context.working_set.get_decl(call.decl_id).name()),
            _ => None,
        }
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
        match (invocation.input, operands) {
            (InputSource::Piped, []) => Some(Self::Piped),
            (InputSource::Leading, [file]) if !is_expanded_glob(file) => Some(Self::File(file)),
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

impl LintContext<'_> {
    pub fn external_invocations<'a>(&'a self, names: &[&str]) -> Vec<ExternalInvocation<'a>> {
        let mut nested_block_ids = Vec::new();
        self.ast.flat_map(
            self.working_set,
            &|expr| expr.extract_block_id().into_iter().collect(),
            &mut nested_block_ids,
        );
        nested_block_ids.sort_unstable();
        nested_block_ids.dedup();
        let nested_blocks = nested_block_ids
            .into_iter()
            .map(|block_id| -> &'a Block { self.working_set.get_block(block_id) });

        once(self.ast)
            .chain(nested_blocks)
            .flat_map(|block| &block.pipelines)
            .flat_map(|pipeline| {
                let elements = &pipeline.elements;
                elements
                    .iter()
                    .enumerate()
                    .filter_map(move |(index, element)| {
                        let Expr::ExternalCall(head, args) = &element.expr.expr else {
                            return None;
                        };
                        let name = literal_content(head)?;
                        names.contains(&name).then(|| ExternalInvocation {
                            name,
                            span: element.expr.span,
                            args,
                            input: if index == 0 {
                                InputSource::Leading
                            } else {
                                InputSource::Piped
                            },
                            previous: index
                                .checked_sub(1)
                                .and_then(|previous| elements.get(previous))
                                .map(|previous| &previous.expr),
                            next: elements.get(index + 1).map(|next| &next.expr),
                        })
                    })
            })
            .collect()
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

    pub fn text(&self, context: &'a LintContext) -> Cow<'a, str> {
        match self {
            Self::Inline(text) => Cow::Borrowed(text),
            Self::Argument(expr) => Cow::Borrowed(context.expr_text(expr)),
        }
    }
}

struct ParsedFlag<'a> {
    flag: Flag,
    value: Option<FlagValue<'a>>,
}

pub struct ParsedCli<'a> {
    flags: Vec<ParsedFlag<'a>>,
    pub operands: Vec<&'a Expression>,
}

impl<'a> ParsedCli<'a> {
    pub fn has(&self, flag: Flag) -> bool {
        self.flags.iter().any(|parsed| parsed.flag == flag)
    }

    pub const fn has_no_flags(&self) -> bool {
        self.flags.is_empty()
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
        let mut parsed = ParsedCli {
            flags: Vec::new(),
            operands: Vec::new(),
        };
        let mut remaining = args.iter();
        let mut only_operands_follow = false;

        while let Some(argument) = remaining.next() {
            let ExternalArgument::Regular(expr) = argument else {
                return None;
            };
            match literal_content(expr).filter(|_| !only_operands_follow) {
                Some("--") => only_operands_follow = true,
                Some(word) if word.starts_with("--") => {
                    let flag = self.parse_long(&word[2..], &mut remaining)?;
                    parsed.flags.push(flag);
                }
                Some(word) if word.len() > 1 && word.starts_with('-') => {
                    parsed
                        .flags
                        .extend(self.parse_short_cluster(&word[1..], &mut remaining)?);
                }
                _ => parsed.operands.push(expr),
            }
        }

        Some(parsed)
    }

    fn parse_long<'a>(
        &self,
        word: &'a str,
        remaining: &mut Iter<'a, ExternalArgument>,
    ) -> Option<ParsedFlag<'a>> {
        let (name, attached) = word
            .split_once('=')
            .map_or((word, None), |(name, value)| (name, Some(value)));
        let flag = *self.flags.iter().find(|flag| flag.long == Some(name))?;
        let value = match (flag.takes_value, attached) {
            (true, Some(value)) => Some(FlagValue::Inline(value)),
            (true, None) => Some(next_value(remaining)?),
            (false, Some(_)) => return None,
            (false, None) => None,
        };
        Some(ParsedFlag { flag, value })
    }

    fn parse_short_cluster<'a>(
        &self,
        cluster: &'a str,
        remaining: &mut Iter<'a, ExternalArgument>,
    ) -> Option<Vec<ParsedFlag<'a>>> {
        if let Some(shorthand) = self.numeric_shorthand
            && cluster.chars().all(|c| c.is_ascii_digit())
        {
            return Some(vec![ParsedFlag {
                flag: shorthand,
                value: Some(FlagValue::Inline(cluster)),
            }]);
        }

        let mut flags = Vec::new();
        for (offset, letter) in cluster.char_indices() {
            let flag = *self.flags.iter().find(|flag| flag.short == Some(letter))?;
            if flag.takes_value {
                let attached = &cluster[offset + letter.len_utf8()..];
                let value = if attached.is_empty() {
                    next_value(remaining)?
                } else {
                    FlagValue::Inline(attached)
                };
                flags.push(ParsedFlag {
                    flag,
                    value: Some(value),
                });
                return Some(flags);
            }
            flags.push(ParsedFlag { flag, value: None });
        }
        Some(flags)
    }
}

fn next_value<'a>(remaining: &mut Iter<'a, ExternalArgument>) -> Option<FlagValue<'a>> {
    match remaining.next()? {
        ExternalArgument::Regular(expr) => Some(FlagValue::Argument(expr)),
        ExternalArgument::Spread(_) => None,
    }
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
    fn input_source_and_neighbours() {
        LintContext::test_with_parsed_source(
            "^tool a | ^tool b | lines; ls | each { ^tool c }",
            |context| {
                let invocations = context.external_invocations(&["tool"]);
                let inputs: Vec<_> = invocations.iter().map(|i| i.input).collect();
                assert_eq!(
                    inputs,
                    [
                        InputSource::Leading,
                        InputSource::Piped,
                        InputSource::Leading
                    ]
                );
                assert_eq!(invocations[0].next_external_name(), Some("tool"));
                assert_eq!(invocations[1].next_command_name(&context), Some("lines"));
            },
        );
    }
}
