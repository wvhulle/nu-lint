use nu_protocol::{
    Span, Type,
    ast::{Call, Comparison, Expr, Expression, Operator, Pipeline},
};

use crate::{
    LintLevel,
    ast::{block::BlockExt, call::CallExt, expression::ExpressionExt, pipeline::PipelineExt},
    context::LintContext,
    rule::{DetectFix, Rule},
    violation::{Detection, Fix, Replacement},
};

#[derive(Clone, Copy)]
enum SetOperation {
    Union,
    Intersect,
    Difference,
}

impl SetOperation {
    const fn command(self) -> &'static str {
        match self {
            Self::Union => "union",
            Self::Intersect => "intersect",
            Self::Difference => "difference",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            Self::Union => "Appending a list and removing duplicates is what `union` does",
            Self::Intersect => {
                "Keeping items found in another list and removing duplicates is what `intersect` \
                 does"
            }
            Self::Difference => {
                "Dropping items found in another list and removing duplicates is what `difference` \
                 does"
            }
        }
    }
}

pub struct FixData {
    replaced_span: Span,
    operand_span: Span,
    operation: SetOperation,
}

struct SetOperationCandidate<'a> {
    operation: SetOperation,
    operand: &'a Expression,
}

const fn is_list_type(ty: &Type) -> bool {
    matches!(ty, Type::List(_) | Type::Table(_))
}

const fn is_list_operand(expr: &Expression) -> bool {
    is_list_type(&expr.ty)
}

fn is_plain_uniq(call: &Call, context: &LintContext) -> bool {
    call.get_call_name(context) == "uniq" && call.arguments.is_empty()
}

fn append_operand<'a>(call: &'a Call, context: &LintContext) -> Option<&'a Expression> {
    if call.get_call_name(context) != "append" || call.arguments.len() != 1 {
        return None;
    }
    call.get_first_positional_arg()
        .filter(|operand| is_list_operand(operand))
}

fn row_condition_membership<'a>(
    call: &Call,
    context: &'a LintContext,
) -> Option<SetOperationCandidate<'a>> {
    if call.get_call_name(context) != "where" || call.arguments.len() != 1 {
        return None;
    }
    let Expr::RowCondition(block_id) = &call.get_first_positional_arg()?.expr else {
        return None;
    };
    let block = context.working_set.get_block(*block_id);
    let row_var = block.signature.required_positional.first()?.var_id?;
    let [pipeline] = block.pipelines.as_slice() else {
        return None;
    };
    let [element] = pipeline.elements.as_slice() else {
        return None;
    };
    let Expr::BinaryOp(lhs, op, rhs) = &element.expr.expr else {
        return None;
    };
    let operation = match &op.expr {
        Expr::Operator(Operator::Comparison(Comparison::In)) => SetOperation::Intersect,
        Expr::Operator(Operator::Comparison(Comparison::NotIn)) => SetOperation::Difference,
        _ => return None,
    };
    (lhs.extract_direct_var() == Some(row_var) && is_list_operand(rhs)).then_some(
        SetOperationCandidate {
            operation,
            operand: rhs,
        },
    )
}

fn set_operation_before_uniq<'a>(
    call: &'a Call,
    context: &'a LintContext,
) -> Option<SetOperationCandidate<'a>> {
    append_operand(call, context)
        .map(|operand| SetOperationCandidate {
            operation: SetOperation::Union,
            operand,
        })
        .or_else(|| row_condition_membership(call, context))
}

fn input_type(pipeline: &Pipeline, element_index: usize, context: &LintContext) -> Option<Type> {
    let input = pipeline.elements.get(element_index.checked_sub(1)?)?;
    input.expr.infer_output_type(context)
}

fn has_list_input(pipeline: &Pipeline, element_index: usize, context: &LintContext) -> bool {
    input_type(pipeline, element_index, context).is_some_and(|ty| is_list_type(&ty))
}

fn check_pipeline(pipeline: &Pipeline, context: &LintContext) -> Vec<(Detection, FixData)> {
    pipeline
        .find_command_pairs(
            context,
            |call, ctx| set_operation_before_uniq(call, ctx).is_some(),
            is_plain_uniq,
        )
        .into_iter()
        .filter(|pair| has_list_input(pipeline, pair.first_index, context))
        .filter_map(|pair| {
            let SetOperationCandidate { operation, operand } =
                set_operation_before_uniq(pair.first, context)?;
            let replacement = format!(
                "{} {}",
                operation.command(),
                context.span_text(operand.span)
            );
            let detection = Detection::from_global_span(operation.message(), pair.span)
                .with_primary_label(format!("same result as `{replacement}`"))
                .with_extra_label("removes duplicates", pair.second.span());
            Some((
                detection,
                FixData {
                    replaced_span: pair.span,
                    operand_span: operand.span,
                    operation,
                },
            ))
        })
        .collect()
}

struct UniqToSetOperation;

impl DetectFix for UniqToSetOperation {
    type FixInput<'a> = FixData;

    fn id(&self) -> &'static str {
        "uniq_to_set_operation"
    }

    fn short_description(&self) -> &'static str {
        "Use `union`, `intersect` or `difference` instead of combining lists and `uniq`"
    }

    fn long_description(&self) -> Option<&'static str> {
        Some(
            "Nushell 0.114.0 added the set builtins `union`, `intersect` and `difference`. They \
             return deduplicated lists in first-occurrence order, which is exactly what `append \
             $other | uniq`, `where $it in $other | uniq` and `where $it not-in $other | uniq` \
             compute. Without the trailing `uniq` the filters keep duplicates and are not \
             equivalent.",
        )
    }

    fn source_link(&self) -> Option<&'static str> {
        Some(
            "https://www.nushell.sh/blog/2026-07-04-nushell_v0_114_0.html#added-some-commands-for-set-operations-and-combinations",
        )
    }

    fn level(&self) -> LintLevel {
        LintLevel::Hint
    }

    fn detect<'a>(&self, context: &'a LintContext) -> Vec<(Detection, Self::FixInput<'a>)> {
        context.ast.detect_in_pipelines(context, check_pipeline)
    }

    fn fix(&self, context: &LintContext, fix_data: &Self::FixInput<'_>) -> Option<Fix> {
        let command = fix_data.operation.command();
        let operand = context.span_text(fix_data.operand_span);
        Some(Fix {
            explanation: format!("Replace with `{command} {operand}`").into(),
            replacements: vec![Replacement::new(
                fix_data.replaced_span,
                format!("{command} {operand}"),
            )],
        })
    }
}

pub static RULE: &dyn Rule = &UniqToSetOperation;

#[cfg(test)]
mod detect_bad;
#[cfg(test)]
mod generated_fix;
#[cfg(test)]
mod ignore_good;
