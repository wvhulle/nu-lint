use std::collections::{HashMap, HashSet};

use nu_protocol::{Span, VarId, ast::Expr};

use crate::{
    context::LintContext,
    lint::{Fix, Replacement, RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

/// Find the span of 'mut ' keyword before the variable name
fn find_mut_keyword_span(source: &str, var_span: Span) -> Span {
    let start = var_span.start.min(source.len());
    let search_start = start.saturating_sub(20);
    let text_before = &source[search_start..start];

    if let Some(mut_pos) = text_before.rfind("mut ") {
        let abs_mut_start = search_start + mut_pos;
        let abs_mut_end = abs_mut_start + 4;
        return Span::new(abs_mut_start, abs_mut_end);
    }

    var_span
}

fn extract_mut_declaration(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<(VarId, String, Span, Span)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    let decl_name = context.working_set.get_decl(call.decl_id).name();
    if decl_name != "mut" {
        return None;
    }

    let var_arg = call.arguments.first()?;

    let (nu_protocol::ast::Argument::Positional(var_expr)
    | nu_protocol::ast::Argument::Unknown(var_expr)) = var_arg
    else {
        return None;
    };

    let Expr::VarDecl(var_id) = &var_expr.expr else {
        return None;
    };

    let var_name = &context.source[var_expr.span.start..var_expr.span.end];

    if var_name.starts_with('_') {
        return None;
    }

    let mut_span = find_mut_keyword_span(context.source, var_expr.span);
    Some((*var_id, var_name.to_string(), var_expr.span, mut_span))
}

fn extract_reassigned_var(expr: &nu_protocol::ast::Expression) -> Option<VarId> {
    let Expr::BinaryOp(lhs, op, _rhs) = &expr.expr else {
        return None;
    };

    let Expr::Operator(nu_protocol::ast::Operator::Assignment(_)) = &op.expr else {
        return None;
    };

    match &lhs.expr {
        Expr::Var(var_id) => Some(*var_id),
        Expr::FullCellPath(cell_path) => {
            if let Expr::Var(var_id) = &cell_path.head.expr {
                Some(*var_id)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    use nu_protocol::ast::Traverse;

    let mut mut_declarations: Vec<(VarId, String, Span, Span)> = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| extract_mut_declaration(expr, context).into_iter().collect(),
        &mut mut_declarations,
    );

    let mut_variables: HashMap<VarId, (String, Span, Span)> = mut_declarations
        .into_iter()
        .map(|(id, name, decl_span, mut_span)| (id, (name, decl_span, mut_span)))
        .collect();

    let mut reassigned: Vec<VarId> = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| extract_reassigned_var(expr).into_iter().collect(),
        &mut reassigned,
    );

    let reassigned_vars: HashSet<VarId> = reassigned.into_iter().collect();

    // Generate violations for mut variables that were never reassigned
    let mut violations = Vec::new();
    for (var_id, (var_name, decl_span, mut_span)) in mut_variables {
        if !reassigned_vars.contains(&var_id) {
            let fix = Fix::new_dynamic(
                format!("Remove 'mut' keyword from variable '{var_name}'"),
                vec![Replacement::new_static(mut_span, "")],
            );

            violations.push(
                RuleViolation::new_dynamic(
                    "unnecessary_mut",
                    format!("Variable '{var_name}' is declared as 'mut' but never reassigned"),
                    decl_span,
                )
                .with_suggestion_dynamic(format!("Remove 'mut' keyword:\nlet {var_name} = ..."))
                .with_fix(fix),
            );
        }
    }

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "unnecessary_mut",
        RuleCategory::CodeQuality,
        Severity::Info,
        "Variables should only be marked 'mut' when they are actually reassigned",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
