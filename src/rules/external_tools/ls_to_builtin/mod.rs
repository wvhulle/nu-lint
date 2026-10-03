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

fn translate(parsed: &ParsedCli, context: &LintContext) -> String {
    let mut command = String::from("ls");
    if parsed.has(ALL) || parsed.has(ALMOST_ALL) {
        command.push_str(" --all");
    }
    if parsed.has(DIRECTORY) {
        command.push_str(" --directory");
    }
    for path in &parsed.operands {
        command.push(' ');
        command.push_str(context.expr_text(path));
    }

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
    type FixInput<'a> = Replacement;

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
                if parsed.operands.iter().any(|path| is_expanded_glob(path)) {
                    return None;
                }
                let replacement = Replacement::new(invocation.span, translate(&parsed, context));
                let detection = Detection::from_global_span(NOTE, invocation.span)
                    .with_primary_label("external 'ls' returns text");
                Some((detection, replacement))
            })
            .collect()
    }

    fn fix(&self, _context: &LintContext, replacement: &Self::FixInput<'_>) -> Option<Fix> {
        Some(Fix {
            explanation: format!("List with '{}'", replacement.replacement_text).into(),
            replacements: vec![replacement.clone()],
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
