use nu_protocol::{
    Span,
    ast::{Argument, Call, Expr, Expression},
};

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

enum EchoedValue {
    BareWord(Span),
    Value(Span),
    Several(Span),
}

pub struct FixData {
    span: Span,
    echoed: EchoedValue,
}

fn positional_arguments(call: &Call) -> Option<Vec<&Expression>> {
    call.arguments
        .iter()
        .map(|argument| match argument {
            Argument::Positional(value) | Argument::Unknown(value) => Some(value),
            Argument::Named(_) | Argument::Spread(_) => None,
        })
        .collect()
}

fn echoed_value(arguments: &[&Expression], context: &LintContext) -> Option<EchoedValue> {
    match arguments {
        [] => None,
        [single] => match StringFormat::from_expression(single, context) {
            Some(StringFormat::BareWord(_)) => Some(EchoedValue::BareWord(single.span)),
            _ => Some(EchoedValue::Value(single.span)),
        },
        [first, .., last] => Some(EchoedValue::Several(Span::new(
            first.span.start,
            last.span.end,
        ))),
    }
}

fn fix_data(expr: &Expression, call: &Call, context: &LintContext) -> Option<FixData> {
    let arguments = positional_arguments(call)?;
    Some(FixData {
        span: expr.span,
        echoed: echoed_value(&arguments, context)?,
    })
}

fn detect_echo(expr: &Expression, context: &LintContext) -> Vec<(Detection, Option<FixData>)> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };
    if call.get_call_name(context) != "echo" {
        return vec![];
    }
    let detection =
        Detection::from_global_span(NOTE, expr.span).with_primary_label("identity 'echo'");
    vec![(detection, fix_data(expr, call, context))]
}

struct RedundantEcho;

impl DetectFix for RedundantEcho {
    type FixInput<'a> = Option<FixData>;

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

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let fix_data = fix_data.as_ref()?;
        let replacement = match fix_data.echoed {
            EchoedValue::BareWord(span) => quote_nu_string(context.span_text(span)),
            EchoedValue::Value(span) => context.span_text(span).to_string(),
            EchoedValue::Several(span) => format!("[{}]", context.span_text(span)),
        };
        Some(Fix {
            explanation: format!("Use '{replacement}' directly").into(),
            replacements: vec![Replacement::new(fix_data.span, replacement)],
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
