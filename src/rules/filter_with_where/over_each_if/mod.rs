use nu_protocol::ast::{Argument, Call, Expr, Expression, Traverse};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, expression::ExpressionExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};

fn get_if_then_block(call: &Call) -> Option<&Expression> {
    call.arguments.get(1).and_then(|arg| match arg {
        Argument::Positional(expr) | Argument::Unknown(expr) => Some(expr),
        _ => None,
    })
}

fn then_block_returns_loop_var(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);

    block.pipelines.len() == 1
        && block.pipelines[0].elements.len() == 1
        && block.pipelines[0].elements[0]
            .expr
            .refers_to_variable(context, loop_var_name)
}

fn is_filtering_pattern(
    block_id: nu_protocol::BlockId,
    context: &LintContext,
    loop_var_name: &str,
) -> bool {
    let block = context.working_set.get_block(block_id);

    if block.pipelines.len() != 1 || block.pipelines[0].elements.len() != 1 {
        return false;
    }

    let elem = &block.pipelines[0].elements[0];

    let Expr::Call(call) = &elem.expr.expr else {
        return false;
    };

    if call.get_call_name(context) != "if" {
        return false;
    }

    let Some(then_block_expr) = get_if_then_block(call) else {
        return false;
    };

    let Expr::Block(then_block_id) = &then_block_expr.expr else {
        return false;
    };

    let then_block = context.working_set.get_block(*then_block_id);
    !then_block.has_side_effects()
        && then_block_returns_loop_var(*then_block_id, context, loop_var_name)
}

fn extract_each_block_id(call: &Call) -> Option<nu_protocol::BlockId> {
    call.arguments.first().and_then(|arg| match arg {
        Argument::Positional(expr) | Argument::Unknown(expr) => match &expr.expr {
            Expr::Block(id) | Expr::Closure(id) => Some(*id),
            _ => None,
        },
        _ => None,
    })
}

fn check_expression(expr: &Expression, context: &LintContext) -> Vec<Detection> {
    let Expr::Call(call) = &expr.expr else {
        return vec![];
    };

    if call.get_call_name(context) != "each" {
        return vec![];
    }

    let Some(loop_var_name) = call.loop_var_from_each(context) else {
        return vec![];
    };

    let Some(block_id) = extract_each_block_id(call) else {
        return vec![];
    };

    is_filtering_pattern(block_id, context, &loop_var_name)
        .then(|| {
            let block = context.working_set.get_block(block_id);
            let block_span = block.span.unwrap_or(expr.span);
            Detection::from_global_span(
                "Consider using 'where' for filtering instead of 'each' with 'if'",
                expr.span,
            )
            .with_primary_label("each with if pattern")
            .with_extra_label("filtering logic inside closure", block_span)
            .with_help("Use '$list | where <condition>' for readability")
        })
        .into_iter()
        .collect()
}

struct WhereInsteadEachThenIf;

impl DetectFix for WhereInsteadEachThenIf {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "where_instead_each_then_if"
    }

    fn explanation(&self) -> &'static str {
        "Use 'where' for filtering instead of 'each' with 'if'"
    }

    fn doc_url(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/where.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Warning
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        let mut violations = Vec::new();
        context.ast.flat_map(
            context.working_set,
            &|expr| check_expression(expr, context),
            &mut violations,
        );

        Self::no_fix(violations)
    }
}

pub static RULE: &dyn Rule = &WhereInsteadEachThenIf;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;
