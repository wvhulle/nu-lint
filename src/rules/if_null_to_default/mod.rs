use nu_protocol::{
    Span, VarId,
    ast::{Block, Call, Comparison, Expr, Expression, Operator},
};

use crate::{
    LintLevel,
    ast::{call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

struct FixData {
    full_span: Span,
    variable: Span,
    fallback: Span,
}

/// Get the single expression from a block, if it contains exactly one.
fn get_single_block_expr(block: &Block) -> Option<&Expression> {
    let [pipeline] = block.pipelines.as_slice() else {
        return None;
    };
    let [element] = pipeline.elements.as_slice() else {
        return None;
    };
    Some(&element.expr)
}

/// Extract `(var_id, var_span, is_equal)` from a null comparison.
fn extract_null_comparison(expr: &Expression) -> Option<(VarId, Span, bool)> {
    let Expr::BinaryOp(left, op, right) = &expr.expr else {
        return None;
    };

    let is_equal = match &op.expr {
        Expr::Operator(Operator::Comparison(Comparison::Equal)) => true,
        Expr::Operator(Operator::Comparison(Comparison::NotEqual)) => false,
        _ => return None,
    };

    // `$var == null` or `$var != null`
    if matches!(&right.expr, Expr::Nothing)
        && let Some(var_id) = left.extract_direct_var()
    {
        return Some((var_id, left.span, is_equal));
    }

    // `null == $var` or `null != $var`
    if matches!(&left.expr, Expr::Nothing)
        && let Some(var_id) = right.extract_direct_var()
    {
        return Some((var_id, right.span, is_equal));
    }

    None
}

fn detect(call: &Call, expr_span: Span, context: &LintContext) -> Option<(Detection, FixData)> {
    if !call.is_call_to_command("if", context) {
        return None;
    }

    let condition = call.get_first_positional_arg()?;
    let (var_id, var_span, is_equal) = extract_null_comparison(condition)?;

    let then_block_id = call.get_positional_arg(1)?.extract_block_id()?;
    let then_block = context.working_set.get_block(then_block_id);

    let (is_else_if, else_expr) = call.get_else_branch()?;
    if is_else_if {
        return None;
    }

    let else_block_id = else_expr.extract_block_id()?;
    let else_block = context.working_set.get_block(else_block_id);

    // Pattern 1: `if $x == null { default } else { $x }`
    // Pattern 2: `if $x != null { $x } else { default }`
    let (var_block, default_block) = if is_equal {
        (else_block, then_block)
    } else {
        (then_block, else_block)
    };

    // var_block must return only the variable
    if !get_single_block_expr(var_block)?.matches_var(var_id) {
        return None;
    }

    let default_span = get_single_block_expr(default_block)?.span;

    let detection = Detection::from_global_span(
        "This if-null pattern can be simplified with the `default` command",
        expr_span,
    )
    .with_primary_label("simplify with `| default`");

    Some((
        detection,
        FixData {
            full_span: expr_span,
            variable: var_span,
            fallback: default_span,
        },
    ))
}

struct IfNullToDefault;

impl DetectFix for IfNullToDefault {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "if_null_to_default"
    }

    fn short_description(&self) -> &'static str {
        "Replace `if $x == null { default } else { $x }` with `$x | default default`"
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/default.html")
    }

    fn level(&self) -> Option<LintLevel> {
        Some(LintLevel::Warning)
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.detect_with_fix_data(|expr, ctx| {
            if let Expr::Call(call) = &expr.expr {
                detect(call, expr.span, ctx).into_iter().collect()
            } else {
                vec![]
            }
        })
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let var_text = context.span_text(fix_data.variable);
        let default_text = context.span_text(fix_data.fallback);
        let replacement = format!("{var_text} | default {default_text}");

        Some(Fix::with_explanation(
            format!("Simplify to: {replacement}"),
            vec![Replacement::new(fix_data.full_span, replacement)],
        ))
    }
}

pub static RULE: &dyn Rule = &IfNullToDefault;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
