use nu_protocol::{
    Span, VarId,
    ast::{Block, Call, Expr, Expression, Pipeline},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::Rule,
    violation::{Fix, Replacement, Violation},
};

struct LetDeclaration<'a> {
    var_id: VarId,
    var_name: String,
    span: Span,
    call: &'a Call,
}

fn extract_let_declaration<'a>(
    expr: &'a Expression,
    context: &LintContext,
) -> Option<LetDeclaration<'a>> {
    let Expr::Call(call) = &expr.expr else {
        return None;
    };

    if !call.is_call_to_command("let", context) {
        return None;
    }

    let (var_id, var_name, _var_span) = call.extract_variable_declaration(context)?;
    Some(LetDeclaration {
        var_id,
        var_name,
        span: expr.span,
        call,
    })
}

fn extract_value_from_let(call: &Call) -> Option<&Expression> {
    call.get_positional_arg(1)
}

/// Check if an expression is just a variable reference
fn extract_var_reference(expr: &Expression) -> Option<VarId> {
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
fn is_simple_var_reference(pipeline: &Pipeline, var_id: VarId) -> Option<Span> {
    if pipeline.elements.len() != 1 {
        return None;
    }

    let element = pipeline.elements.first()?;
    let referenced_var_id = extract_var_reference(&element.expr)?;

    (referenced_var_id == var_id).then_some(element.expr.span)
}

/// Check a block for the unnecessary variable pattern
fn check_block(block: &Block, context: &LintContext, violations: &mut Vec<Violation>) {
    let pipelines = &block.pipelines;

    for i in 0..pipelines.len().saturating_sub(1) {
        let current_pipeline = &pipelines[i];
        let next_pipeline = &pipelines[i + 1];

        // Check if current pipeline has a single element
        if current_pipeline.elements.len() != 1 {
            continue;
        }

        let element = &current_pipeline.elements[0];

        let Some(let_decl) = extract_let_declaration(&element.expr, context) else {
            continue;
        };

        if let Some(ref_span) = is_simple_var_reference(next_pipeline, let_decl.var_id) {
            log::debug!(
                "Found unnecessary variable pattern: let {} = ... followed by ${}",
                let_decl.var_name,
                let_decl.var_name
            );
            let combined_span = Span::new(let_decl.span.start, ref_span.end);

            let value_expr = extract_value_from_let(let_decl.call);
            let replacement_text = value_expr
                .map(|expr| expr.span_text(context).to_string())
                .unwrap_or_default();

            let fix = Fix::with_explanation(
                format!("Return expression directly: {replacement_text}"),
                vec![Replacement::new(combined_span, replacement_text)],
            );

            violations.push(
                Violation::new(
                    format!(
                        "Variable '{}' is assigned and immediately returned - consider returning \
                         the expression directly",
                        let_decl.var_name
                    ),
                    let_decl.span,
                )
                .with_primary_label("variable declared here")
                .with_extra_label("immediately returned here", ref_span)
                .with_help(
                    "Return the expression directly instead of assigning to a variable first",
                )
                .with_fix(fix),
            );
        }
    }
}

fn check(context: &LintContext) -> Vec<Violation> {
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

pub const fn rule() -> Rule {
    Rule::new(
        "unnecessary_variable_before_return",
        "Variable assigned and immediately returned adds unnecessary verbosity",
        check,
        LintLevel::Warning,
    )
    .with_doc_url("https://www.nushell.sh/book/thinking_in_nu.html#implicit-return")
}

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
