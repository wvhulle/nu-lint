use nu_protocol::{Span, VarId, ast::Expr};

use crate::{
    ast::{CallExt, SpanExt},
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

fn is_split_row_call(call: &nu_protocol::ast::Call, context: &LintContext) -> bool {
    let result = call.is_call_to_command("split row", context);
    log::debug!("is_split_row_call: {result}");
    result
}

fn is_indexed_access_call(call: &nu_protocol::ast::Call, context: &LintContext) -> bool {
    let name = call.get_call_name(context);
    let result = matches!(name.as_str(), "get" | "skip");
    log::debug!("is_indexed_access_call for '{name}': {result}");
    result
}

fn has_index_argument(call: &nu_protocol::ast::Call, context: &LintContext) -> bool {
    let result = call.get_first_positional_arg().is_some_and(|arg| {
        let arg_text = arg.span.text(context);
        log::debug!("Checking if '{arg_text}' is an index");
        arg_text.parse::<usize>().is_ok()
    });
    log::debug!("has_index_argument: {result}");
    result
}

fn check_pipeline_for_split_get(
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> Option<RuleViolation> {
    log::debug!(
        "check_pipeline_for_split_get: pipeline has {} elements",
        pipeline.elements.len()
    );

    if pipeline.elements.len() < 2 {
        return None;
    }

    for i in 0..pipeline.elements.len() - 1 {
        let current = &pipeline.elements[i];
        let next = &pipeline.elements[i + 1];

        log::debug!("Checking elements {} and {}", i, i + 1);

        let Expr::Call(current_call) = &current.expr.expr else {
            log::debug!("Element {i} is not a call");
            continue;
        };

        let Expr::Call(next_call) = &next.expr.expr else {
            log::debug!("Element {} is not a call", i + 1);
            continue;
        };

        if is_split_row_call(current_call, context)
            && is_indexed_access_call(next_call, context)
            && has_index_argument(next_call, context)
        {
            log::debug!("Found split row + indexed access pattern!");
            let span = Span::new(current.expr.span.start, next.expr.span.end);
            return Some(
                RuleViolation::new_static(
                    "prefer_parse_command",
                    "Manual string splitting with indexed access - consider using 'parse'",
                    span,
                )
                .with_suggestion_static(
                    "Use 'parse \"pattern {field1} {field2}\"' for structured text extraction",
                ),
            );
        }
    }

    None
}

fn check_expression_for_split_row(expr: &Expr, context: &LintContext) -> bool {
    match expr {
        Expr::Call(call) if is_split_row_call(call, context) => true,
        Expr::FullCellPath(cp) => check_expression_for_split_row(&cp.head.expr, context),
        Expr::Subexpression(block_id) => {
            let block = context.working_set.get_block(*block_id);
            block.pipelines.iter().any(|p| has_split_row_in_pipeline(p, context))
        }
        _ => false,
    }
}

fn has_split_row_in_pipeline(pipeline: &nu_protocol::ast::Pipeline, context: &LintContext) -> bool {
    pipeline.elements.iter().any(|element| {
        check_expression_for_split_row(&element.expr.expr, context)
    })
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
    log::debug!("Found let statement for variable: {var_name}");

    let value_expr = call.get_positional_arg(1)?;
    log::debug!(
        "Value expr type: {:?}",
        std::mem::discriminant(&value_expr.expr)
    );
    
    // Log what we're actually checking
    match &value_expr.expr {
        Expr::Call(_) => log::debug!("Value expr is Call"),
        Expr::Subexpression(_) => log::debug!("Value expr is Subexpression"),
        Expr::FullCellPath(_) => log::debug!("Value expr is FullCellPath"),
        Expr::Block(_) => log::debug!("Value expr is Block"),
        _ => log::debug!("Value expr is something else: {value_expr:?}"),
    }

    // Check direct call
    if let Expr::Call(value_call) = &value_expr.expr
        && is_split_row_call(value_call, context)
    {
        log::debug!("Variable {var_name} is assigned from split row (direct call)");
        return Some((var_id, var_name, expr.span));
    }

    // Check call in cell path
    if let Expr::FullCellPath(cell_path) = &value_expr.expr {
        if let Expr::Call(head_call) = &cell_path.head.expr
            && is_split_row_call(head_call, context)
        {
            log::debug!("Variable {var_name} is assigned from split row (cell path)");
            return Some((var_id, var_name, expr.span));
        }

        // Check subexpression in cell path
        if let Expr::Subexpression(block_id) = &cell_path.head.expr {
            let block = context.working_set.get_block(*block_id);
            if block.pipelines.len() == 1 && has_split_row_in_pipeline(&block.pipelines[0], context)
            {
                log::debug!("Variable {var_name} is assigned from split row (subexpression)");
                return Some((var_id, var_name, expr.span));
            }
        }
    }

    // Check subexpression directly
    if let Expr::Subexpression(block_id) = &value_expr.expr {
        log::debug!("Found direct subexpression with block_id: {block_id:?}");
        let block = context.working_set.get_block(*block_id);
        log::debug!("Block has {} pipelines", block.pipelines.len());
        if block.pipelines.len() == 1 && has_split_row_in_pipeline(&block.pipelines[0], context) {
            log::debug!("Variable {var_name} is assigned from split row (direct subexpression)");
            return Some((var_id, var_name, expr.span));
        }
    }

    // Check block directly (for closures/blocks in let assignments)
    if let Expr::Block(block_id) = &value_expr.expr {
        log::debug!("Found direct block with block_id: {block_id:?}");
        let block = context.working_set.get_block(*block_id);
        log::debug!("Block has {} pipelines", block.pipelines.len());
        if block.pipelines.len() == 1 && has_split_row_in_pipeline(&block.pipelines[0], context) {
            log::debug!("Variable {var_name} is assigned from split row (direct block)");
            return Some((var_id, var_name, expr.span));
        }
        log::debug!("Block does not have split row in single pipeline");
    }

    log::debug!("Variable {var_name} is NOT from split row");
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
    .with_suggestion_static(
        "Use 'parse' command to extract named fields instead of indexed access",
    )
}

fn check_element_for_indexed_access(
    element: &nu_protocol::ast::PipelineElement,
    var_id: VarId,
    var_name: &str,
    decl_span: Span,
    pipeline: &nu_protocol::ast::Pipeline,
    context: &LintContext,
) -> Option<RuleViolation> {
    // Check FullCellPath with subexpression
    if let Expr::FullCellPath(cp) = &element.expr.expr
        && let Expr::Subexpression(block_id) = &cp.head.expr
    {
        let nested_block = context.working_set.get_block(*block_id);
        return check_for_indexed_variable_access(
            var_id, var_name, decl_span, nested_block, context
        );
    }
    
    // Check Call expressions
    if let Expr::Call(call) = &element.expr.expr {
        let has_var_usage = is_var_used_in_indexed_access(var_id, call, context)
            || (is_indexed_access_call(call, context)
                && has_index_argument(call, context)
                && is_var_piped_to_indexed_access(var_id, pipeline));

        if has_var_usage {
            return Some(create_indexed_access_violation(var_name, decl_span));
        }
        
        // Recursively check blocks in call arguments
        for arg in &call.arguments {
            if let nu_protocol::ast::Argument::Positional(arg_expr) 
            | nu_protocol::ast::Argument::Unknown(arg_expr) = arg
                && let Expr::Block(block_id) = &arg_expr.expr
            {
                let nested_block = context.working_set.get_block(*block_id);
                if let Some(violation) = check_for_indexed_variable_access(
                    var_id, var_name, decl_span, nested_block, context
                ) {
                    return Some(violation);
                }
            }
        }
    }
    
    // Check Block expressions directly
    if let Expr::Block(block_id) = &element.expr.expr {
        let nested_block = context.working_set.get_block(*block_id);
        return check_for_indexed_variable_access(
            var_id, var_name, decl_span, nested_block, context
        );
    }
    
    None
}

#[allow(clippy::excessive_nesting)]
fn check_for_indexed_variable_access(
    var_id: VarId,
    var_name: &str,
    decl_span: Span,
    block: &nu_protocol::ast::Block,
    context: &LintContext,
) -> Option<RuleViolation> {
    for pipeline in &block.pipelines {
        // Check if this pipeline uses our variable
        let var_found_in_pipeline = pipeline.elements.iter().any(|elem| {
            match &elem.expr.expr {
                Expr::Var(v_id) if *v_id == var_id => true,
                Expr::FullCellPath(cp) => matches!(&cp.head.expr, Expr::Var(v_id) if *v_id == var_id),
                _ => false,
            }
        });

        if var_found_in_pipeline {
            // Check if any element in this pipeline is an indexed access
            for element in &pipeline.elements {
                if let Expr::Call(call) = &element.expr.expr
                    && is_indexed_access_call(call, context)
                    && has_index_argument(call, context)
                {
                    return Some(create_indexed_access_violation(var_name, decl_span));
                }
            }
        }

        // Recursively check nested expressions
        for element in &pipeline.elements {
            if let Some(violation) = check_element_for_indexed_access(
                element, var_id, var_name, decl_span, pipeline, context
            ) {
                return Some(violation);
            }
        }
    }

    None
}

fn check_block(
    block: &nu_protocol::ast::Block,
    context: &LintContext,
    violations: &mut Vec<RuleViolation>,
) {
    log::debug!("check_block: {} pipelines", block.pipelines.len());

    for pipeline in &block.pipelines {
        if let Some(violation) = check_pipeline_for_split_get(pipeline, context) {
            violations.push(violation);
        }
    }

    for i in 0..block.pipelines.len() {
        let pipeline = &block.pipelines[i];

        if pipeline.elements.len() != 1 {
            continue;
        }

        let element = &pipeline.elements[0];
        let Some((var_id, var_name, decl_span)) =
            extract_split_row_assignment(&element.expr, context)
        else {
            continue;
        };

        log::debug!("Found split row assignment: {var_name}, checking future pipelines");

        for j in (i + 1)..block.pipelines.len() {
            let future_pipeline = &block.pipelines[j];
            if let Some(violation) = check_for_indexed_variable_access(
                var_id,
                &var_name,
                decl_span,
                &nu_protocol::ast::Block {
                    pipelines: vec![future_pipeline.clone()],
                    ..Default::default()
                },
                context,
            ) {
                violations.push(violation);
                break;
            }
        }
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
