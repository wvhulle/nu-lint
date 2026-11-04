use std::ops::ControlFlow;

use nu_lint_analysis::{
    LintLevel,
    ast::{
        call::CallExt,
        string::{StringFormat, quote_nu_string},
    },
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};
use nu_protocol::{
    Span,
    ast::{Argument, Call, Expr, Expression},
};

const NOTE: &str = "'echo' returns its arguments unchanged. Use the value directly, or 'print' to \
                    write to the terminal.";

enum EchoedValue {
    BareWord(Span),
    Value(Span),
    Several(Span),
}

pub struct FixData {
    echo_span: Span,
    echoed_value: EchoedValue,
}

fn echoed_value(call: &Call, context: &LintContext) -> Option<EchoedValue> {
    let positional_arguments = call
        .arguments
        .iter()
        .map(|call_argument| match call_argument {
            Argument::Positional(echo_value) | Argument::Unknown(echo_value) => Some(echo_value),
            Argument::Named(_) | Argument::Spread(_) => None,
        })
        .collect::<Option<Vec<_>>>()?;
    match positional_arguments.as_slice() {
        [] => None,
        [single_argument] => match StringFormat::from_expression(single_argument, context) {
            Some(StringFormat::BareWord(_)) => Some(EchoedValue::BareWord(single_argument.span)),
            _ => Some(EchoedValue::Value(single_argument.span)),
        },
        [first_argument, .., last_argument] => Some(EchoedValue::Several(
            first_argument.span.merge(last_argument.span),
        )),
    }
}

fn detect_echo(expr: &Expression, context: &LintContext) -> Option<(Detection, Option<FixData>)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };
    if call.builtin_name(context) != Some("echo") {
        return None;
    }
    let detection =
        Detection::from_global_span(NOTE, expr.span).with_primary_label("identity 'echo'");
    let fix_data = echoed_value(call, context).map(|echoed_value| FixData {
        echo_span: expr.span,
        echoed_value,
    });
    Some((detection, fix_data))
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
        let mut echo_detections = Vec::new();
        context.traverse_with_parent(|expr, parent_expr| {
            if !parent_expr.is_some_and(|parent| matches!(parent.expr, Expr::AttributeBlock(_))) {
                echo_detections.extend(detect_echo(expr, context));
            }
            ControlFlow::Continue(())
        });
        echo_detections
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let fix_data = fix_data.as_ref()?;
        let replacement_text = match fix_data.echoed_value {
            EchoedValue::BareWord(value_span) => quote_nu_string(context.span_text(value_span)),
            EchoedValue::Value(value_span) => context.span_text(value_span).to_string(),
            EchoedValue::Several(values_span) => format!("[{}]", context.span_text(values_span)),
        };
        Some(Fix {
            explanation: "Use the value directly instead of `echo`".into(),
            replacements: vec![Replacement::new(fix_data.echo_span, replacement_text)],
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
