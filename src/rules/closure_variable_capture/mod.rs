use std::collections::HashSet;

use nu_protocol::{Span, VarId, ast::Expr};

use crate::{
    context::LintContext,
    lint::{RuleViolation, Severity},
    rule::{Rule, RuleCategory},
};

type ClosureInfo = (Span, nu_protocol::BlockId, Vec<VarId>);
type VarUsage = (VarId, Span, String);

/// Extract closure parameters from a block signature
fn extract_closure_params(block: &nu_protocol::ast::Block) -> Vec<VarId> {
    block
        .signature
        .required_positional
        .iter()
        .filter_map(|p| p.var_id)
        .chain(
            block
                .signature
                .optional_positional
                .iter()
                .filter_map(|p| p.var_id),
        )
        .chain(
            block
                .signature
                .rest_positional
                .as_ref()
                .and_then(|p| p.var_id),
        )
        .collect()
}

/// Find all closures in the AST and collect their metadata
fn find_closures(context: &LintContext) -> Vec<ClosureInfo> {
    use nu_protocol::ast::Traverse;

    let mut closure_info = Vec::new();

    context.ast.flat_map(
        context.working_set,
        &|expr| {
            (|| {
                let Expr::Closure(block_id) = &expr.expr else {
                    return None;
                };

                let block = context.working_set.get_block(*block_id);
                let closure_params = extract_closure_params(block);
                Some((expr.span, *block_id, closure_params))
            })()
            .into_iter()
            .collect()
        },
        &mut closure_info,
    );

    closure_info
}

/// Extract variable declaration from let/mut call
fn extract_var_decl_from_call(
    call: &nu_protocol::ast::Call,
    context: &LintContext,
) -> Option<VarId> {
    let decl_name = context.working_set.get_decl(call.decl_id).name();
    matches!(decl_name, "let" | "mut").then(|| {
        let var_arg = call.arguments.first()?;

        let (nu_protocol::ast::Argument::Positional(var_expr)
        | nu_protocol::ast::Argument::Unknown(var_expr)) = var_arg
        else {
            return None;
        };

        match &var_expr.expr {
            Expr::VarDecl(var_id) => Some(*var_id),
            _ => None,
        }
    })?
}

/// Collect all variables declared locally within a block
fn collect_local_variables(
    block: &nu_protocol::ast::Block,
    context: &LintContext,
) -> HashSet<VarId> {
    use nu_protocol::ast::Traverse;

    let mut closure_local_vars_vec = Vec::new();

    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            element.expr.flat_map(
                context.working_set,
                &|inner_expr| {
                    extract_call_var_decl(inner_expr, context)
                        .into_iter()
                        .collect()
                },
                &mut closure_local_vars_vec,
            );
        }
    }

    closure_local_vars_vec.into_iter().collect()
}

fn extract_call_var_decl(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<nu_protocol::VarId> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    extract_var_decl_from_call(call, context)
}

fn extract_var_usage(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<VarUsage> {
    let Expr::Var(var_id) = &expr.expr else {
        return None;
    };

    let var_name = &context.source[expr.span.start..expr.span.end];
    Some((*var_id, expr.span, var_name.to_string()))
}

/// Find all variable references in a block
fn collect_variable_usages(
    block: &nu_protocol::ast::Block,
    context: &LintContext,
) -> Vec<VarUsage> {
    use nu_protocol::ast::Traverse;

    let mut used_vars = Vec::new();

    for pipeline in &block.pipelines {
        for element in &pipeline.elements {
            element.expr.flat_map(
                context.working_set,
                &|inner_expr| extract_var_usage(inner_expr, context).into_iter().collect(),
                &mut used_vars,
            );
        }
    }

    used_vars
}

/// Check if a variable is a built-in Nushell variable
fn is_builtin_variable(var_name: &str) -> bool {
    matches!(var_name, "$in" | "$it" | "$env" | "$nu")
        || var_name.starts_with("$env.")
        || var_name.starts_with("$nu.")
}

/// Create a violation for a captured variable
fn create_capture_violation(var_name: &str, use_span: Span) -> RuleViolation {
    RuleViolation::new_dynamic(
        "closure_variable_capture",
        format!("Variable '{var_name}' captured in closure may be stale or out of scope"),
        use_span,
    )
    .with_suggestion_dynamic(format!(
        "Consider using '$in' parameter or passing '{var_name}' as closure parameter"
    ))
}

/// Check if a variable should be flagged for capture
fn should_flag_captured_variable(
    var_id: VarId,
    var_name: &str,
    closure_params: &HashSet<VarId>,
    local_vars: &HashSet<VarId>,
) -> bool {
    !is_builtin_variable(var_name)
        && !closure_params.contains(&var_id)
        && !local_vars.contains(&var_id)
}

/// Analyze a single closure for variable capture violations
fn analyze_closure(
    block_id: nu_protocol::BlockId,
    closure_params: Vec<VarId>,
    context: &LintContext,
) -> Vec<RuleViolation> {
    let block = context.working_set.get_block(block_id);
    let closure_param_set: HashSet<VarId> = closure_params.into_iter().collect();
    let closure_local_vars = collect_local_variables(block, context);
    let used_vars = collect_variable_usages(block, context);

    used_vars
        .into_iter()
        .filter(|(var_id, _, var_name)| {
            should_flag_captured_variable(
                *var_id,
                var_name,
                &closure_param_set,
                &closure_local_vars,
            )
        })
        .map(|(_, use_span, var_name)| create_capture_violation(&var_name, use_span))
        .collect()
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let closures = find_closures(context);

    closures
        .into_iter()
        .flat_map(|(_closure_span, block_id, closure_params)| {
            analyze_closure(block_id, closure_params, context)
        })
        .collect()
}

pub fn rule() -> Rule {
    Rule::new(
        "closure_variable_capture",
        RuleCategory::Idioms,
        Severity::Warning,
        "Detect variable captures in closures; prefer using $in or explicit closure parameters \
         for clarity and to avoid potential scope issues",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
