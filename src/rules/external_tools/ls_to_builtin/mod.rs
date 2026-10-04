use std::iter::once;

use nu_protocol::Span;

use crate::{
    LintLevel,
    ast::external::{CliSpec, Flag, ParsedCli, is_expanded_glob},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const ALL: Flag = Flag::switch('a', "all");
const ALMOST_ALL: Flag = Flag::switch('A', "almost-all");
const LONG: Flag = Flag::short_switch('l');
const HUMAN_READABLE: Flag = Flag::switch('h', "human-readable");
const BY_TIME: Flag = Flag::short_switch('t');
const BY_SIZE: Flag = Flag::short_switch('S');
const REVERSE: Flag = Flag::switch('r', "reverse");
const DIRECTORY: Flag = Flag::switch('d', "directory");

static SPEC: CliSpec = CliSpec {
    flags: &[
        ALL,
        ALMOST_ALL,
        LONG,
        HUMAN_READABLE,
        BY_TIME,
        BY_SIZE,
        REVERSE,
        DIRECTORY,
    ],
    numeric_shorthand: None,
};

const NOTE: &str = "The built-in 'ls' returns a table with name, type, size and modified columns, \
                    so the listing can be filtered and sorted without parsing text.";

pub struct FixData<'a> {
    span: Span,
    parsed: ParsedCli<'a>,
}

fn translate(parsed: &ParsedCli, context: &LintContext) -> String {
    let hidden = (parsed.has(ALL) || parsed.has(ALMOST_ALL)).then_some("--all");
    let directory = parsed.has(DIRECTORY).then_some("--directory");
    let paths = parsed.operands.iter().map(|path| context.expr_text(path));
    let words: Vec<&str> = once("ls")
        .chain(hidden)
        .chain(directory)
        .chain(paths)
        .collect();
    let command = words.join(" ");

    let sort_column = if parsed.has(BY_TIME) {
        Some("modified")
    } else if parsed.has(BY_SIZE) {
        Some("size")
    } else {
        None
    };
    let ordering = match (sort_column, parsed.has(REVERSE)) {
        (Some(column), false) => format!(" | sort-by {column} --reverse"),
        (Some(column), true) => format!(" | sort-by {column}"),
        (None, true) => " | reverse".to_string(),
        (None, false) => String::new(),
    };
    format!("{command}{ordering}")
}

struct LsToBuiltin;

impl DetectFix for LsToBuiltin {
    type FixInput<'a> = FixData<'a>;

    fn id(&self) -> &'static str {
        "ls_to_builtin"
    }

    fn short_description(&self) -> &'static str {
        "External `ls` replaceable with built-in `ls`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/ls.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context
            .external_invocations(&["ls"])
            .iter()
            .filter(|invocation| invocation.next_external_name().is_none())
            .filter_map(|invocation| {
                let parsed = invocation.parse(&SPEC)?;
                if parsed.operands.iter().copied().any(is_expanded_glob) {
                    return None;
                }
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("external 'ls' returns text");
                Some((
                    detection,
                    FixData {
                        span: invocation.span,
                        parsed,
                    },
                ))
            })
            .collect()
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = translate(&fix_data.parsed, context);
        Some(Fix {
            explanation: format!("List with '{replacement}'").into(),
            replacements: vec![Replacement::new(fix_data.span, replacement)],
        })
    }
}

pub static RULE: &dyn Rule = &LsToBuiltin;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
