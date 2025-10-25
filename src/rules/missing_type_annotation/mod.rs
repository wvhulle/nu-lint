use nu_protocol::ast::Expr;

use crate::{
    context::LintContext,
    lint::{Severity, Violation},
    rule::{Rule, RuleCategory},
};

fn check_signature(sig: &nu_protocol::Signature) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Check required positional parameters
    for param in &sig.required_positional {
        if param.shape == nu_protocol::SyntaxShape::Any {
            violations.push(Violation {
                rule_id: "missing_type_annotation".into(),
                severity: Severity::Info,
                message: format!("Parameter '{}' is missing type annotation", param.name).into(),
                span: nu_protocol::Span::unknown(), /* Signature doesn't store individual
                                                     * param spans */
                suggestion: Some(
                    "Add type annotation like 'param: string' or 'param: int'"
                        .to_string()
                        .into(),
                ),
                fix: None,
                file: None,
            });
        }
    }

    // Check optional positional parameters
    for param in &sig.optional_positional {
        if param.shape == nu_protocol::SyntaxShape::Any {
            violations.push(Violation {
                rule_id: "missing_type_annotation".into(),
                severity: Severity::Info,
                message: format!("Parameter '{}' is missing type annotation", param.name).into(),
                span: nu_protocol::Span::unknown(),
                suggestion: Some(
                    "Add type annotation like 'param: string' or 'param: int'"
                        .to_string()
                        .into(),
                ),
                fix: None,
                file: None,
            });
        }
    }

    // Check rest positional parameter
    if let Some(param) = &sig.rest_positional
        && param.shape == nu_protocol::SyntaxShape::Any
    {
        violations.push(Violation {
            rule_id: "missing_type_annotation".into(),
            severity: Severity::Info,
            message: format!("Parameter '{}' is missing type annotation", param.name).into(),
            span: nu_protocol::Span::unknown(),
            suggestion: Some(
                "Add type annotation like 'param: string' or 'param: list'"
                    .to_string()
                    .into(),
            ),
            fix: None,
            file: None,
        });
    }

    violations
}

fn check(context: &LintContext) -> Vec<Violation> {
    context.collect_violations(|expr, ctx| {
        match &expr.expr {
            Expr::Call(call) => {
                // Check if this is a def command
                let decl = ctx.working_set.get_decl(call.decl_id);
                if decl.name() == "def" {
                    // The second positional argument of 'def' is a Signature expression
                    // def name [params] { body }
                    for arg in &call.arguments {
                        if let nu_protocol::ast::Argument::Positional(arg_expr) = arg
                            && let Expr::Signature(sig) = &arg_expr.expr
                        {
                            return check_signature(sig);
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
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
