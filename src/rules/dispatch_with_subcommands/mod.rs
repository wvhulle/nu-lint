use nu_protocol::ast::{Call, Expr, MatchPattern, Pattern};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn extract_optional_string_param(call: &Call, context: &LintContext) -> Option<String> {
    let sig_text = call.get_positional_arg(1)?.span_text(context);

    (sig_text.contains("?: string") || sig_text.contains(": string = "))
        .then(|| {
            sig_text
                .split(&['[', ']', ':', '?'][..])
                .find(|s| !s.is_empty() && !s.starts_with(' '))
                .map(|s| s.trim().to_string())
        })
        .flatten()
}

fn is_string_literal_pattern(pattern: &MatchPattern) -> bool {
    matches!(
        &pattern.pattern,
        Pattern::Expression(expr) if matches!(&expr.expr, Expr::String(_))
    )
}

fn matches_dispatch_variable(call: &Call, var_name: &str, context: &LintContext) -> bool {
    call.get_first_positional_arg()
        .and_then(|cond| cond.extract_variable_name(context))
        .is_some_and(|v| v.trim_start_matches('$').trim_end_matches('?') == var_name)
}

fn has_string_patterns(call: &Call) -> bool {
    call.get_positional_arg(1)
        .and_then(|arg| match &arg.expr {
            Expr::MatchBlock(patterns) => Some(patterns),
            _ => None,
        })
        .is_some_and(|patterns| patterns.iter().any(|(p, _)| is_string_literal_pattern(p)))
}

fn find_match_dispatch<'a>(
    block_id: nu_protocol::BlockId,
    param_name: &str,
    context: &'a LintContext<'a>,
) -> Option<&'a Call> {
    context
        .working_set
        .get_block(block_id)
        .pipelines
        .iter()
        .flat_map(|p| &p.elements)
        .filter_map(|e| match &e.expr.expr {
            Expr::Call(call) => Some(call.as_ref()),
            _ => None,
        })
        .find(|call| {
            call.get_call_name(context) == "match"
                && matches_dispatch_variable(call, param_name, context)
                && has_string_patterns(call)
        })
}

fn build_violation(param_name: &str, match_call: &Call) -> Detection {
    Detection::from_global_span(
        format!(
            "Main function uses match dispatch on '{param_name}' - use native subcommands instead"
        ),
        match_call.span(),
    )
    .with_primary_label("match-based dispatch")
}

struct DispatchWithSubcommands;

impl DetectFix for DispatchWithSubcommands {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "dispatch_with_subcommands"
    }

    fn short_description(&self) -> &'static str {
        "Match dispatch replaceable with subcommands"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/custom_commands.html#subcommands")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(context.detect(|expr, ctx| {
            let Expr::Call(def_call) = &expr.expr else {
                return vec![];
            };

            let Some(def) = def_call.custom_command_def(ctx) else {
                return vec![];
            };

            if !def.is_main() {
                return vec![];
            }

            extract_optional_string_param(def_call, ctx)
                .and_then(|param| {
                    find_match_dispatch(def.body, &param, ctx).map(|call| (param, call))
                })
                .map(|(param, call)| vec![build_violation(&param, call)])
                .unwrap_or_default()
        }))
    }
}

pub static RULE: &dyn Rule = &DispatchWithSubcommands;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
