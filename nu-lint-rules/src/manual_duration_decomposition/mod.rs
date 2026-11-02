use std::collections::HashSet;

use nu_lint_analysis::{
    LintLevel,
    ast::expression::ExpressionExt,
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::Detection,
};
use nu_protocol::{
    Span, VarId,
    ast::{Expr, Expression, Math, Operator, Traverse},
};

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod ignore_good;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DecompositionStep {
    FloorDivision,
    Modulo,
}

struct TimeUnitOperation {
    variable_id: VarId,
    step: DecompositionStep,
    span: Span,
}

fn extract_time_unit_operation(operation_expr: &Expression) -> Option<TimeUnitOperation> {
    let Expr::BinaryOp(left_operand, operator, right_operand) = &operation_expr.expr else {
        return None;
    };
    let step = match &operator.expr {
        Expr::Operator(Operator::Math(Math::FloorDivide)) => DecompositionStep::FloorDivision,
        Expr::Operator(Operator::Math(Math::Modulo)) => DecompositionStep::Modulo,
        _ => return None,
    };
    if !matches!(right_operand.expr, Expr::Int(60 | 3600 | 86400)) {
        return None;
    }

    Some(TimeUnitOperation {
        variable_id: left_operand.extract_direct_var()?,
        step,
        span: operation_expr.span,
    })
}

fn collect_decompositions(context: &LintContext) -> Vec<Detection> {
    let mut operations = Vec::new();
    let collect_operation = |visited_expr: &Expression| {
        extract_time_unit_operation(visited_expr)
            .into_iter()
            .collect()
    };
    context
        .ast
        .flat_map(context.working_set, &collect_operation, &mut operations);

    let mut reported_variables = HashSet::new();
    operations
        .iter()
        .filter(|operation| operation.step == DecompositionStep::Modulo)
        .filter_map(|modulo_operation| {
            let floor_operation = operations.iter().find(|operation| {
                operation.step == DecompositionStep::FloorDivision
                    && operation.variable_id == modulo_operation.variable_id
            })?;
            reported_variables
                .insert(modulo_operation.variable_id)
                .then(|| {
                    Detection::from_global_span(
                        "Convert to a duration and use `into record` instead of splitting time \
                         units by hand",
                        modulo_operation.span,
                    )
                    .with_primary_label("remainder of a time unit")
                    .with_extra_label("quotient of a time unit", floor_operation.span)
                })
        })
        .collect()
}

struct ManualDurationDecomposition;

impl DetectFix for ManualDurationDecomposition {
    type FixInput<'a> = ();

    fn id(&self) -> &'static str {
        "manual_duration_decomposition"
    }

    fn short_description(&self) -> &'static str {
        "Integer division and modulo used to split a duration"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Taking both the quotient and the remainder of one variable by 60, 3600 or 86400 \
             splits a time value into units by hand. Multiply the value by a unit such as `1sec` \
             to get a duration, then use `into record` to read its hours, minutes and seconds.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some("https://www.nushell.sh/commands/docs/into_record.html")
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        Self::no_fix(collect_decompositions(context))
    }
}

pub static RULE: &dyn Rule = &ManualDurationDecomposition;
