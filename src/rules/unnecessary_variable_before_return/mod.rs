use nu_protocol::{Span, VarId, ast::Expr};

use crate::{
    ast::CallExt,
    context::LintContext,
    rule::{Rule, RuleCategory},
    violation::{RuleViolation, Severity},
};

/// Extract variable declaration from a `let` statement  
fn extract_let_declaration(
    expr: &nu_protocol::ast::Expression,
    context: &LintContext,
) -> Option<(VarId, String, Span)> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    // Only check for 'let', not 'mut' - mut variables may be modified before return
    if !call.is_call_to_command("let", context) {
        return None;
    }

    // Use the helper but return the full expression span (the let statement)
    let (var_id, var_name, _var_span) = call.extract_variable_declaration(context)?;
    Some((var_id, var_name, expr.span))
}

/// Check if an expression is just a variable reference
fn extract_var_reference(expr: &nu_protocol::ast::Expression) -> Option<VarId> {
    match &expr.expr {
        Expr::Var(var_id) | Expr::VarDecl(var_id) => Some(*var_id),
        Expr::FullCellPath(cell_path) if cell_path.tail.is_empty() => {
            // Simple variable without path members (e.g., $var, not $var.field)
            match &cell_path.head.expr {
                Expr::Var(var_id) => Some(*var_id),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Check if a pipeline contains only a single variable reference
fn is_simple_var_reference(pipeline: &nu_protocol::ast::Pipeline, var_id: VarId) -> Option<Span> {
    if pipeline.elements.len() != 1 {
        return None;
    }

    let element = pipeline.elements.first()?;
    let referenced_var_id = extract_var_reference(&element.expr)?;

    if referenced_var_id == var_id {
        Some(element.expr.span)
    } else {
        None
    }
}

/// Check a block for the unnecessary variable pattern
fn check_block(
    block: &nu_protocol::ast::Block,
    context: &LintContext,
    violations: &mut Vec<RuleViolation>,
) {
    let pipelines = &block.pipelines;

    for i in 0..pipelines.len().saturating_sub(1) {
        let current_pipeline = &pipelines[i];
        let next_pipeline = &pipelines[i + 1];

        // Check if current pipeline has a single element
        if current_pipeline.elements.len() != 1 {
            continue;
        }

        let element = &current_pipeline.elements[0];

        // Try to extract a let declaration
        let Some((var_id, var_name, decl_span)) = extract_let_declaration(&element.expr, context)
        else {
            continue;
        };

        // Check if the next pipeline is just a reference to the same variable
        if let Some(ref_span) = is_simple_var_reference(next_pipeline, var_id) {
            log::debug!(
                "Found unnecessary variable pattern: let {var_name} = ... followed by ${var_name}"
            );
            let combined_span = Span::new(decl_span.start, ref_span.end);

            violations.push(
                RuleViolation::new_dynamic(
                    "unnecessary_variable_before_return",
                    format!(
                        "Variable '{var_name}' is assigned and immediately returned - consider \
                         returning the expression directly"
                    ),
                    combined_span,
                )
                .with_suggestion_static(
                    "Return the expression directly instead of assigning to a variable first",
                ),
            );
        }
    }
}

fn check(context: &LintContext) -> Vec<RuleViolation> {
    let mut violations = Vec::new();

    // Check the main block
    check_block(context.ast, context, &mut violations);

    // Recursively check all nested blocks (closures, functions, etc.)
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
        "unnecessary_variable_before_return",
        RuleCategory::CodeQuality,
        Severity::Warning,
        "Variable assigned and immediately returned adds unnecessary verbosity",
        check,
    )
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
