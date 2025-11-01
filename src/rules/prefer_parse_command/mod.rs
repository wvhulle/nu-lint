use nu_protocol::{Span, VarId, ast::Expr};

use crate::{
    ast::{CallExt, SpanExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

fn is_split_row_call(call: &nu_protocol::ast::Call, context: &LintContext) -> bool {
    call.is_call_to_command("split row", context)
}

fn is_indexed_access_call(call: &nu_protocol::ast::Call, context: &LintContext) -> bool {
    let name = call.get_call_name(context);
    matches!(name.as_str(), "get" | "skip")
}

fn has_index_argument(call: &nu_protocol::ast::Call, context: &LintContext) -> bool {
    call.get_first_positional_arg().is_some_and(|arg| {
        let arg_text = arg.span.text(context);
        arg_text.parse::<usize>().is_ok()
    })
}

fn check_pipeline_for_split_get(
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> Option<RuleViolation> {
    if pipeline.elements.len() < 2 {
        return None;
    }

    pipeline.elements.windows(2).find_map(|window| {
        let (current, next) = (&window[0], &window[1]);

        let (Expr::Call(current_call), Expr::Call(next_call)) =
            (&current.expr.expr, &next.expr.expr)
        else {
            return None;
        };

        if is_split_row_call(current_call, context)
            && is_indexed_access_call(next_call, context)
            && has_index_argument(next_call, context)
        {
            let span = Span::new(current.expr.span.start, next.expr.span.end);
            Some(
                RuleViolation::new_static(
                    "prefer_parse_command",
                    "Manual string splitting with indexed access - consider using 'parse'",
                    span,
                )
                .with_suggestion_static(
                    "Use 'parse \"pattern {field1} {field2}\"' for structured text extraction",
                ),
            )
        } else {
            None
        }
    })
}

fn check_expression_for_split_row(expr: &Expr, context: &LintContext) -> bool {
    match expr {
        Expr::Call(call) if is_split_row_call(call, context) => true,
        Expr::FullCellPath(cp) => check_expression_for_split_row(&cp.head.expr, context),
        Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block
                .pipelines
                .iter()
                .any(|p| has_split_row_in_pipeline(p, context))
        }
        _ => false,
    }
}

fn has_split_row_in_pipeline(pipeline: &nu_protocol::ast::Pipeline, context: &LintContext) -> bool {
    pipeline
        .elements
        .iter()
        .any(|element| check_expression_for_split_row(&element.expr.expr, context))
}

fn extract_split_row_assignment(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<(VarId, String, Span)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    if !call.is_call_to_command("let", context) {
        return None;
    }

    let (var_id, var_name, _var_span) = call.extract_variable_declaration(context)?;
    let value_expr = call.get_positional_arg(1)?;

    log::debug!("Checking let statement for variable: {var_name}");

    // Check direct call to split row
    if let Expr::Call(value_call) = &value_expr.expr
        && is_split_row_call(value_call, context)
    {
        log::debug!("Variable {var_name} assigned from split row (direct call)");
        return Some((var_id, var_name, expr.span));
    }

    // Check call in cell path
    if let Expr::FullCellPath(cell_path) = &value_expr.expr {
        if let Expr::Call(head_call) = &cell_path.head.expr
            && is_split_row_call(head_call, context)
        {
            log::debug!("Variable {var_name} assigned from split row (cell path)");
            return Some((var_id, var_name, expr.span));
        }

        // Check subexpression in cell path
        if let Expr::Subexpression(block_id) = &cell_path.head.expr {
            let block = context.working_set.get_block(*block_id);
            if block.pipelines.len() == 1 && has_split_row_in_pipeline(&block.pipelines[0], context)
            {
                log::debug!(
                    "Variable {var_name} assigned from split row (subexpression in cell path)"
                );
                return Some((var_id, var_name, expr.span));
            }
        }
    }

    // Check subexpression directly
    if let Expr::Subexpression(block_id) = &value_expr.expr {
        let block = context.working_set.get_block(*block_id);
        if block.pipelines.len() == 1 && has_split_row_in_pipeline(&block.pipelines[0], context) {
            log::debug!("Variable {var_name} assigned from split row (direct subexpression)");
            return Some((var_id, var_name, expr.span));
        }
    }

    // Check block directly (for closures/blocks in let assignments)
    if let Expr::Block(block_id) = &value_expr.expr {
        let block = context.working_set.get_block(*block_id);
        if block.pipelines.len() == 1 && has_split_row_in_pipeline(&block.pipelines[0], context) {
            log::debug!("Variable {var_name} assigned from split row (direct block)");
            return Some((var_id, var_name, expr.span));
        }
    }

    None
}

fn is_var_used_in_indexed_access(
    var_id: VarId,
    call: &nu_protocol::ast::Call,
    context: &LintContext,
) -> bool {
    if !is_indexed_access_call(call, context) || !has_index_argument(call, context) {
        return false;
    }

    call.arguments.iter().any(|arg| {
        matches!(
            arg,
            nu_protocol::ast::Argument::Positional(arg_expr)
            | nu_protocol::ast::Argument::Unknown(arg_expr)
            if matches!(&arg_expr.expr, Expr::Var(ref_var_id) if *ref_var_id == var_id)
        )
    })
}

fn is_var_piped_to_indexed_access(var_id: VarId, pipeline: &nu_protocol::ast::Pipeline) -> bool {
    if pipeline.elements.is_empty() {
        return false;
    }

    let first = &pipeline.elements[0];
    matches!(&first.expr.expr, Expr::FullCellPath(cell_path) 
        if matches!(&cell_path.head.expr, Expr::Var(ref_var_id) if *ref_var_id == var_id)
        && cell_path.tail.is_empty())
}

fn create_indexed_access_violation(var_name: &str, decl_span: Span) -> RuleViolation {
    RuleViolation::new_dynamic(
        "prefer_parse_command",
        format!(
            "Variable '{var_name}' from split row with indexed access - consider using 'parse'"
        ),
        decl_span,
    )
    .with_suggestion_static("Use 'parse' command to extract named fields instead of indexed access")
}

fn variable_used_in_pipeline(var_id: VarId, pipeline: &nu_protocol::ast::Pipeline) -> bool {
    pipeline.elements.iter().any(|elem| match &elem.expr.expr {
        Expr::Var(v_id) if *v_id == var_id => true,
        Expr::FullCellPath(cp) => matches!(&cp.head.expr, Expr::Var(v_id) if *v_id == var_id),
        _ => false,
    })
}

fn check_pipeline_for_indexed_call(
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> bool {
    pipeline.elements.iter().any(|element| {
        matches!(&element.expr.expr, Expr::Call(call)
            if is_indexed_access_call(call, context) && has_index_argument(call, context))
    })
}

fn check_call_arguments_for_violation(
    call: &nu_protocol::ast::Call,
    var_id: VarId,
    var_name: &str,
    decl_span: Span,
    context: &LintContext,
) -> Option<RuleViolation> {
    call.arguments.iter().find_map(|arg| match arg {
        nu_protocol::ast::Argument::Positional(arg_expr)
        | nu_protocol::ast::Argument::Unknown(arg_expr) => {
            if let Expr::Block(block_id) = &arg_expr.expr {
                let nested_block = context.working_set.get_block(*block_id);
                check_for_indexed_variable_access(
                    var_id,
                    var_name,
                    decl_span,
                    nested_block,
                    context,
                )
            } else {
                None
            }
        }
        _ => None,
    })
}

fn check_element_for_indexed_access(
    element: &nu_protocol::ast::PipelineElement,
    var_id: VarId,
    var_name: &str,
    decl_span: Span,
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> Option<RuleViolation> {
    match &element.expr.expr {
        Expr::FullCellPath(cp) if matches!(&cp.head.expr, Expr::Subexpression(_)) => {
            if let Expr::Subexpression(block_id) = &cp.head.expr {
                let nested_block = context.working_set.get_block(*block_id);
                check_for_indexed_variable_access(
                    var_id,
                    var_name,
                    decl_span,
                    nested_block,
                    context,
                )
            } else {
                None
            }
        }
        Expr::Call(call) => {
            let has_direct_var_usage = is_var_used_in_indexed_access(var_id, call, context)
                || (is_indexed_access_call(call, context)
                    && has_index_argument(call, context)
                    && is_var_piped_to_indexed_access(var_id, pipeline));

            if has_direct_var_usage {
                Some(create_indexed_access_violation(var_name, decl_span))
            } else {
                check_call_arguments_for_violation(call, var_id, var_name, decl_span, context)
            }
        }
        Expr::Block(block_id) => {
            let nested_block = context.working_set.get_block(*block_id);
            check_for_indexed_variable_access(var_id, var_name, decl_span, nested_block, context)
        }
        _ => None,
    }
}

fn check_for_indexed_variable_access(
    var_id: VarId,
    var_name: &str,
    decl_span: Span,
    block: &nu_protocol::ast::Block,
    context: &LintContext,
) -> Option<RuleViolation> {
    log::debug!("Checking for indexed access of variable: {var_name}");

    block.pipelines.iter().find_map(|pipeline| {
        // If variable is used in this pipeline and there's an indexed access call,
        // report violation
        if variable_used_in_pipeline(var_id, pipeline)
            && check_pipeline_for_indexed_call(pipeline, context)
        {
            log::debug!("Found indexed access for variable {var_name} in pipeline");
            return Some(create_indexed_access_violation(var_name, decl_span));
        }

        // Recursively check nested expressions
        pipeline.elements.iter().find_map(|element| {
            check_element_for_indexed_access(
                element, var_id, var_name, decl_span, pipeline, context,
            )
        })
    })
}

fn check_block(
    block: &nu_protocol::ast::Block,
    context: &LintContext,
    violations: &mut Vec<RuleViolation>,
) {
    // Check for inline split row | get/skip patterns
    for pipeline in &block.pipelines {
        if let Some(violation) = check_pipeline_for_split_get(pipeline, context) {
            violations.push(violation);
        }
    }

    // Check for split row assignment followed by indexed access
    let split_row_violations = block
        .pipelines
        .iter()
        .enumerate()
        .filter(|(_, pipeline)| pipeline.elements.len() == 1)
        .filter_map(|(i, pipeline)| {
            let element = &pipeline.elements[0];
            extract_split_row_assignment(&element.expr, context)
                .map(|(var_id, var_name, decl_span)| (i, var_id, var_name, decl_span))
        })
        .find_map(|(i, var_id, var_name, decl_span)| {
            log::debug!("Found split row assignment: {var_name}, checking subsequent pipelines");

            block.pipelines[(i + 1)..]
                .iter()
                .find_map(|future_pipeline| {
                    check_for_indexed_variable_access(
                        var_id,
                        &var_name,
                        decl_span,
                        &nu_protocol::ast::Block {
                            pipelines: vec![future_pipeline.clone()],
                            ..Default::default()
                        },
                        context,
                    )
                })
        });

    if let Some(violation) = split_row_violations {
        violations.push(violation);
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    check_block(context.ast, context, &mut violations);

    violations.extend(
        context.collect_rule_violations(|expr, ctx| match &expr.expr {
            Expr::Closure(block_id) | Expr::Block(block_id) => {
                let mut nested_violations = Vec::new();
                let block = ctx.working_set.get_block(*block_id);
                check_block(block, ctx, &mut nested_violations);
                nested_violations
            }
            _ => vec![],
        }),
    );

    violations
}

pub fn rule() -> Rule {
    Rule::new(
        "prefer_parse_command",
        RuleCategory::Idioms,
        Severity::Warning,
        "Prefer 'parse' command over manual string splitting with indexed access",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
