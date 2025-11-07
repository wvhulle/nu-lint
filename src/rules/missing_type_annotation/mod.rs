use nu_protocol::ast::{Argument, Call, Expr, Expression, Operator, Pipeline};

use crate::{
    ast::{call::CallExt, expression::ExpressionExt, syntax_shape::SyntaxShapeExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{Fix, Replacement, RuleViolation, Severity},
};

fn refers_to_param(expr: &Expression, param_name: &str, ctx: &LintContext) -> bool {
    match &expr.expr {
        Expr::Var(var_id) => {
            let var = ctx.working_set.get_variable(*var_id);
            let var_span_text = &ctx.source[var.declaration_span.start..var.declaration_span.end];
            let var_name_normalized = var_span_text.trim_end_matches('?').trim_start_matches("...");
            let matches = var_name_normalized == param_name;
            log::debug!("    refers_to_param: comparing '{var_name_normalized}' (from '{var_span_text}') == '{param_name}' -> {matches}");
            matches
        }
        Expr::FullCellPath(cell_path) => refers_to_param(&cell_path.head, param_name, ctx),
        _ => false,
    }
}

fn infer_param_type_from_usage(
    param_name: &str,
    body_block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> &'static str {
    let block = ctx.working_set.get_block(body_block_id);
    log::debug!("Inferring type for parameter '{param_name}' in block");

    for pipeline in &block.pipelines {
        if let Some(value) = infer_from_pipe_into(param_name, ctx, pipeline) {
            log::debug!("Found type from pipe: {value}");
            return value;
        }

        for element in &pipeline.elements {
            if let Some(inferred) = check_expr_for_type_hint(&element.expr, param_name, ctx) {
                log::debug!("Found type hint: {inferred}");
                return inferred;
            }
        }
    }

    log::debug!("No type hint found, defaulting to 'any'");
    "any"
}

fn infer_from_pipe_into(
    param_name: &str,
    ctx: &LintContext<'_>,
    pipeline: &Pipeline,
) -> Option<&'static str> {
    for i in 0..pipeline.elements.len() {
        if i > 0 && refers_to_param(&pipeline.elements[i - 1].expr, param_name, ctx) {
            // Parameter is input to this command
            if let Expr::Call(call) = &pipeline.elements[i].expr.expr
                && let Some(inferred) = infer_type_from_command(&call.get_call_name(ctx))
            {
                return Some(inferred);
            }
        }
    }
    None
}

fn check_expr_for_type_hint(
    expr: &Expression,
    param_name: &str,
    ctx: &LintContext,
) -> Option<&'static str> {
    log::debug!(
        "Checking expression for type hint, param: {param_name}, expr: {:?}",
        &expr.expr
    );

    let result = match &expr.expr {
        Expr::Call(call) => {
            let cmd_name = call.get_call_name(ctx);
            log::debug!("  Found Call to '{cmd_name}'");

            if refers_to_param(expr, param_name, ctx) {
                log::debug!("  Expression refers to param, inferring from command");
                infer_type_from_command(&cmd_name)
            } else {
                log::debug!("  Checking call arguments");
                call.arguments.iter().find_map(|arg| match arg {
                    Argument::Positional(arg_expr) | Argument::Unknown(arg_expr) => {
                        check_expr_for_type_hint(arg_expr, param_name, ctx)
                    }
                    _ => None,
                })
            }
        }
        Expr::FullCellPath(cell_path) => {
            let refers = refers_to_param(&cell_path.head, param_name, ctx);
            let has_tail = !cell_path.tail.is_empty();
            log::debug!("  FullCellPath: refers={refers}, has_tail={has_tail}");

            if refers && has_tail {
                Some("record")
            } else {
                check_expr_for_type_hint(&cell_path.head, param_name, ctx)
            }
        }
        Expr::BinaryOp(left, op_expr, right) if matches!(&op_expr.expr, Expr::Operator(op) if matches!(op, Operator::Math(_) | Operator::Comparison(_))) =>
        {
            let left_refers = refers_to_param(left, param_name, ctx);
            let right_refers = refers_to_param(right, param_name, ctx);
            log::debug!(
                "  BinaryOp (math/comparison): left_refers={left_refers}, right_refers={right_refers}"
            );

            if left_refers || right_refers {
                Some("int")
            } else {
                check_expr_for_type_hint(left, param_name, ctx)
                    .or_else(|| check_expr_for_type_hint(right, param_name, ctx))
            }
        }
        Expr::BinaryOp(left, _, right) => {
            log::debug!("  BinaryOp (other)");
            check_expr_for_type_hint(left, param_name, ctx)
                .or_else(|| check_expr_for_type_hint(right, param_name, ctx))
        }
        Expr::Subexpression(block_id) | Expr::Block(block_id) | Expr::Closure(block_id) => {
            log::debug!("  Recursing into block");
            let block = ctx.working_set.get_block(*block_id);

            block
                .pipelines
                .iter()
                .find_map(|pipeline| {
                    infer_from_pipe_into(param_name, ctx, pipeline).or_else(|| {
                        pipeline
                            .elements
                            .iter()
                            .find_map(|element| check_expr_for_type_hint(&element.expr, param_name, ctx))
                    })
                })
        }
        Expr::MatchBlock(match_patterns) => {
            log::debug!("  MatchBlock with {} patterns", match_patterns.len());
            match_patterns
                .iter()
                .find_map(|(_, block_expr)| check_expr_for_type_hint(block_expr, param_name, ctx))
        }
        Expr::Collect(_, inner) | Expr::UnaryNot(inner) => {
            log::debug!("  Collect/UnaryNot, checking inner");
            check_expr_for_type_hint(inner, param_name, ctx)
        }
        other => {
            log::debug!(
                "  Unhandled expression type: {:?}",
                std::mem::discriminant(other)
            );
            None
        }
    };

    if let Some(inferred) = result {
        log::debug!("  -> Inferred type: {inferred}");
    }

    result
}

