use std::cell::Cell;

use nu_protocol::{
    Span,
    ast::{Argument, Expr, Traverse},
};

use crate::{
    Fix, LintLevel, Replacement,
    ast::call::CallExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

const LOG_LEVELS: &[&str] = &["debug", "info", "warning", "error", "critical"];

struct FixData {
    name: String,
    removal_span: Span,
}

fn has_stdlib_log_import(context: &LintContext) -> bool {
    let found = Cell::new(false);
    context.detect(|expr, ctx| {
        let Expr::Call(call) = &expr.expr else {
            return vec![];
        };

        if !call.is_call_to_command("use", ctx) {
            return vec![];
        }

        for arg in &call.arguments {
            if let Argument::Positional(e) = arg {
                let text = ctx.expr_text(e);
                if text == "std/log" || text == "std log" || text.starts_with("std/log ") {
                    found.set(true);
                }
            }
        }
        vec![]
    });
    found.get()
}

fn is_custom_log_command(name: &str) -> bool {
    if name == "log" {
        return true;
    }

    for level in LOG_LEVELS {
        if name == format!("log {level}") || name == format!("log-{level}") {
            return true;
        }
    }

    false
}

struct CustomLogCommand;

impl DetectFix for CustomLogCommand {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "custom_log_command"
    }

    fn short_description(&self) -> &'static str {
        "Custom log command shadows stdlib. Use `use std/log` instead"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/standard_library.html#logging")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        if has_stdlib_log_import(context) {
            return vec![];
        }

        let mut results = Vec::new();

        context.ast.flat_map(
            context.working_set,
            &|expr| {
                let Expr::Call(call) = &expr.expr else {
                    return vec![];
                };

                let Some(def) = call.custom_command_def(context) else {
                    return vec![];
                };

                if !is_custom_log_command(&def.name) {
                    return vec![];
                }

                let message = format!(
                    "Custom '{}' command shadows stdlib. Use `use std/log` instead",
                    def.name
                );

                let detection = Detection::from_global_span(message, def.name_span)
                    .with_primary_label("custom log command");

                let fix_data = FixData {
                    name: def.name,
                    removal_span: context.expand_span_to_statement(expr.span),
                };

                vec![(detection, fix_data)]
            },
            &mut results,
        );

        results
    }

    fn fix(&self, _context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        Some({
            let explanation = format!(
                "Remove custom '{}' function (add `use std/log` manually)",
                fix_data.name
            );
            let replacements = vec![Replacement::new(fix_data.removal_span, String::new())];
            Fix {
                explanation: explanation.into(),
                replacements,
            }
        })
    }
}

pub static RULE: &dyn Rule = &CustomLogCommand;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
