use nu_protocol::{
    Span,
    ast::{Assignment, Expr, Expression, Operator, Pipeline},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

fn refers_to_same_variable(expr1: &Expression, expr2: &Expression, context: &LintContext) -> bool {
    match (
        expr1.extract_variable_name(context),
        expr2.extract_variable_name(context),
    ) {
        (Some(name1), Some(name2)) => name1 == name2,
        _ => false,
    }
}

fn is_list_expression(expr: &Expression) -> bool {
    match &expr.expr {
        Expr::List(_) => true,
        Expr::FullCellPath(path) => matches!(&path.head.expr, Expr::List(_)),
        _ => false,
    }
}

/// Semantic fix data: stores spans and metadata needed to generate fix
pub struct FixData {
    full_span: Span,
    var_span: Span,
    append_value_span: Span,
    value_is_list: bool,
}

/// Helper to create a violation for the append-to-concat-assign pattern
fn create_violation(
    var_span: Span,
    rhs_label: &'static str,
    rhs_span: Span,
    append_value_span: Span,
    ctx: &LintContext,
) -> Detection {
    let var_text = ctx.plain_text(var_span);
    Detection::from_global_span(
        format!("Use ++= operator: {var_text} ++= instead of {var_text} = {var_text} | append ..."),
        var_span,
    )
    .with_primary_label("variable being reassigned")
    .with_extra_label(rhs_label, rhs_span)
    .with_extra_label("value being appended", append_value_span)
}

/// Unwrap nested subexpression wrappers that Nushell's parser creates
fn unwrap_pipeline<'a>(pipeline: &'a Pipeline, ctx: &'a LintContext) -> &'a Pipeline {
    // Nushell wraps `($list | append $x)` as:
    // Pipeline with 1 FullCellPath element, whose head is a Subexpression
    // We need to unwrap to get the actual 2-element pipeline
    if pipeline.elements.len() == 1
        && let Expr::FullCellPath(cell_path) = &pipeline.elements[0].expr.expr
        && let Expr::Subexpression(inner_block_id) = &cell_path.head.expr
    {
        let inner_block = ctx.working_set.get_block(*inner_block_id);
        if let Some(inner_pipeline) = inner_block.pipelines.first() {
            return inner_pipeline;
        }
    }
    pipeline
}

/// Detect Pattern 2: $list = $list | append $x (at pipeline level)
/// This checks entire pipelines for the pattern where:
/// - First element is an assignment ($list = $list)
/// - Second element is a call to `append`
fn detect_pipeline_level_pattern(
    pipeline: &Pipeline,
    ctx: &LintContext,
) -> Vec<(Detection, FixData)> {
    if pipeline.elements.len() < 2 {
        return Vec::new();
    }

    // Check if first element is an assignment
    let first_elem = &pipeline.elements[0];
    let Expr::BinaryOp(left, op_expr, right) = &first_elem.expr.expr else {
        return Vec::new();
    };

    let Expr::Operator(Operator::Assignment(Assignment::Assign)) = &op_expr.expr else {
        return Vec::new();
    };

    // Check if RHS of assignment matches LHS (same variable)
    if !refers_to_same_variable(left, right, ctx) {
        return Vec::new();
    }

    // Check if second element is an append call
    let second_elem = &pipeline.elements[1];
    let Expr::Call(call) = &second_elem.expr.expr else {
        return Vec::new();
    };

    if !call.is_call_to_command("append", ctx) {
        return Vec::new();
    }

    // Extract the value being appended
    let Some(append_value) = call.get_first_positional_arg() else {
        return Vec::new();
    };

    let full_span = Span::new(first_elem.expr.span.start, second_elem.expr.span.end);

    let violation = create_violation(
        left.span,
        "same variable on RHS",
        right.span,
        append_value.span,
        ctx,
    );

    let fix_data = FixData {
        full_span,
        var_span: left.span,
        append_value_span: append_value.span,
        value_is_list: is_list_expression(append_value),
    };

    vec![(violation, fix_data)]
}

/// Detect Pattern 1: $list = ($list | append $x) (with
/// parentheses/subexpression)
fn detect_append_assignment(expr: &Expression, ctx: &LintContext) -> Option<(Detection, FixData)> {
    // Check if this is an assignment expression
    let Expr::BinaryOp(left, op_expr, right) = &expr.expr else {
        return None;
    };

    let Expr::Operator(Operator::Assignment(Assignment::Assign)) = &op_expr.expr else {
        return None;
    };

    // RHS must be a subexpression (the parentheses)
    let Expr::Subexpression(block_id) = &right.expr else {
        return None;
    };

    // Get the pipeline inside the subexpression and unwrap if needed
    let block = ctx.working_set.get_block(*block_id);
    let pipeline = block.pipelines.first()?;
    let actual_pipeline = unwrap_pipeline(pipeline, ctx);

    // Must be a 2-element pipeline: variable | append
    if actual_pipeline.elements.len() != 2 {
        return None;
    }

    let first_elem = &actual_pipeline.elements[0].expr;
    let second_elem = &actual_pipeline.elements[1].expr;

    // First element must match the LHS variable
    if !refers_to_same_variable(left, first_elem, ctx) {
        return None;
    }

    // Second element must be an append call
    let Expr::Call(call) = &second_elem.expr else {
        return None;
    };

    if !call.is_call_to_command("append", ctx) {
        return None;
    }

    let append_value = call.get_first_positional_arg()?;

    let violation = create_violation(
        left.span,
        "same variable repeated in pipeline",
        first_elem.span,
        append_value.span,
        ctx,
    );

    let fix_data = FixData {
        full_span: expr.span,
        var_span: left.span,
        append_value_span: append_value.span,
        value_is_list: is_list_expression(append_value),
    };

    Some((violation, fix_data))
}

struct AppendToConcatAssign;

impl DetectFix for AppendToConcatAssign {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "append_to_concat_assign"
    }

    fn short_description(&self) -> &'static str {
        "Use ++= operator instead of verbose append in assignment"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/book/operators.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut results = Vec::new();

        // Pattern 1: Detect assignments with subexpressions ($list = ($list | append
        // $x))
        results.extend(context.detect_with_fix_data(|expr, ctx| {
            detect_append_assignment(expr, ctx).into_iter().collect()
        }));

        // Pattern 2: Detect pipeline-level patterns ($list = $list | append $x)
        results.extend(
            context
                .ast
                .detect_in_pipelines(context, detect_pipeline_level_pattern),
        );

        results
    }

    fn fix(&self, ctx: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let var_text = ctx.plain_text(fix_data.var_span);
        let value_text = ctx.plain_text(fix_data.append_value_span);

        // If value is already a list, don't wrap it
        let wrapped_value = if fix_data.value_is_list {
            value_text.to_string()
        } else {
            format!("[{value_text}]")
        };

        let new_text = format!("{var_text} ++= {wrapped_value}");

        Some(Fix::with_explanation(
            format!("Replace with concat assignment: {new_text}"),
            vec![Replacement::new(fix_data.full_span, new_text)],
        ))
    }
}

pub static RULE: &dyn Rule = &AppendToConcatAssign;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
