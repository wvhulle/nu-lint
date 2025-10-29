use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

fn create_violation(param_name: &str) -> RuleViolation {
    RuleViolation::new_dynamic(
        "missing_type_annotation",
        format!("Parameter '{param_name}' is missing type annotation"),
        nu_protocol::Span::unknown(),
    )
    .with_suggestion_static("Add type annotation like 'param: string' or 'param: int'")
}

fn check_signature(sig: &nu_protocol::Signature) -> Vec<RuleViolation> {
    sig.required_positional
        .iter()
        .chain(&sig.optional_positional)
        .chain(sig.rest_positional.iter())
        .filter(|param| param.shape == nu_protocol::SyntaxShape::Any)
        .map(|param| create_violation(&param.name))
        .collect()
}

fn check_def_call(call: &nu_protocol::ast::Call, ctx: &LintContext) -> Vec<RuleViolation> {
    let decl = ctx.working_set.get_decl(call.decl_id);
    if decl.name() != "def" {
        return vec![];
    }

    call.arguments
        .iter()
        .filter_map(|arg| {
            if let nu_protocol::ast::Argument::Positional(arg_expr) = arg
                && let Expr::Signature(sig) = &arg_expr.expr
            {
                return Some(check_signature(sig));
            }
            None
        })
        .flatten()
        .collect()
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    context.collect_rule_violations(|expr, ctx| match &expr.expr {
        Expr::Call(call) => check_def_call(call, ctx),
        _ => vec![],
    })
}

pub fn rule() -> Rule {
    Rule::new(
        "missing_type_annotation",
        RuleCategory::TypeSafety,
        Severity::Info,
        "Parameters should have type annotations",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
