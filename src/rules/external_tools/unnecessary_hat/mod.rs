use nu_protocol::ast::{Expr, ExternalArgument};

use crate::{
    LintLevel, Violation,
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement},
};

fn build_fix(
    cmd: &str,
    args: &[ExternalArgument],
    span: nu_protocol::Span,
    ctx: &LintContext,
) -> Fix {
    let args_text: String = args
        .iter()
        .map(|arg| match arg {
            ExternalArgument::Regular(e) | ExternalArgument::Spread(e) => ctx.get_span_text(e.span),
        })
        .collect::<Vec<_>>()
        .join(" ");

    let replacement = if args_text.is_empty() {
        cmd.to_string()
    } else {
        format!("{cmd} {args_text}")
    };

    Fix::with_explanation(
        "Remove unnecessary '^' prefix",
        vec![Replacement::new(span, replacement)],
    )
}

fn has_builtin(name: &str, ctx: &LintContext) -> bool {
    ctx.engine_state.find_decl(name.as_bytes(), &[]).is_some()
}

fn check(ctx: &LintContext) -> Vec<Violation> {
    ctx.collect_rule_violations(|expr, ctx| {
        let Expr::ExternalCall(head, args) = &expr.expr else {
            return vec![];
        };

        let has_hat_prefix = expr.span.start + 1 == head.span.start;
        if !has_hat_prefix {
            return vec![];
        }

        let cmd = ctx.get_span_text(head.span);
        if has_builtin(cmd, ctx) {
            return vec![];
        }

        let hat_span = nu_protocol::Span::new(expr.span.start, expr.span.start + 1);

        vec![
            Violation::new(
                format!("Unnecessary '^' prefix on external command '{cmd}'"),
                hat_span,
            )
            .with_primary_label("redundant prefix")
            .with_extra_label("has no built-in equivalent", head.span)
            .with_help(format!(
                "The '^' prefix is only needed when a built-in command with the same name exists. \
                 '{cmd}' has no built-in equivalent, so the prefix is redundant."
            ))
            .with_fix(build_fix(cmd, args, expr.span, ctx)),
        ]
    })
}

pub const fn rule() -> Rule {
    Rule::new(
        "unnecessary_hat",
        "Detect unnecessary '^' prefix on external commands",
        check,
        LintLevel::Warning,
    )
    .with_doc_url("https://www.nushell.sh/book/external_commands.html")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
