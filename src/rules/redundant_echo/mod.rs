use nu_protocol::ast::{Argument, Expr, Expression};

use crate::{
    LintLevel,
    ast::{
        call::CallExt,
        string::{StringFormat, quote_nu_string},
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

const NOTE: &str = "'echo' returns its arguments unchanged. Use the value directly, or 'print' to \
                    write to the terminal.";

fn standalone_value(argument: &Expression, context: &LintContext) -> String {
    match StringFormat::from_expression(argument, context) {
        Some(StringFormat::BareWord(word)) => quote_nu_string(&word),
        _ => context.expr_text(argument).to_string(),
    }
}

fn echoed_value(arguments: &[&Expression], context: &LintContext) -> Option<String> {
    match arguments {
        [] => None,
        [single] => Some(standalone_value(single, context)),
        several => Some(format!(
            "[{}]",
            several
                .iter()
                .map(|argument| context.expr_text(argument))
                .collect::<Vec<_>>()
                .join(" ")
        )),
    }
}

fn detect_echo(expr: &Expression, context: &LintContext) -> Vec<(Detection, Option<Replacement>)> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };
    if call.get_call_name(context) != "echo" {
        return vec![];
    }
    let positional: Option<Vec<&Expression>> = call
        .arguments
        .iter()
        .map(|argument| match argument {
            Argument::Positional(value) | Argument::Unknown(value) => Some(value),
            Argument::Named(_) | Argument::Spread(_) => None,
        })
        .collect();
    let replacement = positional
        .and_then(|arguments| echoed_value(&arguments, context))
        .map(|value| Replacement::new(expr.span, value));
    let detection =
        Detection::from_global_span(NOTE, expr.span).with_primary_label("identity 'echo'");
    vec![(detection, replacement)]
}

struct RedundantEcho;

impl DetectFix for RedundantEcho {
    type FixInput<'a> = Option<Replacement>;

    fn id(&self) -> &'static str {
        "redundant_echo"
    }

    fn short_description(&self) -> &'static str {
        "Redundant `echo` (identity function)"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/thinking_in_nu.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(detect_echo)
    }

    fn fix(&self, _context: &LintContext, replacement: &Self::FixInput<'_>) -> Option<Fix> {
        let replacement = replacement.as_ref()?;
        Some(Fix {
            explanation: format!("Use '{}' directly", replacement.replacement_text).into(),
            replacements: vec![replacement.clone()],
        })
    }
}

pub static RULE: &dyn Rule = &RedundantEcho;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