fn infer_type_from_command(cmd_name: &str) -> Option<&'static str> {
    match cmd_name {
        "str trim" | "str replace" | "str upcase" | "str downcase" | "str contains" => {
            Some("string")
        }
        "each" | "where" | "filter" | "reduce" | "append" | "prepend" => Some("list"),
        _ => None,
    }
}

fn generate_fix(
    signature: &nu_protocol::Signature,
    body_block_id: nu_protocol::BlockId,
    signature_span: nu_protocol::Span,
    ctx: &LintContext,
) -> Fix {
    let new_sig = generate_typed_signature(signature, body_block_id, ctx);

    Fix::new_static(
        "Add type annotations to parameters",
        vec![Replacement::new_dynamic(signature_span, new_sig)],
    )
}

fn generate_typed_signature(
    signature: &nu_protocol::Signature,
    body_block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> String {
    let params: Vec<String> = signature
        .required_positional
        .iter()
        .map(|param| {
            let type_str = if param.shape == nu_protocol::SyntaxShape::Any {
                infer_param_type_from_usage(&param.name, body_block_id, ctx).to_string()
            } else {
                param.shape.to_type_string()
            };
            format!("{}: {}", param.name, type_str)
        })
        .chain(signature.optional_positional.iter().map(|param| {
            let type_str = if param.shape == nu_protocol::SyntaxShape::Any {
                infer_param_type_from_usage(&param.name, body_block_id, ctx).to_string()
            } else {
                param.shape.to_type_string()
            };
            format!("{}?: {}", param.name, type_str)
        }))
        .chain(signature.rest_positional.iter().map(|param| {
            let type_str = if param.shape == nu_protocol::SyntaxShape::Any {
                infer_param_type_from_usage(&param.name, body_block_id, ctx).to_string()
            } else {
                param.shape.to_type_string()
            };
            format!("...{}: {}", param.name, type_str)
        }))
        .collect();

    format!("[{}]", params.join(", "))
}

fn create_violation(param_name: &str, param_span: nu_protocol::Span, fix: Fix) -> RuleViolation {
    RuleViolation::new_dynamic(
        "missing_type_annotation",
        format!("Parameter '{param_name}' is missing type annotation"),
        param_span,
    )
    .with_suggestion_static("Add type annotation like 'param: string' or 'param: int'")
    .with_fix(fix)
}

fn check_signature(
    sig: &nu_protocol::Signature,
    _call: &Call,
    signature_span: nu_protocol::Span,
    body_block_id: nu_protocol::BlockId,
    ctx: &LintContext,
) -> Vec<RuleViolation> {
    let params_needing_types: Vec<_> = sig
        .required_positional
        .iter()
        .chain(&sig.optional_positional)
        .chain(sig.rest_positional.iter())
        .filter(|param| param.shape == nu_protocol::SyntaxShape::Any)
        .collect();

    if params_needing_types.is_empty() {
        return vec![];
    }

    let fix = generate_fix(sig, body_block_id, signature_span, ctx);

    params_needing_types
        .into_iter()
        .map(|param| {
            let param_span = ctx
                .working_set
                .get_span_contents(signature_span)
                .windows(param.name.len())
                .position(|window| window == param.name.as_bytes())
                .map_or(signature_span, |offset| {
                    nu_protocol::Span::new(
                        signature_span.start + offset,
                        signature_span.start + offset + param.name.len(),
                    )
                });

            create_violation(&param.name, param_span, fix.clone())
        })
        .collect()
}

fn check_def_call(call: &Call, ctx: &LintContext) -> Vec<RuleViolation> {
    let decl = ctx.working_set.get_decl(call.decl_id);
    if decl.name() != "def" && decl.name() != "export def" {
        return vec![];
    }

    let Some(sig_arg) = call.get_positional_arg(1) else {
        return vec![];
    };

    let Some(body_arg) = call.get_positional_arg(2) else {
        return vec![];
    };

    let Some(body_block_id) = body_arg.extract_block_id() else {
        return vec![];
    };

    if let Expr::Signature(sig) = &sig_arg.expr {
        return check_signature(sig, call, sig_arg.span, body_block_id, ctx);
    }

    vec![]
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
        Severity::Warning,
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
