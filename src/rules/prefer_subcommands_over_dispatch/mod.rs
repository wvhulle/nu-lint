use nu_protocol::ast::{Call, Expr, MatchPattern, Pattern};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::Rule,
    violation::Violation,
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

fn extract_branch_names(call: &Call) -> impl Iterator<Item = String> + '_ {
    call.get_positional_arg(1)
        .and_then(|arg| match &arg.expr {
            Expr::MatchBlock(patterns) => Some(patterns),
            _ => None,
        })
        .into_iter()
        .flatten()
        .filter_map(|(pattern, _)| match &pattern.pattern {
            Pattern::Expression(expr) => match &expr.expr {
                Expr::String(s) => Some(s.clone()),
                _ => None,
            },
            _ => None,
        })
}

fn build_violation(param_name: &str, match_call: &Call) -> Violation {
    let subcommand_examples = extract_branch_names(match_call)
        .take(3)
        .map(|name| format!("def \"main {name}\" [] {{ ... }}"))
        .collect::<Vec<_>>()
        .join("\n");

    Violation::new(
        format!(
            "Main function uses match dispatch on '{param_name}' - use native subcommands instead"
        ),
        match_call.span(),
    )
    .with_primary_label("match-based dispatch")
    .with_help(format!(
        "Replace match-based dispatch with native subcommands for automatic help, tab completion, \
         and cleaner code:\n\n{subcommand_examples}\n\nRun 'script.nu --help' to see all \
         subcommands automatically."
    ))
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_rule_violations(|expr, ctx| {
        let Expr::Call(def_call) = &expr.expr else {
            return vec![];
        };

        let is_def = matches!(def_call.get_call_name(ctx).as_str(), "def" | "export def");
        let is_main = def_call
            .extract_function_definition(ctx)
            .is_some_and(|(_, name)| name == "main");

        if !is_def || !is_main {
            return vec![];
        }

        let Some((block_id, _)) = def_call.extract_function_definition(ctx) else {
            return vec![];
        };

        extract_optional_string_param(def_call, ctx)
            .and_then(|param| find_match_dispatch(block_id, &param, ctx).map(|call| (param, call)))
            .map(|(param, call)| vec![build_violation(&param, call)])
            .unwrap_or_default()
    })
}

pub const fn rule() -> Rule {
    Rule::new(
        "prefer_subcommands_over_dispatch",
        "Use native 'def \"main subcommand\"' instead of match-based command dispatch in main",
        check,
        LintLevel::Warning,
    )
    .with_doc_url("https://www.nushell.sh/book/modules/creating_subcommands.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
