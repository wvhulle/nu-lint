use std::{ffi::OsStr, path::Path};

use nu_protocol::ast::{Expression, ExternalArgument};

use crate::{
    LintLevel,
    ast::external::{ExternalInvocation, literal_content},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

const STDIN_FLAG: &str = "--stdin";

fn is_stdin_flag(argument: &ExternalArgument) -> bool {
    matches!(argument, ExternalArgument::Regular(expr) if literal_content(expr) == Some(STDIN_FLAG))
}

fn constant_script_path<'a>(invocation: &ExternalInvocation<'a>) -> Option<&'a Expression> {
    let first_operand = invocation
        .args
        .iter()
        .find(|argument| !is_stdin_flag(argument))?;
    let ExternalArgument::Regular(script) = first_operand else {
        return None;
    };
    let path = literal_content(script)?;
    let is_nu_script = Path::new(path).extension() == Some(OsStr::new("nu"));
    is_nu_script.then_some(script)
}

struct NuScriptToRun;

impl DetectFix for NuScriptToRun {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "nu_script_to_run"
    }

    fn short_description(&self) -> &'static str {
        "Run Nu scripts with the `run` keyword instead of a `nu` subprocess"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Since Nushell 0.114.0, `run script.nu` executes a script in an isolated scope inside \
             the current pipeline. Unlike `^nu script.nu`, it does not start a new process, \
             receives piped input without `--stdin`, and passes structured values in and out \
             instead of text. Because the output becomes structured and a non-final `run` \
             statement is no longer printed, review downstream text processing such as `lines` or \
             `from json` when switching.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/run.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(
            context
                .external_invocations(&["nu"])
                .iter()
                .filter_map(|invocation| {
                    let script = constant_script_path(invocation)?;
                    Some(
                        Detection::from_global_span(
                            format!(
                                "Use `run {}` to run the script inside this Nushell instead of \
                                 starting a new `nu` process",
                                context.span_text(script.span)
                            ),
                            invocation.span,
                        )
                        .with_primary_label("starts a separate nu process")
                        .with_extra_label("script that `run` can execute", script.span),
                    )
                })
                .collect(),
        )
    }
}

pub static RULE: &dyn Rule = &NuScriptToRun;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
